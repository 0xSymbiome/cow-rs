//! Upstream-parity maintenance: the source-lock model and validator, pinned
//! checkout provisioning, the vendored `OpenAPI` document, DTO coverage, and
//! the deployment-registry presence probe.

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

/// Roles a repository row may declare. The validator checks the form of the
/// committed source lock rather than matching it against hardcoded content, so
/// adding or re-pinning an upstream is a single edit to `parity/source-lock.yaml`.
const REPOSITORY_ROLES: &[&str] = &["primary", "wire-authority", "primary-via-submodule"];

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceLock {
    repositories: Vec<RepositoryEntry>,
    fixtures: Vec<FixtureEntry>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RepositoryEntry {
    id: String,
    remote: String,
    commit: String,
    role: String,
    producer_paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureEntry {
    surface: String,
    file: String,
    source_refs: Vec<FixtureSourceRef>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureSourceRef {
    repo: String,
    path: String,
}

pub struct CliOptions {
    pub source_lock: PathBuf,
    pub output_root: Option<PathBuf>,
    pub contracts_root: Option<PathBuf>,
    pub services_root: Option<PathBuf>,
    pub cow_sdk_root: Option<PathBuf>,
}

pub fn provision_upstreams(options: &CliOptions) -> Result<()> {
    let lock = load_source_lock(&options.source_lock)?;

    let output_root = options
        .output_root
        .as_deref()
        .context("provision-upstreams requires --output-root")?;

    if output_root.exists() {
        fs::remove_dir_all(output_root)
            .with_context(|| format!("failed to clear {}", output_root.display()))?;
    }
    fs::create_dir_all(output_root)
        .with_context(|| format!("failed to create {}", output_root.display()))?;

    for repo in &lock.repositories {
        let checkout_root = output_root.join(&repo.id);
        provision_repository_checkout(repo, &checkout_root)?;
    }

    println!(
        "provisioned {} pinned upstream repositories under {}",
        lock.repositories.len(),
        output_root.display()
    );
    Ok(())
}

/// Validates the committed source lock by form.
///
/// Checks unique repository ids, each repository row's shape, the committed
/// fixture provenance, and — when upstream roots are supplied — the pinned
/// commits and clean producer paths in those checkouts. The lock content is
/// the single source of truth; the validator never matches it against
/// hardcoded rows.
pub fn validate(options: &CliOptions) -> Result<()> {
    let lock = load_source_lock(&options.source_lock)?;

    let actual_repos: BTreeMap<&str, &RepositoryEntry> = lock
        .repositories
        .iter()
        .map(|repo| (repo.id.as_str(), repo))
        .collect();
    if actual_repos.len() != lock.repositories.len() {
        bail!("duplicate repository id in source lock");
    }

    for repo in &lock.repositories {
        validate_repository_form(repo)?;
    }

    for (id, root) in &resolve_optional_roots(options) {
        let repo = actual_repos
            .get(id.as_str())
            .with_context(|| format!("missing repository entry for {id}"))?;
        validate_repository_root(repo, root)?;
    }

    validate_fixtures(&lock)?;

    println!(
        "validated {} repositories and {} fixture contracts",
        lock.repositories.len(),
        lock.fixtures.len()
    );
    Ok(())
}

/// Validates one repository row's shape: a GitHub `.git` remote, a
/// 40-character lowercase hex commit, a known role, and a non-empty list of
/// relative producer paths with no traversal or duplicates.
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
    if !REPOSITORY_ROLES.contains(&repo.role.as_str()) {
        bail!("repository {} has unknown role: {}", repo.id, repo.role);
    }
    if repo.producer_paths.is_empty() {
        bail!(
            "repository {} must declare at least one producer path",
            repo.id
        );
    }
    let mut seen = BTreeSet::new();
    for path in &repo.producer_paths {
        if path.is_empty() || path.starts_with('/') || path.split('/').any(|part| part == "..") {
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

fn validate_fixtures(lock: &SourceLock) -> Result<()> {
    let repo_paths: BTreeMap<&str, Vec<&str>> = lock
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

    for fixture in &lock.fixtures {
        if !Path::new(&fixture.file).exists() {
            bail!("fixture file missing: {}", fixture.file);
        }
        let fixture_raw = fs::read_to_string(&fixture.file)
            .with_context(|| format!("failed to read fixture {}", fixture.file))?;
        let fixture_json: serde_json::Value = serde_json::from_str(&fixture_raw)
            .with_context(|| format!("failed to parse fixture {}", fixture.file))?;

        for source_ref in &fixture.source_refs {
            let known_paths = repo_paths
                .get(source_ref.repo.as_str())
                .with_context(|| format!("unknown repo {} in fixture contract", source_ref.repo))?;
            if !known_paths.contains(&source_ref.path.as_str()) {
                bail!(
                    "fixture {} references path not declared in source lock: {}:{}",
                    fixture.file,
                    source_ref.repo,
                    source_ref.path
                );
            }
        }

        if let Some(embedded_source_refs) = fixture_json
            .get("source_refs")
            .and_then(serde_json::Value::as_array)
        {
            for embedded_ref in embedded_source_refs {
                let Some(repo) = embedded_ref.get("repo").and_then(serde_json::Value::as_str)
                else {
                    bail!("fixture {} has source_ref without repo", fixture.file);
                };
                let Some(path) = embedded_ref.get("path").and_then(serde_json::Value::as_str)
                else {
                    bail!("fixture {} has source_ref without path", fixture.file);
                };

                let known_paths = repo_paths
                    .get(repo)
                    .with_context(|| format!("unknown repo {repo} in fixture {}", fixture.file))?;
                if !known_paths.contains(&path) {
                    bail!(
                        "fixture {} embeds source_ref path not declared in source lock: {}:{}",
                        fixture.file,
                        repo,
                        path
                    );
                }

                if let Some(commit) = embedded_ref
                    .get("commit")
                    .and_then(serde_json::Value::as_str)
                {
                    let expected_commit = repo_commits.get(repo).with_context(|| {
                        format!("missing repository commit for embedded source_ref repo {repo}")
                    })?;
                    if commit != *expected_commit {
                        bail!(
                            "fixture {} embeds stale commit for repo {}: fixture={}, lock={}",
                            fixture.file,
                            repo,
                            commit,
                            expected_commit
                        );
                    }
                }
            }
        }
    }

    Ok(())
}

fn resolve_optional_roots(options: &CliOptions) -> BTreeMap<String, PathBuf> {
    let mut roots = BTreeMap::new();
    if let Some(path) = &options.contracts_root {
        roots.insert("contracts".to_string(), path.clone());
    }
    if let Some(path) = &options.services_root {
        roots.insert("services".to_string(), path.clone());
    }
    if let Some(path) = &options.cow_sdk_root {
        roots.insert("cow-sdk".to_string(), path.clone());
    }
    roots
}

fn load_source_lock(path: &Path) -> Result<SourceLock> {
    let raw =
        fs::read_to_string(path).with_context(|| format!("failed to read {}", path.display()))?;
    serde_norway::from_str(&raw).context("failed to parse source lock")
}

fn repository_entry<'a>(lock: &'a SourceLock, id: &str) -> Result<&'a RepositoryEntry> {
    lock.repositories
        .iter()
        .find(|repo| repo.id == id)
        .with_context(|| format!("missing repository entry for {id}"))
}

fn validate_repository_root(repo: &RepositoryEntry, root: &Path) -> Result<()> {
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

fn provision_repository_checkout(repo: &RepositoryEntry, checkout_root: &Path) -> Result<()> {
    if checkout_root.exists() {
        fs::remove_dir_all(checkout_root)
            .with_context(|| format!("failed to clear {}", checkout_root.display()))?;
    }

    let parent = checkout_root
        .parent()
        .with_context(|| format!("missing parent for {}", checkout_root.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;

    let checkout_name = checkout_root
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("invalid checkout path {}", checkout_root.display()))?;
    run_git_command(
        parent,
        &[
            "clone",
            "--filter=blob:none",
            "--no-checkout",
            repo.remote.as_str(),
            checkout_name,
        ],
    )?;

    if let Err(error) = run_git_command(
        checkout_root,
        &["fetch", "--depth", "1", "origin", repo.commit.as_str()],
    ) {
        eprintln!(
            "shallow fetch failed for {} at {}: {error:#}; retrying with a full commit fetch",
            repo.id, repo.commit
        );
        run_git_command(checkout_root, &["fetch", "origin", repo.commit.as_str()])?;
    }

    run_git_command(
        checkout_root,
        &["checkout", "--detach", repo.commit.as_str()],
    )?;
    validate_repository_root(repo, checkout_root)?;
    Ok(())
}

fn run_git_command(root: &Path, args: &[&str]) -> Result<()> {
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

fn git_stdout(root: &Path, args: &[&str]) -> Result<String> {
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
    use super::{CliOptions, RepositoryEntry, git_stdout, validate, validate_repository_form};
    use anyhow::{Context, Result, bail};
    use std::{
        env,
        fmt::Write as _,
        fs,
        path::{Path, PathBuf},
        process::Command,
        time::{SystemTime, UNIX_EPOCH},
    };

    const REMOTE: &str = "https://github.com/cowprotocol/contracts.git";
    const VALID_SHA: &str = "c6b61ce75841ce4c25ab126def9cc981c568e6c6";
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

    fn entry(commit: &str, remote: &str, role: &str, producer_paths: &[&str]) -> RepositoryEntry {
        RepositoryEntry {
            id: "contracts".to_owned(),
            remote: remote.to_owned(),
            commit: commit.to_owned(),
            role: role.to_owned(),
            producer_paths: producer_paths
                .iter()
                .map(|path| (*path).to_owned())
                .collect(),
        }
    }

    fn cli(source_lock: PathBuf, contracts_root: Option<PathBuf>) -> CliOptions {
        CliOptions {
            source_lock,
            output_root: None,
            contracts_root,
            services_root: None,
            cow_sdk_root: None,
        }
    }

    #[test]
    fn malformed_source_lock_files_fail_closed() -> Result<()> {
        let dir = temp_dir("malformed-lock")?;

        // Unknown fields are rejected by the typed model, so a stray or
        // misspelled key (for example `producer_path:`) cannot be ignored.
        let unknown = dir.join("unknown.yaml");
        fs::write(
            &unknown,
            "repositories: []\nfixtures: []\nmetadata: legacy\n",
        )?;
        let error = validate(&cli(unknown, None)).expect_err("unknown fields must fail");
        assert!(format!("{error:#}").contains("failed to parse source lock"));

        // Missing required sections fail the same way.
        let missing = dir.join("missing.yaml");
        fs::write(&missing, "repositories: []\n")?;
        let error = validate(&cli(missing, None)).expect_err("missing fixtures must fail");
        assert!(format!("{error:#}").contains("failed to parse source lock"));

        fs::remove_dir_all(&dir).ok();
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

    /// Writes a well-formed lock with one `contracts` repo pinned at
    /// `commit`, plus an optional fixture whose embedded `source_ref` pins
    /// `fixture_commit` (pass a value other than `commit` to exercise drift).
    fn write_lock(path: &Path, commit: &str, fixture: Option<(&Path, &str)>) -> Result<()> {
        let mut yaml = format!(
            "repositories:\n- id: contracts\n  remote: {REMOTE}\n  commit: {commit}\n  role: \
             primary\n  producer_paths:\n  - src/ts/order.ts\n  - src/ts/sign.ts\n"
        );
        if let Some((fixture_file, fixture_commit)) = fixture {
            let json = serde_json::json!({
                "surface": "contracts",
                "source_refs": [
                    { "repo": "contracts", "commit": fixture_commit, "path": "src/ts/order.ts" }
                ]
            });
            fs::write(
                fixture_file,
                serde_json::to_string_pretty(&json).expect("fixture json serializes"),
            )?;
            write!(
                yaml,
                "fixtures:\n- surface: contracts\n  file: '{}'\n  source_refs:\n  - repo: \
                 contracts\n    path: src/ts/order.ts\n",
                fixture_file.display()
            )
            .expect("writing to a String is infallible");
        } else {
            yaml.push_str("fixtures: []\n");
        }
        fs::write(path, yaml).with_context(|| format!("failed to write {}", path.display()))
    }

    #[test]
    fn validate_accepts_a_well_formed_lock_and_fixture_without_roots() -> Result<()> {
        let dir = temp_dir("accept")?;
        let lock = dir.join("source-lock.yaml");
        let fixture = dir.join("contracts.json");
        write_lock(&lock, VALID_SHA, Some((&fixture, VALID_SHA)))?;

        validate(&cli(lock, None))?;

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_rejects_duplicate_repository_ids() -> Result<()> {
        let dir = temp_dir("dup")?;
        let lock = dir.join("source-lock.yaml");
        let row = format!(
            "- id: contracts\n  remote: {REMOTE}\n  commit: {VALID_SHA}\n  role: primary\n  \
             producer_paths:\n  - src/ts/order.ts\n"
        );
        fs::write(&lock, format!("repositories:\n{row}{row}fixtures: []\n"))?;

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
        validate_repository_form(&entry(VALID_SHA, REMOTE, "primary", PRODUCER_PATHS))
            .expect("a well-formed row passes");
    }

    #[test]
    fn repository_form_rejects_a_non_hex_commit() {
        let error =
            validate_repository_form(&entry("not-a-sha", REMOTE, "primary", PRODUCER_PATHS))
                .expect_err("a non-hex commit is rejected");
        assert!(format!("{error:#}").contains("40-character lowercase hex"));
    }

    #[test]
    fn repository_form_rejects_a_non_github_remote() {
        let error = validate_repository_form(&entry(
            VALID_SHA,
            "git@github.com:cowprotocol/contracts.git",
            "primary",
            PRODUCER_PATHS,
        ))
        .expect_err("a non-https remote is rejected");
        assert!(format!("{error:#}").contains("https github .git URL"));
    }

    #[test]
    fn repository_form_rejects_an_unknown_role() {
        let error =
            validate_repository_form(&entry(VALID_SHA, REMOTE, "reference-only", PRODUCER_PATHS))
                .expect_err("an unknown role is rejected");
        assert!(format!("{error:#}").contains("unknown role"));
    }

    #[test]
    fn repository_form_rejects_empty_traversing_and_duplicate_producer_paths() {
        let empty = validate_repository_form(&entry(VALID_SHA, REMOTE, "primary", &[]))
            .expect_err("empty producer paths are rejected");
        assert!(format!("{empty:#}").contains("at least one producer path"));

        let traversal =
            validate_repository_form(&entry(VALID_SHA, REMOTE, "primary", &["../escape.ts"]))
                .expect_err("a traversing producer path is rejected");
        assert!(format!("{traversal:#}").contains("invalid producer path"));

        let duplicate = validate_repository_form(&entry(
            VALID_SHA,
            REMOTE,
            "primary",
            &["src/ts/order.ts", "src/ts/order.ts"],
        ))
        .expect_err("a duplicate producer path is rejected");
        assert!(format!("{duplicate:#}").contains("duplicate producer path"));
    }

    #[test]
    fn validate_with_roots_accepts_a_matching_checkout() -> Result<()> {
        let dir = temp_dir("roots-accept")?;
        let contracts = dir.join("contracts");
        let commit = init_repo(&contracts, REMOTE)?;
        let lock = dir.join("source-lock.yaml");
        write_lock(&lock, &commit, None)?;

        validate(&cli(lock, Some(contracts)))?;

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_with_roots_accepts_an_ssh_remote() -> Result<()> {
        let dir = temp_dir("roots-ssh")?;
        let contracts = dir.join("contracts");
        let commit = init_repo(&contracts, REMOTE)?;
        run_git(
            &contracts,
            &[
                "remote",
                "set-url",
                "origin",
                "git@github.com:cowprotocol/contracts.git",
            ],
        )?;
        let lock = dir.join("source-lock.yaml");
        write_lock(&lock, &commit, None)?;

        validate(&cli(lock, Some(contracts)))?;

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_with_roots_rejects_a_wrong_remote() -> Result<()> {
        let dir = temp_dir("roots-remote")?;
        let contracts = dir.join("contracts");
        let commit = init_repo(&contracts, REMOTE)?;
        run_git(
            &contracts,
            &[
                "remote",
                "set-url",
                "origin",
                "https://github.com/example/contracts.git",
            ],
        )?;
        let lock = dir.join("source-lock.yaml");
        write_lock(&lock, &commit, None)?;

        let error = validate(&cli(lock, Some(contracts))).expect_err("a wrong remote is rejected");
        assert!(
            format!("{error:#}").contains("repository contracts remote mismatch"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_with_roots_rejects_dirty_producer_paths() -> Result<()> {
        let dir = temp_dir("roots-dirty")?;
        let contracts = dir.join("contracts");
        let commit = init_repo(&contracts, REMOTE)?;
        fs::write(
            contracts.join("src/ts/order.ts"),
            "local uncommitted drift\n",
        )?;
        let lock = dir.join("source-lock.yaml");
        write_lock(&lock, &commit, None)?;

        let error =
            validate(&cli(lock, Some(contracts))).expect_err("dirty producer paths are rejected");
        assert!(
            format!("{error:#}").contains("has uncommitted changes in producer paths"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_with_roots_rejects_a_commit_mismatch() -> Result<()> {
        let dir = temp_dir("roots-commit")?;
        let contracts = dir.join("contracts");
        init_repo(&contracts, REMOTE)?;
        let lock = dir.join("source-lock.yaml");
        write_lock(&lock, "0000000000000000000000000000000000000000", None)?;

        let error =
            validate(&cli(lock, Some(contracts))).expect_err("a commit mismatch is rejected");
        assert!(
            format!("{error:#}").contains("repository contracts commit mismatch"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }

    #[test]
    fn validate_rejects_fixture_commit_drift() -> Result<()> {
        let dir = temp_dir("fixture-drift")?;
        let lock = dir.join("source-lock.yaml");
        let fixture = dir.join("contracts.json");
        write_lock(
            &lock,
            VALID_SHA,
            Some((&fixture, "1111111111111111111111111111111111111111")),
        )?;

        let error = validate(&cli(lock, None)).expect_err("fixture commit drift is rejected");
        assert!(
            format!("{error:#}").contains("embeds stale commit for repo contracts"),
            "unexpected error: {error:#}"
        );

        let _ = fs::remove_dir_all(&dir);
        Ok(())
    }
}
