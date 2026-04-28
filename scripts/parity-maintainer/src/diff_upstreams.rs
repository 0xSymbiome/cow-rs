use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use clap::Args;

use crate::{RepositoryEntry, load_source_lock};

#[derive(Debug, Args)]
pub(crate) struct DiffUpstreamsArgs {
    #[arg(long, default_value = crate::DEFAULT_SOURCE_LOCK)]
    source_lock: PathBuf,
    #[arg(long)]
    output: PathBuf,
}

struct DiffRow {
    repo: String,
    path: String,
    status: String,
    classification: &'static str,
    summary: String,
}

pub(crate) fn run(args: DiffUpstreamsArgs) -> Result<()> {
    let lock = load_source_lock(&args.source_lock)?;
    let mut rows = Vec::new();

    for repo in &lock.repositories {
        let head = upstream_head(repo)?;
        let checkout = ScratchCheckout::fetch(repo, &head)?;
        let diff_rows = diff_repo(repo, checkout.path(), &head)?;
        rows.extend(diff_rows);
    }

    write_report(&args.output, &rows)?;
    println!("wrote upstream diff report to {}", args.output.display());
    Ok(())
}

fn upstream_head(repo: &RepositoryEntry) -> Result<String> {
    let head = git_output(Path::new("."), &["ls-remote", repo.remote.as_str(), "HEAD"])
        .or_else(|_| {
            git_output(
                Path::new("."),
                &["ls-remote", repo.remote.as_str(), "refs/heads/main"],
            )
        })
        .with_context(|| format!("failed to query upstream HEAD for {}", repo.id))?;

    head.lines()
        .next()
        .and_then(|line| line.split_whitespace().next())
        .map(ToOwned::to_owned)
        .with_context(|| format!("upstream HEAD for {} returned no commit", repo.id))
}

fn diff_repo(repo: &RepositoryEntry, checkout: &Path, head: &str) -> Result<Vec<DiffRow>> {
    let mut args = vec!["diff", "--name-status", repo.commit.as_str(), head, "--"];
    args.extend(repo.producer_paths.iter().map(String::as_str));
    let status = git_output(checkout, &args)?;
    if status.trim().is_empty() {
        return Ok(vec![DiffRow {
            repo: repo.id.clone(),
            path: "(no producer-path changes)".to_string(),
            status: "-".to_string(),
            classification: "irrelevant",
            summary: "source-lock producer paths match upstream HEAD".to_string(),
        }]);
    }

    status
        .lines()
        .map(|line| {
            let mut parts = line.split_whitespace();
            let status = parts.next().unwrap_or("-").to_string();
            let path = parts.last().unwrap_or("(unknown)").to_string();
            let patch = git_output(checkout, &["diff", repo.commit.as_str(), head, "--", &path])
                .unwrap_or_default();
            let classification = classify_change(&path, &status, &patch);
            Ok(DiffRow {
                repo: repo.id.clone(),
                path,
                status,
                classification,
                summary: classification_summary(classification).to_string(),
            })
        })
        .collect()
}

fn classify_change(path: &str, status: &str, patch: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if is_irrelevant_path(&lower) {
        return "irrelevant";
    }
    if status.starts_with('D') || status.starts_with('R') {
        return "release-blocking";
    }
    if lower.contains("deployment") || lower.contains("settlement") || lower.contains("vault") {
        return "release-blocking";
    }
    if lower.contains("openapi") {
        if patch_has_only_additions(patch) {
            return "additive-safe";
        }
        return "requires-update";
    }
    if lower.contains("types")
        || lower.contains("order")
        || lower.contains("request")
        || lower.contains("api")
    {
        return "requires-update";
    }
    "requires-update"
}

fn is_irrelevant_path(path: &str) -> bool {
    path.ends_with(".md")
        || path.ends_with(".txt")
        || path.contains("/docs/")
        || path.contains("/test/")
        || path.contains(".test.")
        || path.contains(".spec.")
}

fn patch_has_only_additions(patch: &str) -> bool {
    let mut saw_addition = false;
    for line in patch.lines() {
        if line.starts_with("+++") || line.starts_with("---") {
            continue;
        }
        if line.starts_with('+') {
            saw_addition = true;
        } else if line.starts_with('-') {
            return false;
        }
    }
    saw_addition
}

fn classification_summary(classification: &str) -> &'static str {
    match classification {
        "irrelevant" => "no SDK-facing parity refresh required",
        "additive-safe" => {
            "additive producer change; refresh fixtures or inventories on the next parity pass"
        }
        "requires-update" => "producer path changed; inspect and refresh parity artifacts",
        "release-blocking" => {
            "potential wire or deployment contract change; block release until triaged"
        }
        _ => "unclassified upstream diff",
    }
}

fn write_report(path: &Path, rows: &[DiffRow]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }

    let mut report = String::from(
        "# Upstream Diff Report\n\n| repo | path | status | classification | summary |\n| --- | --- | --- | --- | --- |\n",
    );
    for row in rows {
        report.push_str(&format!(
            "| {} | {} | {} | {} | {} |\n",
            row.repo, row.path, row.status, row.classification, row.summary
        ));
    }
    fs::write(path, report).with_context(|| format!("failed to write {}", path.display()))
}

struct ScratchCheckout {
    path: PathBuf,
}

impl ScratchCheckout {
    fn fetch(repo: &RepositoryEntry, head: &str) -> Result<Self> {
        let path = std::env::temp_dir().join(format!(
            "parity-maintainer-diff-{}-{}",
            repo.id,
            unique_suffix()
        ));
        fs::create_dir_all(&path)
            .with_context(|| format!("failed to create {}", path.display()))?;
        git(&path, &["init"])?;
        git(&path, &["remote", "add", "origin", repo.remote.as_str()])?;
        fetch_commit(&path, repo.commit.as_str())?;
        fetch_commit(&path, head)?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for ScratchCheckout {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn fetch_commit(path: &Path, commit: &str) -> Result<()> {
    git(path, &["fetch", "--depth", "1", "origin", commit]).or_else(|_| {
        git(path, &["fetch", "origin", commit])
            .or_else(|_| git(path, &["fetch", "origin"]))
            .with_context(|| format!("failed to fetch commit {commit}"))
    })
}

fn git(path: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git in {}", path.display()))?;
    if !output.status.success() {
        bail!(
            "git command failed in {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(())
}

fn git_output(path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .current_dir(path)
        .args(args)
        .output()
        .with_context(|| format!("failed to run git in {}", path.display()))?;
    if !output.status.success() {
        bail!(
            "git command failed in {}: {}",
            path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    Ok(String::from_utf8(output.stdout)?.trim().to_string())
}

fn unique_suffix() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos()
}
