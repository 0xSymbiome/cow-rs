//! `verify-sol-provenance` subcommand.
//!
//! Mechanically gates every `.sol` file under `crates/contracts/abi/`
//! against the canonical provenance manifest embedded in
//! `parity/source-lock.yaml`, per the discipline codified in
//! `docs/adr/0012-alloy-sol-bindings-and-registry-authority.md`.
//!
//! The default and only shipped posture is the byte-identical mirror:
//! each repository entry in `parity/source-lock.yaml` declares a
//! `vendored:` array whose rows carry the local `.sol` path, the
//! upstream path under that repository, and the SHA-256 of the
//! upstream file's bytes at the repository's pinned commit. The
//! verifier SHA-256-checks every local file listed in any `vendored:`
//! array against the manifest's recorded digest. With `--upstream-root`
//! it additionally re-derives the SHA from the live upstream checkout
//! via `git show <commit>:<path>` and (with `--refresh-sha256`) updates
//! the manifest. With `--upstream-github` it fetches each row from
//! `https://raw.githubusercontent.com/<owner>/<repo>/<commit>/<path>`
//! and compares against the manifest, so CI verifies the manifest
//! against GitHub canonical content on every run without requiring any
//! local upstream clone.
//!
//! The verifier also recognises a provenance-headed excerpt fallback
//! for files whose canonical upstream cannot be vendored as a single
//! byte-stream (the file's first ~50 lines carry a `// Provenance`
//! header listing the upstream sources folded into the excerpt; the
//! verifier walks every non-header line and requires substring presence
//! in at least one named upstream when `--upstream-root` is provided).
//! No currently-shipped file uses this fallback — all 37 vendored
//! `.sol` files under `crates/contracts/abi/` are byte-identical
//! mirrors gated by the SHA-256 contract above.
//!
//! A `.sol` file FAILS unclassified when it is not listed in any
//! `vendored:` array AND its header is missing or malformed.
//!
//! Implementation notes:
//!
//! 1. **`Pinned commit:` is optional in the excerpt-fallback header.**
//!    When omitted, the verifier emits an advisory WARN and continues;
//!    when present it MUST match the source-lock-pinned commit for the
//!    named repository or the file FAILS.
//! 2. **Byte-identical mirror wins over excerpt fallback** when both
//!    markers are present (the manifest entry is the source of truth;
//!    any Provenance header in such a file is structurally checked but
//!    does not gate the SHA-256 comparison).
//! 3. **`out/*.json` siblings under `composable-cow/out/`** are forge
//!    artifacts, not `.sol` files; the walker only collects `*.sol`, so
//!    these are skipped without an explicit filter.
//! 4. **Symlinks and non-UTF-8 bytes FAIL.** Every vendored `.sol`
//!    must be a real LF-encoded UTF-8 text file.
//! 5. **Files outside `crates/contracts/abi/`** are never visited by
//!    the walker, which is rooted under that path.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use clap::Args;
use serde::Deserialize;
use sha2::{Digest, Sha256};

/// CLI arguments for the `verify-sol-provenance` subcommand.
#[derive(Args, Debug)]
pub struct VerifySolProvenanceArgs {
    /// Path to the source-lock manifest. Used to resolve `vendored:`
    /// SHA-256 rows and (when `Pinned commit:` is present in an
    /// excerpt-fallback header) to validate header pin coherence.
    #[arg(long, default_value = "parity/source-lock.yaml")]
    pub source_lock: PathBuf,

    /// Root directory containing the vendored `.sol` corpus.
    #[arg(long, default_value = "crates/contracts/abi")]
    pub abi_root: PathBuf,

    /// Optional path to a parent directory holding source-lock-pinned
    /// upstream checkouts under `<upstream-root>/<repo-id>/`. When
    /// provided, the verifier (a) for byte-identical mirrors, re-derives
    /// the upstream SHA-256 via `git show <commit>:<path>` and compares
    /// against the manifest; (b) for any provenance-headed excerpt
    /// fallback file, walks each non-header line looking for substring
    /// presence in one of the named upstream files.
    #[arg(long)]
    pub upstream_root: Option<PathBuf>,

    /// Rewrite `sha256` fields under each `vendored:` row in
    /// `parity/source-lock.yaml` from the live upstream checkout.
    /// Requires `--upstream-root`. Without this flag the verifier
    /// refuses to mutate source-lock.
    #[arg(long, default_value_t = false)]
    pub refresh_sha256: bool,

    /// Fetch each `vendored:` row from
    /// `https://raw.githubusercontent.com/<owner>/<repo>/<commit>/<upstream-path>`
    /// (parsed from the row's `remote:` field) and compare SHA-256
    /// against the manifest entry. Requires network access. Verifies the
    /// manifest is byte-identical to GitHub's canonical content at the
    /// pinned commit, without depending on any local upstream clone.
    #[arg(long, default_value_t = false)]
    pub upstream_github: bool,

    /// Emit JSON-formatted findings to stdout in addition to the
    /// tabular report. Used by CI to preserve structured artifacts.
    #[arg(long, default_value_t = false)]
    pub json: bool,

    /// Verification mode. `strict` (default) exits non-zero on any
    /// FAIL. `advisory` always exits zero (CI-skip mode for partial
    /// rollouts and contributor pre-flight).
    #[arg(long, default_value = "strict")]
    pub mode: Mode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Mode {
    Strict,
    Advisory,
}

/// Outcome of verifying a single `.sol` file.
#[derive(Debug, Clone)]
struct FileVerdict {
    path: PathBuf,
    pattern: Pattern,
    status: Status,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pattern {
    /// Byte-identical mirror; canonical SHA-256 lives in the `vendored:`
    /// array of the file's repository entry in `parity/source-lock.yaml`.
    Mirror,
    /// Provenance-headed excerpt; the `// Provenance` block lists every
    /// upstream source file the excerpt folds. Escape-hatch fallback;
    /// no currently-shipped file uses this path.
    Excerpt,
    /// Neither classification was detected.
    Unclassified,
}

impl Pattern {
    fn as_str(self) -> &'static str {
        match self {
            Self::Mirror => "byte-identical mirror",
            Self::Excerpt => "provenance-headed excerpt",
            Self::Unclassified => "unclassified",
        }
    }
}

#[derive(Debug, Clone)]
enum Status {
    Pass(String),
    Fail(String),
    Warn(String),
}

/// Entry point for the subcommand.
pub fn run(args: &VerifySolProvenanceArgs) -> Result<()> {
    if args.refresh_sha256 && args.upstream_root.is_none() {
        bail!("--refresh-sha256 requires --upstream-root");
    }

    let lock = load_source_lock(&args.source_lock)?;
    let vendored_index = build_vendored_index(&lock)?;
    let files = walk_sol_files(&args.abi_root)?;
    if files.is_empty() {
        println!(
            "verify-sol-provenance: no .sol files under {}",
            args.abi_root.display()
        );
        return Ok(());
    }

    // Refresh pass: when --refresh-sha256 is set, the verifier writes
    // updated SHAs back to source-lock.yaml at the end. The mutation
    // map is built up while iterating files.
    let mut refreshed: BTreeMap<(String, String), String> = BTreeMap::new();

    let mut verdicts = Vec::with_capacity(files.len());
    for path in files {
        let rel = pathdiff::diff_paths(&path, &args.abi_root)
            .unwrap_or_else(|| path.clone());
        let rel_str = rel.to_string_lossy().replace('\\', "/");

        let verdict = verify_one(
            &path,
            &rel_str,
            &vendored_index,
            &lock,
            args,
            &mut refreshed,
        )
        .unwrap_or_else(|err| FileVerdict {
            path: path.clone(),
            pattern: Pattern::Unclassified,
            status: Status::Fail(format!("verifier error: {err:#}")),
        });
        verdicts.push(verdict);
    }

    if args.refresh_sha256 && !refreshed.is_empty() {
        rewrite_source_lock_with_refreshed_shas(&args.source_lock, &refreshed)?;
    }

    print_report(&verdicts);

    if args.json {
        emit_json_findings(&verdicts)?;
    }

    let failures: Vec<&FileVerdict> = verdicts
        .iter()
        .filter(|v| matches!(v.status, Status::Fail(_)))
        .collect();

    if !failures.is_empty() && matches!(args.mode, Mode::Strict) {
        bail!(
            "verify-sol-provenance: {} of {} file(s) failed",
            failures.len(),
            verdicts.len()
        );
    }
    if failures.is_empty() {
        println!(
            "verify-sol-provenance: {} file(s) PASS",
            verdicts.len()
        );
    } else {
        println!(
            "verify-sol-provenance (advisory): {} of {} file(s) failed",
            failures.len(),
            verdicts.len()
        );
    }
    Ok(())
}

/// Minimal source-lock view consumed by the verifier.
#[derive(Debug, Deserialize)]
struct SourceLock {
    repositories: Vec<RepoEntry>,
}

#[derive(Debug, Deserialize)]
struct RepoEntry {
    id: String,
    #[serde(default)]
    commit: String,
    #[serde(default)]
    remote: String,
    #[serde(default)]
    vendored: Vec<VendoredEntry>,
}

#[derive(Debug, Deserialize, Clone)]
struct VendoredEntry {
    /// Path under `crates/contracts/abi/`. Forward slashes only.
    #[serde(rename = "local")]
    local_path: String,
    /// Upstream path under the repository root. Forward slashes only.
    upstream: String,
    /// SHA-256 of the upstream file at the repository's pinned commit.
    sha256: String,
}

/// Resolved view of a single vendored row, keyed by local relative path.
#[derive(Debug, Clone)]
struct ResolvedEntry {
    repo_id: String,
    commit: String,
    remote: String,
    upstream: String,
    sha256: String,
}

/// Build a `local_relative_path → ResolvedEntry` map from the source-lock
/// manifest. Forward-slash normalised so Windows checkouts hit the same
/// keys as Unix.
fn build_vendored_index(lock: &SourceLock) -> Result<BTreeMap<String, ResolvedEntry>> {
    let mut idx: BTreeMap<String, ResolvedEntry> = BTreeMap::new();
    for repo in &lock.repositories {
        for v in &repo.vendored {
            let key = v.local_path.replace('\\', "/");
            if let Some(prev) = idx.get(&key) {
                bail!(
                    "vendored entry `{}` duplicated under repos `{}` and `{}`",
                    key,
                    prev.repo_id,
                    repo.id
                );
            }
            if !is_lowercase_hex64(&v.sha256) {
                bail!(
                    "vendored entry `{}` has malformed sha256 `{}`",
                    key,
                    v.sha256
                );
            }
            idx.insert(
                key,
                ResolvedEntry {
                    repo_id: repo.id.clone(),
                    commit: repo.commit.clone(),
                    remote: repo.remote.clone(),
                    upstream: v.upstream.clone(),
                    sha256: v.sha256.clone(),
                },
            );
        }
    }
    Ok(idx)
}

fn load_source_lock(path: &Path) -> Result<SourceLock> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read source lock {}", path.display()))?;
    serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse source lock {}", path.display()))
}

/// Recursively collects every `*.sol` file under `root`, sorted.
fn walk_sol_files(root: &Path) -> Result<Vec<PathBuf>> {
    let mut acc = Vec::new();
    if !root.exists() {
        bail!("abi root does not exist: {}", root.display());
    }
    walk_dir(root, &mut acc)?;
    acc.sort();
    Ok(acc)
}

fn walk_dir(dir: &Path, acc: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(dir)
        .with_context(|| format!("failed to read directory {}", dir.display()))?;
    for entry in entries {
        let entry = entry
            .with_context(|| format!("failed to read entry under {}", dir.display()))?;
        let file_type = entry
            .file_type()
            .with_context(|| format!("failed to read file type for {}", entry.path().display()))?;
        if file_type.is_symlink() {
            bail!(
                "symlink encountered at {}: vendored .sol must be regular files",
                entry.path().display()
            );
        }
        let path = entry.path();
        if file_type.is_dir() {
            walk_dir(&path, acc)?;
        } else if file_type.is_file()
            && path.extension().and_then(|e| e.to_str()) == Some("sol")
        {
            acc.push(path);
        }
    }
    Ok(())
}

/// Verifies a single `.sol` file.
fn verify_one(
    path: &Path,
    rel: &str,
    vendored_index: &BTreeMap<String, ResolvedEntry>,
    lock: &SourceLock,
    args: &VerifySolProvenanceArgs,
    refreshed: &mut BTreeMap<(String, String), String>,
) -> Result<FileVerdict> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read {} as UTF-8 text", path.display()))?;

    let header = parse_provenance_header(&raw);
    let manifest_entry = vendored_index.get(rel);

    let pattern = if manifest_entry.is_some() {
        Pattern::Mirror
    } else if header.is_some() {
        Pattern::Excerpt
    } else {
        Pattern::Unclassified
    };

    let status = match pattern {
        Pattern::Mirror => verify_mirror(
            path,
            &raw,
            rel,
            manifest_entry.expect("classified A iff entry present"),
            args,
            refreshed,
        )?,
        Pattern::Excerpt => verify_excerpt(
            path,
            &raw,
            header.as_ref().expect("classified B iff header present"),
            lock,
            args,
        )?,
        Pattern::Unclassified => Status::Fail(
            "neither a `vendored:` SHA-256 row nor a `// Provenance` header was found \
             (add a `vendored:` row \
             under the matching repository in parity/source-lock.yaml, or add a \
             `// Provenance` header listing the upstream sources this excerpt folds)"
                .to_string(),
        ),
    };

    Ok(FileVerdict {
        path: path.to_path_buf(),
        pattern,
        status,
    })
}

/// Byte-identical mirror verification: SHA-256 the committed bytes,
/// compare against the manifest entry, and optionally against the live
/// upstream file (via local git checkout and/or GitHub raw content).
fn verify_mirror(
    _path: &Path,
    raw: &str,
    _rel: &str,
    entry: &ResolvedEntry,
    args: &VerifySolProvenanceArgs,
    refreshed: &mut BTreeMap<(String, String), String>,
) -> Result<Status> {
    let on_disk_sha = sha256_hex(raw.as_bytes());

    if on_disk_sha != entry.sha256 {
        return Ok(Status::Fail(format!(
            "SHA-256 mismatch vs manifest at {}@{}: on-disk={on_disk_sha}, manifest={}",
            entry.repo_id, entry.commit, entry.sha256
        )));
    }

    let mut attestations: Vec<String> = vec!["manifest".into()];

    // GitHub-raw cross-check (network-only, no local clone required).
    if args.upstream_github {
        match fetch_github_raw(entry) {
            Ok(bytes) => {
                let github_sha = sha256_hex(&bytes);
                if github_sha != entry.sha256 {
                    return Ok(Status::Fail(format!(
                        "GitHub raw SHA does not match manifest at {}@{}: manifest={}, github={github_sha}",
                        entry.repo_id, entry.commit, entry.sha256
                    )));
                }
                attestations.push("GitHub raw".into());
            }
            Err(err) => {
                return Ok(Status::Fail(format!(
                    "GitHub raw fetch failed for {}@{}:{}: {err}",
                    entry.repo_id, entry.commit, entry.upstream
                )));
            }
        }
    }

    // Local-checkout cross-check via `git show <commit>:<path>`.
    if let Some(upstream_root) = args.upstream_root.as_deref() {
        let upstream_repo_dir = upstream_root.join(&entry.repo_id);
        if !upstream_repo_dir.join(".git").exists() {
            return Ok(Status::Fail(format!(
                "upstream repository directory missing at {} (expected git checkout for `{}`@{})",
                upstream_repo_dir.display(),
                entry.repo_id,
                entry.commit
            )));
        }
        let upstream_bytes = match std::process::Command::new("git")
            .arg("-C")
            .arg(&upstream_repo_dir)
            .arg("show")
            .arg(format!("{}:{}", entry.commit, entry.upstream))
            .output()
        {
            Ok(out) if out.status.success() => out.stdout,
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Ok(Status::Fail(format!(
                    "git show {}:{} failed in {}: {} (run `git -C {} fetch origin {}` to bring the pinned commit into the checkout)",
                    entry.commit,
                    entry.upstream,
                    upstream_repo_dir.display(),
                    stderr.trim(),
                    upstream_repo_dir.display(),
                    entry.commit
                )));
            }
            Err(err) => {
                return Ok(Status::Fail(format!(
                    "failed to spawn `git` for upstream read at {}: {err}",
                    upstream_repo_dir.display()
                )));
            }
        };
        let upstream_sha = sha256_hex(&upstream_bytes);

        if args.refresh_sha256 {
            // Bootstrap / refresh: require that on-disk == upstream so
            // the manifest never carries a hash that doesn't trace to
            // upstream bytes at the pinned commit.
            if upstream_sha != on_disk_sha {
                return Ok(Status::Fail(format!(
                    "cannot refresh manifest: on-disk={on_disk_sha} != upstream={upstream_sha}; \
                     either re-vendor the file from upstream or convert to the \
                     provenance-headed excerpt fallback"
                )));
            }
            if upstream_sha != entry.sha256 {
                refreshed.insert(
                    (entry.repo_id.clone(), entry.upstream.clone()),
                    upstream_sha.clone(),
                );
                return Ok(Status::Pass(format!(
                    "refreshed manifest SHA from upstream ({upstream_sha})"
                )));
            }
        }

        if upstream_sha != entry.sha256 {
            return Ok(Status::Fail(format!(
                "manifest SHA does not match local upstream git-show at {}@{}: manifest={}, upstream={upstream_sha}",
                entry.repo_id, entry.commit, entry.sha256
            )));
    }
    if on_disk_sha != entry.sha256 {
        return Ok(Status::Fail(format!(
            "on-disk SHA does not match manifest at {}@{}: on-disk={on_disk_sha}, manifest={}",
            entry.repo_id, entry.commit, entry.sha256
        )));
        }
        attestations.push("local git-show".into());
    }

    Ok(Status::Pass(format!(
        "SHA match at {}@{} via {} ({on_disk_sha})",
        entry.repo_id,
        entry.commit,
        attestations.join(" + ")
    )))
}

/// Parse `<owner>/<repo>` from a github remote URL.  Returns Err for
/// non-github remotes.
fn parse_github_owner_repo(remote: &str) -> Result<(String, String)> {
    let trimmed = remote.trim().trim_end_matches('/');
    let stripped = trimmed.trim_end_matches(".git");
    // Accept any github.com URL (https, ssh, scp-like).
    let body = stripped
        .strip_prefix("https://github.com/")
        .or_else(|| stripped.strip_prefix("http://github.com/"))
        .or_else(|| stripped.strip_prefix("git@github.com:"))
        .ok_or_else(|| anyhow::anyhow!("remote `{remote}` is not a github.com URL"))?;
    let mut parts = body.splitn(2, '/');
    let owner = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("remote `{remote}` is missing the <owner> segment"))?;
    let repo = parts
        .next()
        .filter(|s| !s.is_empty())
        .ok_or_else(|| anyhow::anyhow!("remote `{remote}` is missing the <repo> segment"))?;
    Ok((owner.to_string(), repo.to_string()))
}

/// Fetch the upstream bytes for a `vendored:` row from GitHub's raw
/// content API.  Trust posture: GitHub TLS + content delivery + the
/// pinned commit captured in `parity/source-lock.yaml`.
fn fetch_github_raw(entry: &ResolvedEntry) -> Result<Vec<u8>> {
    let (owner, repo) = parse_github_owner_repo(&entry.remote)?;
    let url = format!(
        "https://raw.githubusercontent.com/{owner}/{repo}/{commit}/{path}",
        commit = entry.commit,
        path = entry.upstream
    );
    let client = reqwest::blocking::Client::builder()
        .user_agent("cow-rs parity-maintainer verify-sol-provenance")
        .timeout(std::time::Duration::from_secs(30))
        .https_only(true)
        .build()
        .context("failed to build reqwest client")?;
    let response = client
        .get(&url)
        .send()
        .with_context(|| format!("failed to send GET {url}"))?;
    let status = response.status();
    if !status.is_success() {
        anyhow::bail!("GET {url} returned HTTP {status}");
    }
    let bytes = response
        .bytes()
        .with_context(|| format!("failed to read body of GET {url}"))?;
    Ok(bytes.to_vec())
}

/// Provenance header parsed from a provenance-headed excerpt fallback file.
#[derive(Debug, Clone)]
struct ProvenanceHeader {
    repo_remote: String,
    pinned_commit: Option<String>,
    source_files: Vec<String>,
}

/// Parses the `// Provenance` block at the top of a `.sol` file. Returns
/// `Some(header)` when the block is structurally well-formed; `None`
/// when no block is detected.
fn parse_provenance_header(raw: &str) -> Option<ProvenanceHeader> {
    let lines = raw.lines();
    let mut in_block = false;
    let mut repo_remote: Option<String> = None;
    let mut pinned_commit: Option<String> = None;
    let mut source_files: Vec<String> = Vec::new();
    let mut listing_sources = false;

    for line in lines.take(80) {
        let trimmed = line.trim_start();
        if !in_block {
            if trimmed == "// Provenance" {
                in_block = true;
            }
            continue;
        }
        if !trimmed.starts_with("//") {
            // First non-comment line ends the header block.
            break;
        }
        let payload = trimmed.trim_start_matches("//").trim();
        if let Some(remote) = payload.strip_prefix("Upstream repository:") {
            repo_remote = Some(remote.trim().to_string());
            listing_sources = false;
        } else if let Some(commit) = payload.strip_prefix("Pinned commit:") {
            pinned_commit = Some(commit.trim().to_string());
            listing_sources = false;
        } else if payload.starts_with("Source files folded into this excerpt:") {
            listing_sources = true;
        } else if listing_sources && payload.starts_with("* ") {
            // Capture the path portion before the dash separator.
            let rest = &payload[2..];
            let path = rest
                .split('—')
                .next()
                .unwrap_or(rest)
                .split(" - ")
                .next()
                .unwrap_or(rest)
                .trim();
            if !path.is_empty() {
                source_files.push(path.to_string());
            }
        } else if listing_sources && !payload.starts_with("*") && !payload.is_empty() {
            // Continuation lines are description-only; ignore.
        }
    }

    if !in_block {
        return None;
    }
    Some(ProvenanceHeader {
        repo_remote: repo_remote.unwrap_or_default(),
        pinned_commit,
        source_files,
    })
}

/// Provenance-headed excerpt fallback verification.
fn verify_excerpt(
    path: &Path,
    raw: &str,
    header: &ProvenanceHeader,
    lock: &SourceLock,
    args: &VerifySolProvenanceArgs,
) -> Result<Status> {
    if header.repo_remote.is_empty() {
        return Ok(Status::Fail(
            "Provenance header lists no `Upstream repository:`".to_string(),
        ));
    }
    if header.source_files.is_empty() {
        return Ok(Status::Fail(
            "Provenance header lists no `Source files folded into this excerpt`".to_string(),
        ));
    }

    // Find the matching source-lock repo by remote URL prefix.
    let normalized_remote = header
        .repo_remote
        .trim_end_matches(".git")
        .trim_end_matches('/');
    let repo = lock.repositories.iter().find(|r| {
        // Source-lock repositories carry full `https://github.com/<owner>/<repo>.git`
        // remotes; the header carries the same URL without `.git`.
        let normalized_lock = r
            .id
            .as_str();
        !normalized_lock.is_empty()
            && (normalized_remote.contains(normalized_lock)
                || normalized_remote == normalized_lock)
    });

    if let (Some(repo), Some(header_commit)) = (repo, header.pinned_commit.as_ref())
        && !repo.commit.is_empty()
        && repo.commit != "standard"
        && &repo.commit != header_commit
    {
        return Ok(Status::Fail(format!(
            "Provenance header `Pinned commit:` ({}) does not match \
             source-lock pin ({}) for repo `{}`",
            header_commit, repo.commit, repo.id
        )));
    }

    if header.pinned_commit.is_none() {
        // Advisory only: existing headers pre-dating the doctrine may
        // omit Pinned commit.
        if args.upstream_root.is_none() {
            return Ok(Status::Warn(format!(
                "Provenance header is missing `Pinned commit:` (path={})",
                path.display()
            )));
        }
    }

    let Some(upstream_root) = args.upstream_root.as_deref() else {
        // Without an upstream checkout we accept the structural check.
        return Ok(Status::Pass(format!(
            "header parses ({} source(s) listed)",
            header.source_files.len()
        )));
    };

    let Some(repo) = repo else {
        return Ok(Status::Fail(format!(
            "Provenance header names `{}` but no matching repository was found in source-lock",
            header.repo_remote
        )));
    };

    // Collect upstream file contents.
    let mut upstreams: Vec<String> = Vec::with_capacity(header.source_files.len());
    for src in &header.source_files {
        let upstream_file = upstream_root.join(&repo.id).join(src);
        if !upstream_file.exists() {
            return Ok(Status::Fail(format!(
                "Provenance header names `{}` but {} does not exist",
                src,
                upstream_file.display()
            )));
        }
        let text = fs::read_to_string(&upstream_file)
            .with_context(|| format!("failed to read upstream file {}", upstream_file.display()))?;
        upstreams.push(text);
    }

    // Walk every non-header line of the excerpt looking for substring
    // presence in at least one upstream.
    let mut missing: Vec<(usize, String)> = Vec::new();
    let mut in_header_block = true;
    for (lineno, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if in_header_block {
            // The header block ends at the first non-comment, non-blank
            // line. After that we walk the body.
            if !trimmed.is_empty() && !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
                in_header_block = false;
            } else {
                continue;
            }
        }
        if is_boilerplate(trimmed) {
            continue;
        }
        let normalized = normalize_whitespace(trimmed);
        if normalized.is_empty() {
            continue;
        }
        let found = upstreams.iter().any(|u| {
            normalize_whitespace(u)
                .contains(&normalized)
        });
        if !found {
            missing.push((lineno + 1, trimmed.to_string()));
        }
    }

    if !missing.is_empty() {
        let mut detail = format!(
            "{} excerpt line(s) not present in any named upstream:\n",
            missing.len()
        );
        for (lineno, content) in missing.iter().take(5) {
            detail.push_str(&format!("    L{lineno}: {content}\n"));
        }
        if missing.len() > 5 {
            detail.push_str(&format!("    ... and {} more\n", missing.len() - 5));
        }
        return Ok(Status::Fail(detail.trim_end().to_string()));
    }

    Ok(Status::Pass(format!(
        "all excerpt lines found across {} upstream source(s)",
        header.source_files.len()
    )))
}

/// Lines that are boilerplate (license, pragma, blank, brace-only, etc.)
/// and excluded from the upstream-presence check.
fn is_boilerplate(s: &str) -> bool {
    if s.is_empty() {
        return true;
    }
    if s.starts_with("//") || s.starts_with("/*") || s.starts_with("*/") || s.starts_with("*") {
        return true;
    }
    if s.starts_with("pragma ") {
        return true;
    }
    // Brace-only or punctuation-only lines.
    if s.chars().all(|c| c.is_ascii_whitespace() || matches!(c, '{' | '}' | ';' | ',')) {
        return true;
    }
    false
}

fn normalize_whitespace(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[must_use]
fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|b| format!("{b:02x}")).collect()
}

#[must_use]
fn is_lowercase_hex64(s: &str) -> bool {
    s.len() == 64 && s.chars().all(|c| matches!(c, '0'..='9' | 'a'..='f'))
}

fn print_report(verdicts: &[FileVerdict]) {
    // Determine the widest path so the report aligns. Cap at 90 to
    // prevent runaway column width on deeply-nested paths.
    let path_width = verdicts
        .iter()
        .map(|v| v.path.to_string_lossy().len())
        .max()
        .unwrap_or(40)
        .min(90);

    for v in verdicts {
        let path = v.path.to_string_lossy();
        let pattern = v.pattern.as_str();
        let (verdict, message) = match &v.status {
            Status::Pass(msg) => ("PASS", msg.as_str()),
            Status::Fail(msg) => ("FAIL", msg.as_str()),
            Status::Warn(msg) => ("WARN", msg.as_str()),
        };
        println!(
            "{verdict:5} {path:<path_width$}  [{pattern}]  {message}",
            path_width = path_width
        );
    }
}

fn emit_json_findings(verdicts: &[FileVerdict]) -> Result<()> {
    let rows: Vec<serde_json::Value> = verdicts
        .iter()
        .map(|v| {
            let (verdict, message) = match &v.status {
                Status::Pass(msg) => ("pass", msg.as_str()),
                Status::Fail(msg) => ("fail", msg.as_str()),
                Status::Warn(msg) => ("warn", msg.as_str()),
            };
            serde_json::json!({
                "path": v.path.to_string_lossy(),
                "pattern": v.pattern.as_str(),
                "verdict": verdict,
                "message": message,
            })
        })
        .collect();
    let report = serde_json::json!({
        "schema_version": 1,
        "findings": rows,
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

/// In-place update of `parity/source-lock.yaml` for refreshed SHAs.
/// Uses line-based string substitution to preserve formatting (comments,
/// blank lines, key ordering) that a serde_yaml round-trip would lose.
fn rewrite_source_lock_with_refreshed_shas(
    path: &Path,
    refreshed: &BTreeMap<(String, String), String>,
) -> Result<()> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to re-read source lock {}", path.display()))?;
    let mut out = String::with_capacity(raw.len());
    let mut current_repo: Option<String> = None;
    let mut current_upstream: Option<String> = None;
    let mut applied = 0;

    for line in raw.lines() {
        // Track the active repository id while iterating.
        if let Some(rest) = line.strip_prefix("- id: ") {
            current_repo = Some(rest.trim().to_string());
            current_upstream = None;
        } else if let Some(rest) = line.trim_start().strip_prefix("- local: ") {
            // A `vendored:` entry starts.
            current_upstream = None;
            let _ = rest;
        } else if let Some(rest) = line.trim_start().strip_prefix("upstream: ") {
            current_upstream = Some(rest.trim().to_string());
        }

        if let (Some(repo), Some(upstream)) = (&current_repo, &current_upstream)
            && let Some(new_sha) = refreshed.get(&(repo.clone(), upstream.clone()))
            && line.trim_start().starts_with("sha256: ")
        {
            // Preserve the indent before `sha256:`.
            let indent_len = line.len() - line.trim_start().len();
            let indent = &line[..indent_len];
            out.push_str(&format!("{indent}sha256: {new_sha}\n"));
            applied += 1;
            current_upstream = None;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }

    fs::write(path, out)
        .with_context(|| format!("failed to write refreshed source lock {}", path.display()))?;
    if applied > 0 {
        eprintln!(
            "verify-sol-provenance: refreshed {applied} SHA(s) in {}",
            path.display()
        );
    }
    Ok(())
}

mod pathdiff {
    use std::path::{Component, Path, PathBuf};

    pub(super) fn diff_paths(path: &Path, base: &Path) -> Option<PathBuf> {
        let path = path.canonicalize().ok()?;
        let base = base.canonicalize().ok()?;
        let mut ita = path.components();
        let mut itb = base.components();
        let mut comps: Vec<Component<'_>> = Vec::new();
        loop {
            match (ita.next(), itb.next()) {
                (None, None) => break,
                (Some(a), None) => {
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
                (None, _) => comps.push(Component::ParentDir),
                (Some(a), Some(b)) if comps.is_empty() && a == b => continue,
                (Some(a), Some(_)) => {
                    comps.push(Component::ParentDir);
                    for _ in itb {
                        comps.push(Component::ParentDir);
                    }
                    comps.push(a);
                    comps.extend(ita.by_ref());
                    break;
                }
            }
        }
        Some(comps.iter().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn write_file(dir: &Path, name: &str, contents: &str) -> PathBuf {
        let path = dir.join(name);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        let mut f = fs::File::create(&path).unwrap();
        f.write_all(contents.as_bytes()).unwrap();
        path
    }

    fn make_lock(yaml: &str) -> SourceLock {
        serde_yaml::from_str(yaml).expect("lock parses")
    }

    fn sample_args(source_lock: PathBuf, abi_root: PathBuf) -> VerifySolProvenanceArgs {
        VerifySolProvenanceArgs {
            source_lock,
            abi_root,
            upstream_root: None,
            refresh_sha256: false,
            upstream_github: false,
            json: false,
            mode: Mode::Strict,
        }
    }

    #[test]
    fn parse_github_owner_repo_accepts_https_with_dot_git() {
        let (owner, repo) =
            parse_github_owner_repo("https://github.com/cowprotocol/contracts.git").unwrap();
        assert_eq!(owner, "cowprotocol");
        assert_eq!(repo, "contracts");
    }

    #[test]
    fn parse_github_owner_repo_accepts_https_without_dot_git() {
        let (owner, repo) =
            parse_github_owner_repo("https://github.com/cowdao-grants/cow-shed").unwrap();
        assert_eq!(owner, "cowdao-grants");
        assert_eq!(repo, "cow-shed");
    }

    #[test]
    fn parse_github_owner_repo_rejects_non_github() {
        assert!(parse_github_owner_repo("https://gitlab.com/foo/bar").is_err());
        assert!(parse_github_owner_repo("https://eips.ethereum.org/EIPS/eip-20").is_err());
    }

    #[test]
    fn parse_github_owner_repo_rejects_malformed() {
        assert!(parse_github_owner_repo("https://github.com/").is_err());
        assert!(parse_github_owner_repo("https://github.com/owner-only").is_err());
    }

    #[test]
    fn lowercase_hex64_validator() {
        assert!(is_lowercase_hex64(&"a".repeat(64)));
        assert!(is_lowercase_hex64(&"0".repeat(64)));
        assert!(!is_lowercase_hex64(&"A".repeat(64)));
        assert!(!is_lowercase_hex64(&"a".repeat(63)));
        assert!(!is_lowercase_hex64(&"a".repeat(65)));
        assert!(!is_lowercase_hex64(&"g".repeat(64)));
    }

    #[test]
    fn normalize_whitespace_collapses_runs() {
        assert_eq!(normalize_whitespace("a   b\t\tc"), "a b c");
        assert_eq!(normalize_whitespace("  spaced  "), "spaced");
    }

    #[test]
    fn refresh_sha256_without_upstream_root_is_rejected() {
        let dir = TempDir::new().unwrap();
        let lock_path = dir.path().join("source-lock.yaml");
        fs::write(&lock_path, "repositories: []\n").unwrap();
        let mut args = sample_args(lock_path, dir.path().to_path_buf());
        args.refresh_sha256 = true;
        let err = run(&args).unwrap_err();
        assert!(err.to_string().contains("--refresh-sha256"));
    }

    #[test]
    fn pattern_a_pass_when_manifest_matches_on_disk() {
        let dir = TempDir::new().unwrap();
        let abi = dir.path().join("abi");
        fs::create_dir_all(&abi).unwrap();
        let body = "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\n\ncontract X {}\n";
        let _sol = write_file(&abi, "fam/X.sol", body);
        let body_sha = sha256_hex(body.as_bytes());

        let lock_yaml = format!(
            r"
repositories:
- id: fam
  commit: 1111111111111111111111111111111111111111
  vendored:
  - local: fam/X.sol
    upstream: src/X.sol
    sha256: {body_sha}
",
        );
        let lock_path = dir.path().join("lock.yaml");
        fs::write(&lock_path, &lock_yaml).unwrap();

        let args = sample_args(lock_path, abi.clone());
        let lock = make_lock(&lock_yaml);
        let idx = build_vendored_index(&lock).unwrap();
        let mut refreshed = BTreeMap::new();
        let verdict = verify_one(
            &abi.join("fam/X.sol"),
            "fam/X.sol",
            &idx,
            &lock,
            &args,
            &mut refreshed,
        )
        .unwrap();
        assert_eq!(verdict.pattern, Pattern::Mirror);
        assert!(matches!(verdict.status, Status::Pass(_)));
    }

    #[test]
    fn pattern_a_fail_when_manifest_disagrees_with_on_disk() {
        let dir = TempDir::new().unwrap();
        let abi = dir.path().join("abi");
        fs::create_dir_all(&abi).unwrap();
        let body = "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\n\ncontract X {}\n";
        let _sol = write_file(&abi, "fam/X.sol", body);

        let wrong_sha = "f".repeat(64);
        let lock_yaml = format!(
            r"
repositories:
- id: fam
  commit: 1111111111111111111111111111111111111111
  vendored:
  - local: fam/X.sol
    upstream: src/X.sol
    sha256: {wrong_sha}
",
        );
        let lock_path = dir.path().join("lock.yaml");
        fs::write(&lock_path, &lock_yaml).unwrap();

        let args = sample_args(lock_path, abi.clone());
        let lock = make_lock(&lock_yaml);
        let idx = build_vendored_index(&lock).unwrap();
        let mut refreshed = BTreeMap::new();
        let verdict = verify_one(
            &abi.join("fam/X.sol"),
            "fam/X.sol",
            &idx,
            &lock,
            &args,
            &mut refreshed,
        )
        .unwrap();
        assert_eq!(verdict.pattern, Pattern::Mirror);
        assert!(matches!(verdict.status, Status::Fail(_)));
    }

    #[test]
    fn pattern_b_pass_when_every_excerpt_line_is_in_one_named_upstream() {
        let dir = TempDir::new().unwrap();
        let abi = dir.path().join("abi");
        let upstream = dir.path().join("upstream").join("fam");
        fs::create_dir_all(&abi).unwrap();
        fs::create_dir_all(&upstream).unwrap();

        // The excerpt's body lines must appear in the upstream file.
        let upstream_body = "interface Iface {\n    function foo() external view returns (bool);\n}\n";
        write_file(&upstream, "src/Iface.sol", upstream_body);

        let excerpt = "// SPDX-License-Identifier: MIT\npragma solidity ^0.8.0;\n\n\
             // Provenance\n\
             // ----------\n\
             // Upstream repository: https://github.com/example/fam\n\
             // Source files folded into this excerpt:\n\
             //   * src/Iface.sol — the foo() view\n\n\
             interface Iface {\n\
                 function foo() external view returns (bool);\n\
             }\n";
        write_file(&abi, "fam/Iface.sol", excerpt);

        let lock_yaml = "
repositories:
- id: fam
  commit: 2222222222222222222222222222222222222222
";
        let lock_path = dir.path().join("lock.yaml");
        fs::write(&lock_path, lock_yaml).unwrap();

        let mut args = sample_args(lock_path, abi.clone());
        args.upstream_root = Some(dir.path().join("upstream"));
        let lock = make_lock(lock_yaml);
        let idx = build_vendored_index(&lock).unwrap();
        let mut refreshed = BTreeMap::new();
        let verdict = verify_one(
            &abi.join("fam/Iface.sol"),
            "fam/Iface.sol",
            &idx,
            &lock,
            &args,
            &mut refreshed,
        )
        .unwrap();
        assert_eq!(verdict.pattern, Pattern::Excerpt);
        assert!(
            matches!(verdict.status, Status::Pass(_)),
            "expected Pass, got {:?}",
            verdict.status
        );
    }

    #[test]
    fn pattern_b_fail_when_an_excerpt_line_is_in_no_named_upstream() {
        let dir = TempDir::new().unwrap();
        let abi = dir.path().join("abi");
        let upstream = dir.path().join("upstream").join("fam");
        fs::create_dir_all(&abi).unwrap();
        fs::create_dir_all(&upstream).unwrap();

        let upstream_body = "interface Iface {\n    function foo() external view returns (bool);\n}\n";
        write_file(&upstream, "src/Iface.sol", upstream_body);

        let excerpt = "// SPDX-License-Identifier: MIT\n\
             pragma solidity ^0.8.0;\n\n\
             // Provenance\n\
             // ----------\n\
             // Upstream repository: https://github.com/example/fam\n\
             // Source files folded into this excerpt:\n\
             //   * src/Iface.sol — the foo() view\n\n\
             interface Iface {\n\
                 function bar() external pure returns (uint256);\n\
             }\n";
        write_file(&abi, "fam/Iface.sol", excerpt);

        let lock_yaml = "
repositories:
- id: fam
  commit: 2222222222222222222222222222222222222222
";
        let lock_path = dir.path().join("lock.yaml");
        fs::write(&lock_path, lock_yaml).unwrap();

        let mut args = sample_args(lock_path, abi.clone());
        args.upstream_root = Some(dir.path().join("upstream"));
        let lock = make_lock(lock_yaml);
        let idx = build_vendored_index(&lock).unwrap();
        let mut refreshed = BTreeMap::new();
        let verdict = verify_one(
            &abi.join("fam/Iface.sol"),
            "fam/Iface.sol",
            &idx,
            &lock,
            &args,
            &mut refreshed,
        )
        .unwrap();
        assert_eq!(verdict.pattern, Pattern::Excerpt);
        assert!(matches!(verdict.status, Status::Fail(_)));
    }

    #[test]
    fn unclassified_file_fails_explicitly() {
        let dir = TempDir::new().unwrap();
        let abi = dir.path().join("abi");
        fs::create_dir_all(&abi).unwrap();
        // No manifest entry, no Provenance header.
        write_file(&abi, "fam/Mystery.sol", "contract Mystery {}\n");

        let lock_yaml = "
repositories:
- id: fam
  commit: 3333333333333333333333333333333333333333
";
        let lock_path = dir.path().join("lock.yaml");
        fs::write(&lock_path, lock_yaml).unwrap();

        let args = sample_args(lock_path, abi.clone());
        let lock = make_lock(lock_yaml);
        let idx = build_vendored_index(&lock).unwrap();
        let mut refreshed = BTreeMap::new();
        let verdict = verify_one(
            &abi.join("fam/Mystery.sol"),
            "fam/Mystery.sol",
            &idx,
            &lock,
            &args,
            &mut refreshed,
        )
        .unwrap();
        assert_eq!(verdict.pattern, Pattern::Unclassified);
        assert!(matches!(verdict.status, Status::Fail(_)));
    }

    #[test]
    fn pinned_commit_mismatch_in_header_fails() {
        let dir = TempDir::new().unwrap();
        let abi = dir.path().join("abi");
        fs::create_dir_all(&abi).unwrap();

        let excerpt = "// SPDX-License-Identifier: MIT\n\
             pragma solidity ^0.8.0;\n\n\
             // Provenance\n\
             // ----------\n\
             // Upstream repository: https://github.com/example/fam\n\
             // Pinned commit: bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb\n\
             // Source files folded into this excerpt:\n\
             //   * src/Iface.sol — desc\n\n\
             interface X {}\n";
        write_file(&abi, "fam/X.sol", excerpt);

        let lock_yaml = "
repositories:
- id: fam
  commit: aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
";
        let lock_path = dir.path().join("lock.yaml");
        fs::write(&lock_path, lock_yaml).unwrap();

        let args = sample_args(lock_path, abi.clone());
        let lock = make_lock(lock_yaml);
        let idx = build_vendored_index(&lock).unwrap();
        let mut refreshed = BTreeMap::new();
        let verdict = verify_one(
            &abi.join("fam/X.sol"),
            "fam/X.sol",
            &idx,
            &lock,
            &args,
            &mut refreshed,
        )
        .unwrap();
        assert_eq!(verdict.pattern, Pattern::Excerpt);
        match &verdict.status {
            Status::Fail(msg) => assert!(msg.contains("does not match"), "msg was: {msg}"),
            other => panic!("expected Fail, got {other:?}"),
        }
    }

    #[test]
    fn duplicate_vendored_local_path_rejected() {
        let lock_yaml = "
repositories:
- id: fam-a
  commit: aaaa
  vendored:
  - local: shared/Common.sol
    upstream: src/Common.sol
    sha256: 0000000000000000000000000000000000000000000000000000000000000000
- id: fam-b
  commit: bbbb
  vendored:
  - local: shared/Common.sol
    upstream: src/Common.sol
    sha256: 1111111111111111111111111111111111111111111111111111111111111111
";
        let lock: SourceLock = serde_yaml::from_str(lock_yaml).unwrap();
        let err = build_vendored_index(&lock).unwrap_err();
        assert!(err.to_string().contains("duplicated"));
    }
}
