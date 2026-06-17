//! Lock-driven upstream synchronization and drift detection.
//!
//! `sync` materializes every source-lock repository as a blob-less clone
//! under a scratch root and checks out the pinned commit, so maintainers
//! review against exactly the snapshot the lock claims. `sync --update`
//! fetches each remote's default branch, prints the per-file drift table
//! (git blob OIDs — no committed checksums, the pin already content-addresses
//! every path), rewrites the lock's `commit:` lines textually (comments
//! preserved), and fails closed if any producer path is missing at the new
//! pin. `drift` is the read-only report with CI-friendly exit codes.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use super::{RepositoryEntry, git_stdout, load_source_lock, run_git_command};

/// Default scratch root for upstream checkouts (gitignored via `target/`).
pub const DEFAULT_UPSTREAM_ROOT: &str = "target/upstream";

#[derive(Debug, clap::Args)]
pub struct SyncArgs {
    #[arg(long, default_value = super::DEFAULT_SOURCE_LOCK)]
    pub source_lock: PathBuf,
    /// Repository ids to sync (default: every lock row).
    #[arg(long = "repo")]
    pub repos: Vec<String>,
    /// Checkout root for the upstream clones.
    #[arg(long, env = "XTASK_UPSTREAM_ROOT", default_value = DEFAULT_UPSTREAM_ROOT)]
    pub root: PathBuf,
    /// Advance pins to each remote's default branch and rewrite the lock.
    #[arg(long)]
    pub update: bool,
    /// Re-clone mismatched checkouts and reset dirty ones instead of refusing.
    #[arg(long)]
    pub force: bool,
}

#[derive(Debug, clap::Args)]
pub struct DriftArgs {
    #[arg(long, default_value = super::DEFAULT_SOURCE_LOCK)]
    pub source_lock: PathBuf,
    /// Repository ids to check (default: every lock row).
    #[arg(long = "repo")]
    pub repos: Vec<String>,
    /// Checkout root for the upstream clones.
    #[arg(long, env = "XTASK_UPSTREAM_ROOT", default_value = DEFAULT_UPSTREAM_ROOT)]
    pub root: PathBuf,
    /// Ref to compare the pins against (default: each remote's default branch).
    #[arg(long)]
    pub against: Option<String>,
}

/// Outcome of a read-only drift report.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DriftStatus {
    Clean,
    Drifted,
}

pub fn sync(args: &SyncArgs) -> Result<()> {
    let lock = load_source_lock(&args.source_lock)?;
    let targets = select_repositories(&lock.repositories, &args.repos)?;

    if args.update && !args.force && lock_has_uncommitted_edits(&args.source_lock)? {
        bail!(
            "{} has uncommitted edits; commit or stash them before --update (or pass --force)",
            args.source_lock.display()
        );
    }

    let mut updates: BTreeMap<String, String> = BTreeMap::new();
    for repo in &targets {
        let checkout = args.root.join(&repo.id);
        ensure_checkout(repo, &checkout, args.force)?;
        fetch_commit(&checkout, &repo.commit)
            .with_context(|| format!("pin {} unreachable for {}", repo.commit, repo.id))?;

        if args.update {
            if let Some(reason) = &repo.hold {
                println!(
                    "{}: pin held at {} and not advanced ({reason})",
                    repo.id,
                    &repo.commit[..12.min(repo.commit.len())]
                );
                checkout_detached(&checkout, &repo.commit, args.force)?;
                continue;
            }

            let head = remote_default_head(&checkout)
                .with_context(|| format!("failed to resolve the default branch of {}", repo.id))?;
            fetch_commit(&checkout, &head)
                .with_context(|| format!("failed to fetch {head} for {}", repo.id))?;

            let rows = drift_rows(&checkout, &repo.producer_paths, &repo.commit, &head)?;
            print_drift(&repo.id, &repo.commit, &head, &rows);

            let missing: Vec<&str> = rows
                .iter()
                .filter(|row| row.new_oid.is_none())
                .map(|row| row.path.as_str())
                .collect();
            if !missing.is_empty() {
                bail!(
                    "{}: producer path(s) missing at {head}: {} — re-pin manually after \
                     updating the lock's producer paths",
                    repo.id,
                    missing.join(", ")
                );
            }

            if head == repo.commit {
                println!("{}: already at the remote default branch head", repo.id);
            } else {
                updates.insert(repo.id.clone(), head.clone());
            }
            checkout_detached(&checkout, &head, args.force)?;
        } else {
            checkout_detached(&checkout, &repo.commit, args.force)?;
            println!(
                "{}: checked out pin {} under {}",
                repo.id,
                &repo.commit[..12],
                checkout.display()
            );
        }
    }

    if args.update {
        if updates.is_empty() {
            println!("all selected pins already match their remote default branches");
        } else {
            let mut text = fs::read_to_string(&args.source_lock)
                .with_context(|| format!("failed to read {}", args.source_lock.display()))?;
            for (id, commit) in &updates {
                text = update_lock_commit(&text, id, commit)?;
            }
            let tmp = args.source_lock.with_extension("yaml.tmp");
            fs::write(&tmp, &text).with_context(|| format!("failed to write {}", tmp.display()))?;
            fs::rename(&tmp, &args.source_lock)
                .with_context(|| format!("failed to replace {}", args.source_lock.display()))?;

            // The rewritten lock must still be well-formed before the update
            // counts. Full validation is deliberately NOT run here: the
            // fixture ratchet and the vendored-OpenAPI stamp are expected to
            // fail until the maintainer refreshes them, and that refresh is
            // the next step the drift table describes.
            super::validate_lock_form(&args.source_lock)?;
            println!(
                "updated {} pin(s) in {}; refresh the fixtures and vendored artifacts cited \
                 by the drift table above — `cargo parity-validate` fails closed until every \
                 stale fixture commit and the OpenAPI stamp are re-verified",
                updates.len(),
                args.source_lock.display()
            );
        }
    }
    Ok(())
}

pub fn drift(args: &DriftArgs) -> Result<DriftStatus> {
    let lock = load_source_lock(&args.source_lock)?;
    let targets = select_repositories(&lock.repositories, &args.repos)?;

    let mut status = DriftStatus::Clean;
    for repo in &targets {
        let checkout = args.root.join(&repo.id);
        ensure_checkout(repo, &checkout, false)?;
        fetch_commit(&checkout, &repo.commit)
            .with_context(|| format!("pin {} unreachable for {}", repo.commit, repo.id))?;

        let against = match &args.against {
            Some(reference) if is_commit_sha(reference) => reference.clone(),
            Some(reference) => resolve_remote_ref(&checkout, reference)
                .with_context(|| format!("failed to resolve {reference} for {}", repo.id))?,
            None => remote_default_head(&checkout)
                .with_context(|| format!("failed to resolve the default branch of {}", repo.id))?,
        };
        fetch_commit(&checkout, &against)
            .with_context(|| format!("failed to fetch {against} for {}", repo.id))?;

        // A held pin is intentionally behind its upstream default branch; its
        // movement is shown for visibility but never counts as actionable drift.
        let held = repo.hold.is_some();
        let rows = drift_rows(&checkout, &repo.producer_paths, &repo.commit, &against)?;
        print_drift(&repo.id, &repo.commit, &against, &rows);
        if let Some(reason) = &repo.hold {
            println!("  held: {reason}");
        }
        if !held && rows.iter().any(|row| row.old_oid != row.new_oid) {
            status = DriftStatus::Drifted;
        }

        // Additive-change radar: a watched directory is diffed over the union
        // of files present at each commit, so a newly added sibling (a new
        // schema version next to a tracked one) is surfaced even though no
        // tracked producer path changed.
        for dir in &repo.watch_dirs {
            let rows = watch_rows(&checkout, dir, &repo.commit, &against)?;
            print_watch(&repo.id, dir, &rows);
            if !held && rows.iter().any(|row| row.old_oid != row.new_oid) {
                status = DriftStatus::Drifted;
            }
        }
    }
    Ok(status)
}

struct DriftRow {
    path: String,
    old_oid: Option<String>,
    new_oid: Option<String>,
}

/// Compares producer-path blob OIDs between two commits.
fn drift_rows(
    checkout: &Path,
    producer_paths: &[String],
    old: &str,
    new: &str,
) -> Result<Vec<DriftRow>> {
    let old_oids = ls_tree_oids(checkout, old, producer_paths)?;
    let new_oids = ls_tree_oids(checkout, new, producer_paths)?;
    Ok(producer_paths
        .iter()
        .map(|path| DriftRow {
            path: path.clone(),
            old_oid: old_oids.get(path).cloned(),
            new_oid: new_oids.get(path).cloned(),
        })
        .collect())
}

fn print_drift(repo_id: &str, old: &str, new: &str, rows: &[DriftRow]) {
    let changed = rows.iter().filter(|row| row.old_oid != row.new_oid).count();
    let mut table = format!(
        "{repo_id}: {} -> {} ({changed} of {} producer path(s) differ)\n",
        &old[..12.min(old.len())],
        &new[..12.min(new.len())],
        rows.len()
    );
    for row in rows {
        let state = match (&row.old_oid, &row.new_oid) {
            (Some(a), Some(b)) if a == b => "unchanged",
            (Some(_), Some(_)) => "CHANGED",
            (Some(_), None) => "REMOVED",
            (None, Some(_)) => "missing-at-pin",
            (None, None) => "missing-at-both",
        };
        let _ = writeln!(table, "  {state:>15}  {}", row.path);
    }
    print!("{table}");
}

/// Compares the file set under one watched directory between two commits over
/// the union of paths present at either, so an added or removed file surfaces
/// even though no tracked producer path changed. `git ls-tree -r` expands the
/// directory pathspec recursively.
fn watch_rows(checkout: &Path, dir: &str, old: &str, new: &str) -> Result<Vec<DriftRow>> {
    let pathspec = [dir.to_owned()];
    let old_oids = ls_tree_oids(checkout, old, &pathspec)?;
    let new_oids = ls_tree_oids(checkout, new, &pathspec)?;
    let paths: BTreeSet<&String> = old_oids.keys().chain(new_oids.keys()).collect();
    Ok(paths
        .into_iter()
        .map(|path| DriftRow {
            path: path.clone(),
            old_oid: old_oids.get(path).cloned(),
            new_oid: new_oids.get(path).cloned(),
        })
        .collect())
}

/// Prints only the differing rows of a watched directory (the unchanged set is
/// summarized by count) so a weekly radar over a many-file tree stays readable.
fn print_watch(repo_id: &str, dir: &str, rows: &[DriftRow]) {
    let changed: Vec<&DriftRow> = rows.iter().filter(|row| row.old_oid != row.new_oid).collect();
    let mut table = format!(
        "{repo_id} watch {dir}: {} of {} file(s) differ\n",
        changed.len(),
        rows.len()
    );
    for row in changed {
        let state = match (&row.old_oid, &row.new_oid) {
            (None, Some(_)) => "ADDED",
            (Some(_), None) => "REMOVED",
            _ => "CHANGED",
        };
        let _ = writeln!(table, "  {state:>9}  {}", row.path);
    }
    print!("{table}");
}

/// Maps each producer path to its blob OID at `commit` (absent = no blob).
fn ls_tree_oids(
    checkout: &Path,
    commit: &str,
    producer_paths: &[String],
) -> Result<BTreeMap<String, String>> {
    let mut args: Vec<&str> = vec!["ls-tree", "-r", commit, "--"];
    args.extend(producer_paths.iter().map(String::as_str));
    let listing = git_stdout(checkout, &args)?;
    let mut oids = BTreeMap::new();
    for line in listing.lines() {
        // "<mode> blob <oid>\t<path>"
        let Some((meta, path)) = line.split_once('\t') else {
            continue;
        };
        let mut fields = meta.split_whitespace();
        let (_mode, kind, oid) = (fields.next(), fields.next(), fields.next());
        if kind == Some("blob")
            && let Some(oid) = oid
        {
            oids.insert(path.to_owned(), oid.to_owned());
        }
    }
    Ok(oids)
}

/// Resolves the remote's default branch head (`ls-remote --symref origin HEAD`).
fn remote_default_head(checkout: &Path) -> Result<String> {
    let listing = git_stdout(checkout, &["ls-remote", "--symref", "origin", "HEAD"])?;
    listing
        .lines()
        .find_map(|line| {
            let (sha, reference) = line.split_once('\t')?;
            (reference.trim() == "HEAD" && is_commit_sha(sha.trim())).then(|| sha.trim().to_owned())
        })
        .context("origin HEAD did not resolve to a commit")
}

fn resolve_remote_ref(checkout: &Path, reference: &str) -> Result<String> {
    let listing = git_stdout(checkout, &["ls-remote", "origin", reference])?;
    listing
        .lines()
        .find_map(|line| {
            let sha = line.split_whitespace().next()?;
            is_commit_sha(sha).then(|| sha.to_owned())
        })
        .with_context(|| format!("origin has no ref named {reference}"))
}

/// Ensures a blob-less clone of the repository exists at `checkout`.
pub(crate) fn ensure_checkout(repo: &RepositoryEntry, checkout: &Path, force: bool) -> Result<()> {
    if checkout.join(".git").exists() {
        if !force && is_dirty(checkout)? {
            bail!(
                "{} has uncommitted changes; clean it or pass --force",
                checkout.display()
            );
        }
        if force {
            run_git_command(checkout, &["reset", "--hard"])?;
            run_git_command(checkout, &["clean", "-fd"])?;
        }
        return Ok(());
    }
    if checkout.exists() {
        if !force {
            bail!(
                "{} exists but is not a git checkout; remove it or pass --force",
                checkout.display()
            );
        }
        fs::remove_dir_all(checkout)
            .with_context(|| format!("failed to clear {}", checkout.display()))?;
    }
    let parent = checkout
        .parent()
        .with_context(|| format!("missing parent for {}", checkout.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("failed to create {}", parent.display()))?;
    let name = checkout
        .file_name()
        .and_then(|name| name.to_str())
        .with_context(|| format!("invalid checkout path {}", checkout.display()))?;
    run_git_command(
        parent,
        &[
            "clone",
            "--filter=blob:none",
            "--no-checkout",
            repo.remote.as_str(),
            name,
        ],
    )
}

pub(crate) fn fetch_commit(checkout: &Path, commit: &str) -> Result<()> {
    if git_stdout(checkout, &["cat-file", "-e", commit]).is_ok() {
        return Ok(());
    }
    if run_git_command(checkout, &["fetch", "--depth", "1", "origin", commit]).is_err() {
        run_git_command(checkout, &["fetch", "origin", commit])?;
    }
    Ok(())
}

pub(crate) fn checkout_detached(checkout: &Path, commit: &str, force: bool) -> Result<()> {
    let mut args = vec!["checkout", "--detach"];
    if force {
        args.push("--force");
    }
    args.push(commit);
    run_git_command(checkout, &args)
}

fn is_dirty(checkout: &Path) -> Result<bool> {
    Ok(!git_stdout(checkout, &["status", "--porcelain"])?
        .trim()
        .is_empty())
}

fn lock_has_uncommitted_edits(source_lock: &Path) -> Result<bool> {
    let lock_dir = source_lock.parent().unwrap_or_else(|| Path::new("."));
    let name = source_lock
        .file_name()
        .and_then(|name| name.to_str())
        .context("source lock has no file name")?;
    // Outside a git checkout there is nothing to protect.
    git_stdout(lock_dir, &["status", "--porcelain", "--", name])
        .map_or_else(|_| Ok(false), |status| Ok(!status.trim().is_empty()))
}

fn select_repositories<'a>(
    repositories: &'a [RepositoryEntry],
    requested: &[String],
) -> Result<Vec<&'a RepositoryEntry>> {
    if requested.is_empty() {
        return Ok(repositories.iter().collect());
    }
    let mut selected = Vec::new();
    for id in requested {
        let repo = repositories.iter().find(|repo| &repo.id == id);
        let Some(repo) = repo else {
            let known: Vec<&str> = repositories.iter().map(|r| r.id.as_str()).collect();
            bail!(
                "unknown repository id {id}; available: {}",
                known.join(", ")
            );
        };
        selected.push(repo);
    }
    Ok(selected)
}

/// Rewrites the `commit:` line of one repository row, preserving every other
/// byte of the lock (the serde model would drop comments).
fn update_lock_commit(text: &str, repo_id: &str, new_commit: &str) -> Result<String> {
    let mut lines: Vec<&str> = text.lines().collect();
    let row_start = lines
        .iter()
        .position(|line| line.trim_end() == format!("- id: {repo_id}"))
        .with_context(|| format!("repository {repo_id} not found in the source lock"))?;
    let commit_line = lines
        .iter()
        .enumerate()
        .skip(row_start + 1)
        .take_while(|(_, line)| !line.starts_with("- id: "))
        .find(|(_, line)| line.starts_with("  commit: "))
        .map(|(index, _)| index)
        .with_context(|| format!("repository {repo_id} has no commit line"))?;
    let owned = format!("  commit: {new_commit}");
    lines[commit_line] = &owned;
    let mut updated = lines.join("\n");
    if text.ends_with('\n') {
        updated.push('\n');
    }
    Ok(updated)
}

fn is_commit_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    const LOCK: &str = "# header comment\nrepositories:\n- id: demo\n  remote: https://github.com/example/demo.git\n  commit: 1111111111111111111111111111111111111111\n  # why: demo row.\n  role: primary\n  producer_paths:\n  - a.txt # inline note\n  - b.txt\nfixtures: []\n";

    #[test]
    fn lock_commit_rewrite_preserves_comments_and_errors_on_unknown_rows() {
        let updated = update_lock_commit(LOCK, "demo", "2222222222222222222222222222222222222222")
            .expect("rewrite succeeds");
        assert!(updated.contains("# header comment"));
        assert!(updated.contains("  # why: demo row."));
        assert!(updated.contains("  - a.txt # inline note"));
        assert!(updated.contains("  commit: 2222222222222222222222222222222222222222"));
        assert!(!updated.contains("1111111111111111111111111111111111111111"));

        let error = update_lock_commit(LOCK, "absent", "2".repeat(40).as_str())
            .expect_err("unknown rows fail");
        assert!(format!("{error:#}").contains("absent"));
    }

    fn git(dir: &Path, args: &[&str]) {
        let output = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .env("GIT_AUTHOR_NAME", "t")
            .env("GIT_AUTHOR_EMAIL", "t@t")
            .env("GIT_COMMITTER_NAME", "t")
            .env("GIT_COMMITTER_EMAIL", "t@t")
            .output()
            .expect("git runs");
        assert!(
            output.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn rev(dir: &Path) -> String {
        git_stdout(dir, &["rev-parse", "HEAD"]).expect("rev-parse")
    }

    #[test]
    fn drift_rows_classify_changed_and_removed_paths() {
        let scratch = std::env::temp_dir().join(format!(
            "xtask-sync-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        let origin = scratch.join("origin");
        std::fs::create_dir_all(&origin).expect("mkdir");
        git(&origin, &["init", "--quiet"]);
        std::fs::write(origin.join("a.txt"), "one").expect("write");
        std::fs::write(origin.join("b.txt"), "keep").expect("write");
        git(&origin, &["add", "."]);
        git(&origin, &["commit", "--quiet", "-m", "c1"]);
        let pin = rev(&origin);

        std::fs::write(origin.join("a.txt"), "two").expect("write");
        git(&origin, &["rm", "--quiet", "b.txt"]);
        git(&origin, &["add", "."]);
        git(&origin, &["commit", "--quiet", "-m", "c2"]);
        let head = rev(&origin);

        let paths = vec!["a.txt".to_owned(), "b.txt".to_owned()];
        let rows = drift_rows(&origin, &paths, &pin, &head).expect("rows");
        assert_eq!(rows.len(), 2);
        assert_ne!(rows[0].old_oid, rows[0].new_oid, "a.txt changed");
        assert!(rows[0].new_oid.is_some());
        assert!(
            rows[1].old_oid.is_some() && rows[1].new_oid.is_none(),
            "b.txt removed"
        );

        let _ = std::fs::remove_dir_all(&scratch);
    }
}
