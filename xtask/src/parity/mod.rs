//! Upstream-parity maintenance: the source-lock model and validator, pinned
//! checkout synchronization, the vendored `OpenAPI` document, DTO coverage,
//! and the deployment-registry presence probe.
//!
//! The lock pins repositories; every other provenance fact derives from those
//! pins at check time. Fixtures self-describe through a `sources`/`standards`
//! header validated per-file by globbing `parity/fixtures/**/*.json`, so a
//! fixture cannot exist outside the contract.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

pub mod openapi_coverage;
pub mod registry_confirm;
pub mod sync;
pub mod vendor_openapi;

pub const DEFAULT_SOURCE_LOCK: &str = "parity/source-lock.yaml";

/// Path of the vendored services `OpenAPI` document inside the upstream
/// `services` checkout.
pub(crate) const SERVICES_OPENAPI_PATH: &str = "crates/orderbook/openapi.yml";

/// Stamp line prefix the vendoring command writes and the validator reads.
pub(crate) const VENDORED_STAMP_PREFIX: &str = "# Vendored from cowprotocol/services @ ";

/// Provenance lookalikes. Unknown top-level keys are payload by design, so a
/// provenance-shaped key the grammar does not know would be silently ignored
/// — leaving provenance-shaped prose that nothing validates (the pre-glob
/// hole that hid a stale commit in an unchecked `@source_ref` for months) —
/// and `source` is the near-miss typo of `sources` that would slip whenever a
/// fixture also carries `standards`. Not a compatibility shim: these keys are
/// rejected because they look validated and are not.
const PROVENANCE_LOOKALIKE_KEYS: &[&str] = &["source", "source_refs"];

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SourceLock {
    pub(crate) repositories: Vec<RepositoryEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RepositoryEntry {
    pub(crate) id: String,
    pub(crate) remote: String,
    pub(crate) commit: String,
    pub(crate) producer_paths: Vec<String>,
}

/// Provenance header every committed fixture carries. Unknown top-level keys
/// are the fixture payload (`cases`, `rows`, `examples`, `payload`, …) and
/// stay untyped here; the typed sub-models are strict so a misspelled header
/// key fails closed.
#[derive(Debug, Deserialize)]
struct FixtureHeader {
    surface: String,
    #[serde(default)]
    sources: BTreeMap<String, FixtureSource>,
    #[serde(default)]
    standards: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureSource {
    commit: String,
    refs: Vec<String>,
}

pub struct CliOptions {
    pub source_lock: PathBuf,
    /// Root holding one checkout per lock repository (`<root>/<id>`, the
    /// layout `parity sync` materializes). When set, every repository row is
    /// deep-validated against its checkout and the vendored `OpenAPI` body is
    /// compared against the blob at the pinned services commit.
    pub upstream_root: Option<PathBuf>,
}

/// Validates the committed source lock and the fixture corpus by form.
///
/// Offline (always): unique repository ids, each repository row's shape,
/// every fixture's provenance header against the pins, and the vendored
/// `OpenAPI` stamp against the services pin. With `upstream_root`: each
/// repository's checkout (remote, `HEAD` at the pin, clean producer paths)
/// and the vendored `OpenAPI` body against the pinned blob. The lock content
/// is the single source of truth; the validator never matches it against
/// hardcoded rows.
pub fn validate(options: &CliOptions) -> Result<()> {
    let lock = validate_lock_form(&options.source_lock)?;
    let fixture_count = validate_fixtures(&options.source_lock, &lock)?;
    validate_vendored_openapi_stamp(&options.source_lock, &lock)?;

    if let Some(root) = &options.upstream_root {
        for repo in &lock.repositories {
            let checkout = root.join(&repo.id);
            validate_repository_root(repo, &checkout).with_context(|| {
                format!(
                    "deep validation failed for {} under {} (run `cargo xtask parity sync`)",
                    repo.id,
                    root.display()
                )
            })?;
        }
        validate_vendored_openapi_body(&options.source_lock, &lock, root)?;
    }

    println!(
        "validated {} repositories and {fixture_count} fixture files",
        lock.repositories.len(),
    );
    Ok(())
}

/// Parses the lock and validates the repository rows only — the shared core
/// of [`validate`] and the post-rewrite check in `sync --update`, which must
/// not demand the fixture ratchet already be refreshed (refreshing is the
/// step the drift table tells the maintainer to do next).
pub(crate) fn validate_lock_form(source_lock: &Path) -> Result<SourceLock> {
    let lock = load_source_lock(source_lock)?;

    let mut ids = BTreeSet::new();
    for repo in &lock.repositories {
        if !ids.insert(repo.id.as_str()) {
            bail!("duplicate repository id in source lock: {}", repo.id);
        }
        validate_repository_form(repo)?;
    }
    Ok(lock)
}

/// Validates one repository row's shape: a GitHub `.git` remote, a
/// 40-character lowercase hex commit, and a non-empty list of relative
/// producer paths with no traversal or duplicates.
fn validate_repository_form(repo: &RepositoryEntry) -> Result<()> {
    if repo.id.trim().is_empty() {
        bail!("repository id must be non-empty");
    }
    if !(repo.remote.starts_with("https://github.com/")
        && repo.remote.strip_suffix(".git").is_some())
    {
        bail!(
            "repository {} remote must be an https github .git URL: {}",
            repo.id,
            repo.remote
        );
    }
    if repo.commit.len() != 40
        || !repo
            .commit
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        bail!(
            "repository {} commit must be a 40-character lowercase hex sha",
            repo.id
        );
    }
    if repo.producer_paths.is_empty() {
        bail!(
            "repository {} must declare at least one producer path",
            repo.id
        );
    }
    let mut seen = BTreeSet::new();
    for path in &repo.producer_paths {
        if !is_clean_relative_path(path) {
            bail!(
                "repository {} has an invalid producer path: {}",
                repo.id,
                path
            );
        }
        if !seen.insert(path.as_str()) {
            bail!(
                "repository {} has a duplicate producer path: {}",
                repo.id,
                path
            );
        }
    }
    Ok(())
}

fn is_clean_relative_path(path: &str) -> bool {
    !path.is_empty() && !path.starts_with('/') && path.split('/').all(|part| part != "..")
}

/// The parity tree is anchored to the lock's directory, so the validator and
/// its tests run against any root (`parity/` in the repository, a temp dir in
/// unit tests).
fn parity_root(source_lock: &Path) -> &Path {
    source_lock.parent().unwrap_or_else(|| Path::new("."))
}

pub(crate) fn vendored_openapi_path(source_lock: &Path) -> PathBuf {
    parity_root(source_lock).join("openapi/services-orderbook.yml")
}

/// Validates every fixture under `<lock dir>/fixtures/**/*.json` against the
/// lock: a parseable header, a unique non-empty `surface`, at least one of
/// `sources`/`standards`, source commits equal to the owning pin (the
/// freshness ratchet), refs and case-level `source_ref`s confined to declared
/// producer paths, and no provenance-lookalike keys. Returns the file count.
fn validate_fixtures(source_lock: &Path, lock: &SourceLock) -> Result<usize> {
    let repo_paths: BTreeMap<&str, BTreeSet<&str>> = lock
        .repositories
        .iter()
        .map(|repo| {
            (
                repo.id.as_str(),
                repo.producer_paths.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    let repo_commits: BTreeMap<&str, &str> = lock
        .repositories
        .iter()
        .map(|repo| (repo.id.as_str(), repo.commit.as_str()))
        .collect();

    let fixtures_root = parity_root(source_lock).join("fixtures");
    let files = collect_files(&fixtures_root, "json")
        .with_context(|| format!("failed to walk {}", fixtures_root.display()))?;
    if files.is_empty() {
        bail!("no fixture files found under {}", fixtures_root.display());
    }

    let mut surfaces: BTreeMap<String, PathBuf> = BTreeMap::new();
    for file in &files {
        validate_fixture_file(file, &repo_paths, &repo_commits, &mut surfaces)
            .with_context(|| format!("fixture {} failed validation", file.display()))?;
    }
    Ok(files.len())
}

fn validate_fixture_file(
    file: &Path,
    repo_paths: &BTreeMap<&str, BTreeSet<&str>>,
    repo_commits: &BTreeMap<&str, &str>,
    surfaces: &mut BTreeMap<String, PathBuf>,
) -> Result<()> {
    let raw = fs::read_to_string(file).context("failed to read fixture")?;
    let json: serde_json::Value = serde_json::from_str(&raw).context("failed to parse fixture")?;
    let object = json
        .as_object()
        .context("fixture root must be a JSON object")?;

    for key in PROVENANCE_LOOKALIKE_KEYS {
        if object.contains_key(*key) {
            bail!(
                "provenance-lookalike key `{key}` would be ignored as payload and stay \
                 unvalidated — use the sources/standards header (parity/README.md, fixture \
                 header grammar)"
            );
        }
    }

    let header: FixtureHeader =
        serde_json::from_value(json.clone()).context("invalid provenance header")?;

    if header.surface.trim().is_empty() {
        bail!("surface must be non-empty");
    }
    if let Some(previous) = surfaces.insert(header.surface.clone(), file.to_path_buf()) {
        bail!(
            "surface `{}` is already declared by {}",
            header.surface,
            previous.display()
        );
    }
    if header.sources.is_empty() && header.standards.is_empty() {
        bail!(
            "declares no sources or standards — every fixture names its provenance \
             (parity/README.md, fixture header grammar)"
        );
    }
    if header.standards.iter().any(|entry| entry.trim().is_empty()) {
        bail!("standards entries must be non-empty");
    }

    // Per-repo paths this file declares; case-level refs may only cite these.
    let mut declared: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for (repo, source) in &header.sources {
        let Some(known_paths) = repo_paths.get(repo.as_str()) else {
            bail!("cites repo `{repo}` which is not pinned in the source lock");
        };
        let pinned = repo_commits
            .get(repo.as_str())
            .expect("repo_paths and repo_commits share keys");
        if source.commit != *pinned {
            bail!(
                "cites {repo}@{} but the lock pins {pinned}; re-verify the fixture against \
                 the new pin, then update its sources commit",
                source.commit
            );
        }
        if source.refs.is_empty() {
            bail!("sources entry for `{repo}` declares no refs");
        }
        let declared_paths = declared.entry(repo.as_str()).or_default();
        for ref_entry in &source.refs {
            let path = ref_entry.split('#').next().unwrap_or_default();
            if !is_clean_relative_path(path) {
                bail!("invalid ref path `{ref_entry}` for repo `{repo}`");
            }
            if !known_paths.contains(path) {
                bail!(
                    "references path not declared in the source lock: {repo}:{path} — add it \
                     to the lock row's producer_paths or fix the ref"
                );
            }
            declared_paths.insert(path);
        }
    }

    validate_case_refs(&json, &declared)
}

/// Walks the fixture payload for case-level `source_ref` strings
/// (`repo:path#fragment`). Commit segments are forbidden by grammar — commits
/// live once per repo in the file's `sources` — and a case may only cite a
/// path its file-level sources declare.
fn validate_case_refs(
    value: &serde_json::Value,
    declared: &BTreeMap<&str, BTreeSet<&str>>,
) -> Result<()> {
    match value {
        serde_json::Value::Object(map) => {
            for (key, nested) in map {
                if key == "@source_ref" {
                    bail!(
                        "provenance-lookalike key `@source_ref` would be ignored as payload and \
                         stay unvalidated — use `source_ref` with `repo:path#fragment` \
                         (parity/README.md, fixture header grammar)"
                    );
                }
                if key == "source_ref" {
                    let reference = nested
                        .as_str()
                        .context("source_ref must be a `repo:path#fragment` string")?;
                    validate_case_ref(reference, declared)?;
                }
                validate_case_refs(nested, declared)?;
            }
        }
        serde_json::Value::Array(items) => {
            for item in items {
                validate_case_refs(item, declared)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_case_ref(reference: &str, declared: &BTreeMap<&str, BTreeSet<&str>>) -> Result<()> {
    let Some((repo, rest)) = reference.split_once(':') else {
        bail!("source_ref `{reference}` must be `repo:path#fragment`");
    };
    if rest.split('#').next().unwrap_or_default().contains(':') {
        bail!(
            "source_ref `{reference}` carries a commit segment; commits live in the file's \
             sources.<repo>.commit, not in case refs"
        );
    }
    let path = rest.split('#').next().unwrap_or_default();
    if !declared.get(repo).is_some_and(|paths| paths.contains(path)) {
        bail!(
            "source_ref `{reference}` cites {repo}:{path}, which the fixture's sources do \
             not declare"
        );
    }
    Ok(())
}

/// Offline half of the vendored-`OpenAPI` drift gate: the stamp's commit must
/// equal the lock's services pin, so re-pinning without re-vendoring fails
/// closed on every `validate` run. A missing vendored file is not checked
/// here — `cargo parity-openapi-coverage` reads it on every run and owns its
/// existence.
fn validate_vendored_openapi_stamp(source_lock: &Path, lock: &SourceLock) -> Result<()> {
    let path = vendored_openapi_path(source_lock);
    if !path.exists() {
        return Ok(());
    }
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let stamped = raw
        .lines()
        .take_while(|line| line.starts_with('#'))
        .find_map(|line| line.strip_prefix(VENDORED_STAMP_PREFIX))
        .map(str::trim)
        .with_context(|| {
            format!(
                "{} is missing its provenance stamp; run `cargo parity-vendor-openapi`",
                path.display()
            )
        })?;
    let services = repository_entry(lock, "services")?;
    if stamped != services.commit {
        bail!(
            "{} is stamped {stamped} but the lock pins services at {}; run \
             `cargo parity-vendor-openapi`",
            path.display(),
            services.commit
        );
    }
    Ok(())
}

/// Deep half of the vendored-`OpenAPI` drift gate: the stamp-stripped body
/// must equal `crates/orderbook/openapi.yml` at the pinned services commit.
/// No committed checksum is involved — both sides derive from the pin.
fn validate_vendored_openapi_body(
    source_lock: &Path,
    lock: &SourceLock,
    upstream_root: &Path,
) -> Result<()> {
    let path = vendored_openapi_path(source_lock);
    if !path.exists() {
        return Ok(());
    }
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    let body = raw
        .lines()
        .skip_while(|line| line.starts_with('#'))
        .collect::<Vec<_>>()
        .join("\n");

    let services = repository_entry(lock, "services")?;
    let checkout = upstream_root.join(&services.id);
    let upstream = git_stdout(
        &checkout,
        &[
            "show",
            &format!("{}:{SERVICES_OPENAPI_PATH}", services.commit),
        ],
    )
    .with_context(|| {
        format!(
            "failed to read {SERVICES_OPENAPI_PATH} at the services pin from {}",
            checkout.display()
        )
    })?;
    if body.trim() != upstream.trim() {
        bail!(
            "{} body does not match {SERVICES_OPENAPI_PATH} at the pinned services commit; \
             run `cargo parity-vendor-openapi` or investigate the edit",
            path.display()
        );
    }
    Ok(())
}

/// Recursively collects files with the given extension, sorted for
/// deterministic reports.
pub(crate) fn collect_files(root: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_files_inner(root, extension, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_files_inner(current: &Path, extension: &str, files: &mut Vec<PathBuf>) -> Result<()> {
    for entry in
        fs::read_dir(current).with_context(|| format!("failed to read {}", current.display()))?
    {
        let path = entry
            .with_context(|| format!("failed to inspect entry in {}", current.display()))?
            .path();
        if path.is_dir() {
            collect_files_inner(&path, extension, files)?;
        } else if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            files.push(path);
        }
    }
    Ok(())
}

fn load_source_lock(path: &Path) -> Result<SourceLock> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_norway::from_str(&raw).context("failed to parse source lock")
}

pub(crate) fn repository_entry<'a>(lock: &'a SourceLock, id: &str) -> Result<&'a RepositoryEntry> {
    lock.repositories
        .iter()
        .find(|repo| repo.id == id)
        .with_context(|| format!("missing repository entry for {id}"))
}

pub(crate) fn validate_repository_root(repo: &RepositoryEntry, root: &Path) -> Result<()> {
    let canonical_root = fs::canonicalize(root)
        .with_context(|| format!("failed to canonicalize {}", root.display()))?;
    validate_git_toplevel(repo, &canonical_root)?;
    validate_repository_remote(repo, &canonical_root)?;

    let commit = git_stdout(&canonical_root, &["rev-parse", "HEAD"])?;
    if repo.commit != commit {
        bail!(
            "repository {} commit mismatch: lock={}, actual={}",
            repo.id,
            repo.commit,
            commit
        );
    }
    for producer_path in &repo.producer_paths {
        let path = canonical_root.join(producer_path);
        if !path.exists() {
            bail!("missing producer path {}", path.display());
        }
    }
    validate_clean_producer_paths(repo, &canonical_root)?;
    Ok(())
}

fn validate_git_toplevel(repo: &RepositoryEntry, root: &Path) -> Result<()> {
    let git_root_raw = git_stdout(root, &["rev-parse", "--show-toplevel"])?;
    let git_root = fs::canonicalize(Path::new(&git_root_raw))
        .with_context(|| format!("failed to canonicalize git top-level {git_root_raw}"))?;

    if git_root != root {
        bail!(
            "repository {} root mismatch: supplied root {} resolves to git top-level {}; supply an independent checkout of {} at the pinned source-lock commit",
            repo.id,
            root.display(),
            git_root.display(),
            repo.remote
        );
    }

    Ok(())
}

fn validate_repository_remote(repo: &RepositoryEntry, root: &Path) -> Result<()> {
    let expected = normalize_repository_remote(&repo.remote);
    let remotes = git_stdout(root, &["remote", "-v"])?;
    let mut found = Vec::new();

    for line in remotes.lines() {
        let mut parts = line.split_whitespace();
        let _name = parts.next();
        if let Some(url) = parts.next() {
            found.push(url.to_string());
        }
    }

    if found
        .iter()
        .any(|remote| normalize_repository_remote(remote) == expected)
    {
        return Ok(());
    }

    let found = if found.is_empty() {
        "none".to_string()
    } else {
        found.join(", ")
    };
    bail!(
        "repository {} remote mismatch: expected {}, found {}",
        repo.id,
        repo.remote,
        found
    );
}

fn validate_clean_producer_paths(repo: &RepositoryEntry, root: &Path) -> Result<()> {
    let mut args: Vec<&str> = vec!["status", "--porcelain", "--"];
    args.extend(repo.producer_paths.iter().map(String::as_str));

    let status = git_stdout(root, &args)?;
    if !status.trim().is_empty() {
        bail!(
            "repository {} has uncommitted changes in producer paths:\n{}",
            repo.id,
            status.trim()
        );
    }

    Ok(())
}

fn normalize_repository_remote(remote: &str) -> String {
    let mut normalized = remote.trim().trim_end_matches('/').to_ascii_lowercase();

    if let Some(rest) = normalized.strip_prefix("git@github.com:") {
        normalized = format!("github.com/{rest}");
    } else if let Some(rest) = normalized.strip_prefix("ssh://git@github.com/") {
        normalized = format!("github.com/{rest}");
    } else if let Some(rest) = normalized.strip_prefix("https://github.com/") {
        normalized = format!("github.com/{rest}");
    }

    normalized
        .strip_suffix(".git")
        .unwrap_or(&normalized)
        .to_string()
}

pub(crate) fn run_git_command(root: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .current_dir(root)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git in {}", root.display()))?;
    if !output.status.success() {
        bail!(
            "git command failed in {}: {}",
            root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

pub(crate) fn git_stdout(root: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git in {}", root.display()))?;
    if !output.status.success() {
        bail!(
            "git command failed in {}: {}",
            root.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::{
        CliOptions, RepositoryEntry, VENDORED_STAMP_PREFIX, git_stdout, validate,
        validate_repository_form,
    };
    use anyhow::{Context, Result, bail};
    use std::{
        env, fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    const REMOTE: &str = "https://github.com/cowprotocol/contracts.git";
    const VALID_SHA: &str = "c6b61ce75841ce4c25ab126def9cc981c568e6c6";
    const OTHER_SHA: &str = "1111111111111111111111111111111111111111";
    const PRODUCER_PATHS: &[&str] = &["src/ts/order.ts", "src/ts/sign.ts"];

    fn temp_dir(name: &str) -> Result<PathBuf> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        let dir = env::temp_dir().join(format!("xtask-{name}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&dir).with_context(|| format!("failed to create {}", dir.display()))?;
        Ok(dir)
    }

    fn entry(commit: &str, remote: &str, producer_paths: &[&str]) -> RepositoryEntry {
        RepositoryEntry {
            id: "contracts".to_owned(),
            remote: remote.to_owned(),
            commit: commit.to_owned(),
            producer_paths: producer_paths
                .iter()
                .map(|path| (*path).to_owned())
                .collect(),
        }
    }

    fn cli(source_lock: PathBuf, upstream_root: Option<PathBuf>) -> CliOptions {
        CliOptions {
            source_lock,
            upstream_root,
        }
    }

    /// Writes a well-formed lock with one `contracts` repo pinned at `commit`
    /// plus a minimal valid fixture, so corpus-level rules have something to
    /// pass. Tests then overwrite the fixture (or add files) to exercise one
    /// rule at a time.
    fn write_lock_and_fixture(dir: &Path, commit: &str) -> Result<PathBuf> {
        let lock = dir.join("source-lock.yaml");
        fs::write(
            &lock,
            format!(
                "repositories:\n- id: contracts\n  remote: {REMOTE}\n  commit: {commit}\n  \
                 producer_paths:\n  - src/ts/order.ts\n  - src/ts/sign.ts\n"
            ),
        )?;
        write_fixture(dir, "contracts.json", &valid_fixture_json(commit))?;
        Ok(lock)
    }

    fn valid_fixture_json(commit: &str) -> serde_json::Value {
        serde_json::json!({
            "surface": "contracts",
            "sources": {
                "contracts": {
                    "commit": commit,
                    "refs": ["src/ts/order.ts#ORDER_TYPE_FIELDS"]
                }
            },
            "cases": [
                { "id": "case-1", "source_ref": "contracts:src/ts/order.ts#ORDER_TYPE_FIELDS" }
            ]
        })
    }

    fn write_fixture(dir: &Path, name: &str, json: &serde_json::Value) -> Result<()> {
        let path = dir.join("fixtures").join(name);
        fs::create_dir_all(path.parent().expect("fixture path has a parent"))?;
        fs::write(
            &path,
            serde_json::to_string_pretty(json).expect("fixture json serializes"),
        )?;
        Ok(())
    }

    #[test]
    fn malformed_source_lock_files_fail_closed() -> Result<()> {
        let dir = temp_dir("malformed-lock")?;

        // Unknown fields are rejected by the typed model, so a stray or
        // legacy key (for example the retired `fixtures:` section) cannot be
        // silently ignored.
        let unknown = dir.join("unknown.yaml");
        fs::write(&unknown, "repositories: []\nfixtures: []\n")?;
        let error = validate(&cli(unknown, None)).expect_err("unknown fields must fail");
        assert!(format!("{error:#}").contains("failed to parse source lock"));

        // Missing required sections fail the same way.
        let missing = dir.join("missing.yaml");
        fs::write(&missing, "schema: legacy\n")?;
        let error = validate(&cli(missing, None)).expect_err("missing repositories must fail");
        assert!(format!("{error:#}").contains("failed to parse source lock"));

        fs::remove_dir_all(&dir).ok();
        Ok(())
    }

    #[test]
    fn validate_accepts_a_well_formed_lock_and_fixture_corpus() -> Result<()> {
        let dir = temp_dir("accept")?;
        let lock = write_lock_and_fixture(&dir, VALID_SHA)?;

        validate(&cli(lock, None))?;

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_repository_ids() -> Result<()> {
        let dir = temp_dir("dup")?;
        let lock = dir.join("source-lock.yaml");
        let row = format!(
            "- id: contracts\n  remote: {REMOTE}\n  commit: {VALID_SHA}\n  producer_paths:\n  - \
             src/ts/order.ts\n"
        );
        fs::write(&lock, format!("repositories:\n{row}{row}"))?;

        let error = validate(&cli(lock, None)).expect_err("duplicate ids must be rejected");
        assert!(
            format!("{error:#}").contains("duplicate repository id"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn repository_form_accepts_a_well_formed_row() {
        validate_repository_form(&entry(VALID_SHA, REMOTE, PRODUCER_PATHS))
            .expect("a well-formed row passes");
    }

    #[test]
    fn repository_form_rejects_a_non_hex_commit() {
        let error = validate_repository_form(&entry("not-a-sha", REMOTE, PRODUCER_PATHS))
            .expect_err("a non-hex commit is rejected");
        assert!(format!("{error:#}").contains("40-character lowercase hex"));
    }

    #[test]
    fn repository_form_rejects_a_non_github_remote() {
        let error = validate_repository_form(&entry(
            VALID_SHA,
            "git@github.com:cowprotocol/contracts.git",
            PRODUCER_PATHS,
        ))
        .expect_err("a non-https remote is rejected");
        assert!(format!("{error:#}").contains("https github .git URL"));
    }

    #[test]
    fn repository_form_rejects_empty_traversing_and_duplicate_producer_paths() {
        let empty = validate_repository_form(&entry(VALID_SHA, REMOTE, &[]))
            .expect_err("empty producer paths are rejected");
        assert!(format!("{empty:#}").contains("at least one producer path"));

        let traversal = validate_repository_form(&entry(VALID_SHA, REMOTE, &["../escape.ts"]))
            .expect_err("a traversing producer path is rejected");
        assert!(format!("{traversal:#}").contains("invalid producer path"));

        let duplicate = validate_repository_form(&entry(
            VALID_SHA,
            REMOTE,
            &["src/ts/order.ts", "src/ts/order.ts"],
        ))
        .expect_err("a duplicate producer path is rejected");
        assert!(format!("{duplicate:#}").contains("duplicate producer path"));
    }

    #[test]
    fn fixtures_without_provenance_fail_closed() -> Result<()> {
        let dir = temp_dir("no-provenance")?;
        let lock = write_lock_and_fixture(&dir, VALID_SHA)?;
        write_fixture(
            &dir,
            "orderbook/trade.json",
            &serde_json::json!({ "surface": "orderbook-trade", "payload": {} }),
        )?;

        let error = validate(&cli(lock, None)).expect_err("headerless fixtures must fail");
        assert!(
            format!("{error:#}").contains("declares no sources or standards"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn fixtures_with_provenance_lookalike_keys_fail_closed() -> Result<()> {
        let dir = temp_dir("lookalike")?;
        let lock = write_lock_and_fixture(&dir, VALID_SHA)?;
        // `source` is the near-miss of `sources`: with `standards` also
        // present the presence rule would pass and the typo would silently
        // carry unvalidated provenance.
        let mut fixture = valid_fixture_json(VALID_SHA);
        fixture["standards"] = serde_json::json!(["EIP-712"]);
        fixture["source"] = serde_json::json!("contracts src/ts/order.ts");
        write_fixture(&dir, "contracts.json", &fixture)?;

        let error = validate(&cli(lock, None)).expect_err("lookalike keys must fail");
        assert!(
            format!("{error:#}").contains("provenance-lookalike key `source`"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn fixtures_citing_a_stale_commit_trip_the_ratchet() -> Result<()> {
        let dir = temp_dir("ratchet")?;
        let lock = write_lock_and_fixture(&dir, VALID_SHA)?;
        write_fixture(&dir, "contracts.json", &valid_fixture_json(OTHER_SHA))?;

        let error = validate(&cli(lock, None)).expect_err("stale commits must fail");
        let rendered = format!("{error:#}");
        assert!(
            rendered.contains("re-verify the fixture against the new pin"),
            "unexpected error: {rendered}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn fixtures_citing_undeclared_paths_or_repos_fail() -> Result<()> {
        let dir = temp_dir("undeclared")?;
        let lock = write_lock_and_fixture(&dir, VALID_SHA)?;

        let mut unknown_repo = valid_fixture_json(VALID_SHA);
        unknown_repo["sources"] = serde_json::json!({
            "services": { "commit": VALID_SHA, "refs": ["crates/model/src/order.rs"] }
        });
        unknown_repo["cases"] = serde_json::json!([]);
        write_fixture(&dir, "contracts.json", &unknown_repo)?;
        let error = validate(&cli(lock.clone(), None)).expect_err("unknown repo must fail");
        assert!(format!("{error:#}").contains("not pinned in the source lock"));

        let mut unknown_path = valid_fixture_json(VALID_SHA);
        unknown_path["sources"]["contracts"]["refs"] = serde_json::json!(["src/ts/types.ts#Order"]);
        unknown_path["cases"] = serde_json::json!([]);
        write_fixture(&dir, "contracts.json", &unknown_path)?;
        let error = validate(&cli(lock, None)).expect_err("undeclared path must fail");
        assert!(
            format!("{error:#}").contains("references path not declared in the source lock"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn case_refs_with_commit_segments_or_undeclared_paths_fail() -> Result<()> {
        let dir = temp_dir("case-refs")?;
        let lock = write_lock_and_fixture(&dir, VALID_SHA)?;

        let mut commit_segment = valid_fixture_json(VALID_SHA);
        commit_segment["cases"][0]["source_ref"] =
            serde_json::json!("contracts:c94c595a:src/ts/order.ts#ORDER_TYPE_FIELDS");
        write_fixture(&dir, "contracts.json", &commit_segment)?;
        let error = validate(&cli(lock.clone(), None)).expect_err("commit segments must fail");
        assert!(
            format!("{error:#}").contains("carries a commit segment"),
            "unexpected error: {error:#}"
        );

        let mut undeclared = valid_fixture_json(VALID_SHA);
        undeclared["cases"][0]["source_ref"] = serde_json::json!("contracts:src/ts/sign.ts#x");
        write_fixture(&dir, "contracts.json", &undeclared)?;
        let error = validate(&cli(lock.clone(), None)).expect_err("undeclared case ref must fail");
        assert!(
            format!("{error:#}").contains("which the fixture's sources do not declare"),
            "unexpected error: {error:#}"
        );

        let mut lookalike = valid_fixture_json(VALID_SHA);
        lookalike["cases"][0] = serde_json::json!({ "id": "case-1", "@source_ref": "anything" });
        write_fixture(&dir, "contracts.json", &lookalike)?;
        let error = validate(&cli(lock, None)).expect_err("lookalike case refs must fail");
        assert!(
            format!("{error:#}").contains("provenance-lookalike key `@source_ref`"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn duplicate_surfaces_across_files_fail() -> Result<()> {
        let dir = temp_dir("dup-surface")?;
        let lock = write_lock_and_fixture(&dir, VALID_SHA)?;
        write_fixture(&dir, "second.json", &valid_fixture_json(VALID_SHA))?;

        let error = validate(&cli(lock, None)).expect_err("duplicate surfaces must fail");
        assert!(
            format!("{error:#}").contains("is already declared by"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn standards_only_fixtures_are_valid() -> Result<()> {
        let dir = temp_dir("standards")?;
        let lock = write_lock_and_fixture(&dir, VALID_SHA)?;
        write_fixture(
            &dir,
            "retry_after/accept.json",
            &serde_json::json!({
                "surface": "retry-after-accept",
                "standards": ["RFC 7231 §7.1.1.1"],
                "cases": []
            }),
        )?;

        validate(&cli(lock, None))?;

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn vendored_openapi_stamp_must_match_the_services_pin() -> Result<()> {
        let dir = temp_dir("stamp")?;
        let lock = dir.join("source-lock.yaml");
        fs::write(
            &lock,
            format!(
                "repositories:\n- id: services\n  remote: \
                 https://github.com/cowprotocol/services.git\n  commit: {VALID_SHA}\n  \
                 producer_paths:\n  - crates/orderbook/openapi.yml\n"
            ),
        )?;
        write_fixture(
            &dir,
            "wire.json",
            &serde_json::json!({
                "surface": "wire",
                "sources": {
                    "services": { "commit": VALID_SHA, "refs": ["crates/orderbook/openapi.yml"] }
                }
            }),
        )?;
        let vendored = dir.join("openapi/services-orderbook.yml");
        fs::create_dir_all(vendored.parent().expect("vendored path has a parent"))?;

        fs::write(
            &vendored,
            format!("{VENDORED_STAMP_PREFIX}{VALID_SHA}\n# Path: x\nopenapi: 3.0.3\n"),
        )?;
        validate(&cli(lock.clone(), None))?;

        fs::write(
            &vendored,
            format!("{VENDORED_STAMP_PREFIX}{OTHER_SHA}\n# Path: x\nopenapi: 3.0.3\n"),
        )?;
        let error = validate(&cli(lock, None)).expect_err("stale stamps must fail");
        assert!(
            format!("{error:#}").contains("run `cargo parity-vendor-openapi`"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    fn run_git(root: &Path, args: &[&str]) -> Result<()> {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args(args)
            .output()
            .with_context(|| format!("failed to run git in {}", root.display()))?;
        if !output.status.success() {
            bail!(
                "git command failed in {}: {}",
                root.display(),
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(())
    }

    fn init_repo(root: &Path, remote: &str) -> Result<String> {
        fs::create_dir_all(root)?;
        for producer_path in PRODUCER_PATHS {
            let path = root.join(producer_path);
            fs::create_dir_all(path.parent().expect("producer path has a parent"))?;
            fs::write(&path, format!("fixture source for {producer_path}\n"))?;
        }
        run_git(root, &["init"])?;
        run_git(root, &["config", "user.email", "tests@example.com"])?;
        run_git(root, &["config", "user.name", "xtask tests"])?;
        run_git(root, &["remote", "add", "origin", remote])?;
        run_git(root, &["add", "."])?;
        run_git(root, &["commit", "-m", "initial fixture sources"])?;
        git_stdout(root, &["rev-parse", "HEAD"])
    }

    /// Writes a lock + fixture pinned at the checkout's real commit, with the
    /// checkout living at `<dir>/upstreams/contracts` — the `<root>/<id>`
    /// layout deep validation expects.
    fn deep_setup(dir: &Path) -> Result<(PathBuf, PathBuf, String)> {
        let upstreams = dir.join("upstreams");
        let contracts = upstreams.join("contracts");
        let commit = init_repo(&contracts, REMOTE)?;
        let lock = write_lock_and_fixture(dir, &commit)?;
        Ok((lock, upstreams, commit))
    }

    #[test]
    fn validate_with_upstream_root_accepts_a_matching_checkout() -> Result<()> {
        let dir = temp_dir("roots-accept")?;
        let (lock, upstreams, _) = deep_setup(&dir)?;

        validate(&cli(lock, Some(upstreams)))?;

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_with_upstream_root_accepts_an_ssh_remote() -> Result<()> {
        let dir = temp_dir("roots-ssh")?;
        let (lock, upstreams, _) = deep_setup(&dir)?;
        run_git(
            &upstreams.join("contracts"),
            &[
                "remote",
                "set-url",
                "origin",
                "git@github.com:cowprotocol/contracts.git",
            ],
        )?;

        validate(&cli(lock, Some(upstreams)))?;

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_with_upstream_root_rejects_a_wrong_remote() -> Result<()> {
        let dir = temp_dir("roots-remote")?;
        let (lock, upstreams, _) = deep_setup(&dir)?;
        run_git(
            &upstreams.join("contracts"),
            &[
                "remote",
                "set-url",
                "origin",
                "https://github.com/example/contracts.git",
            ],
        )?;

        let error = validate(&cli(lock, Some(upstreams))).expect_err("a wrong remote is rejected");
        assert!(
            format!("{error:#}").contains("remote mismatch"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_with_upstream_root_rejects_dirty_producer_paths() -> Result<()> {
        let dir = temp_dir("roots-dirty")?;
        let (lock, upstreams, _) = deep_setup(&dir)?;
        fs::write(
            upstreams.join("contracts/src/ts/order.ts"),
            "local uncommitted drift\n",
        )?;

        let error =
            validate(&cli(lock, Some(upstreams))).expect_err("dirty producer paths are rejected");
        assert!(
            format!("{error:#}").contains("has uncommitted changes in producer paths"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_with_upstream_root_rejects_a_commit_mismatch() -> Result<()> {
        let dir = temp_dir("roots-commit")?;
        let upstreams = dir.join("upstreams");
        let contracts = upstreams.join("contracts");
        init_repo(&contracts, REMOTE)?;
        // The lock pins a different (well-formed) commit than the checkout's
        // HEAD, and the fixture cites the lock's pin so only the deep check
        // can fail.
        let lock = write_lock_and_fixture(&dir, VALID_SHA)?;

        let error =
            validate(&cli(lock, Some(upstreams))).expect_err("a commit mismatch is rejected");
        assert!(
            format!("{error:#}").contains("commit mismatch"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_with_upstream_root_rejects_a_missing_checkout() -> Result<()> {
        let dir = temp_dir("roots-missing")?;
        let lock = write_lock_and_fixture(&dir, VALID_SHA)?;

        let error = validate(&cli(lock, Some(dir.join("upstreams"))))
            .expect_err("a missing checkout is rejected");
        assert!(
            format!("{error:#}").contains("deep validation failed for contracts"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }
}
