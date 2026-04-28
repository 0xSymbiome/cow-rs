#![allow(dead_code)]

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{Context, Result, bail};

pub fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_parity-maintainer"))
}

pub struct RepoSpec<'a> {
    pub id: &'a str,
    pub remote: String,
    pub commit: String,
    pub role: &'a str,
    pub producer_paths: Vec<&'a str>,
}

pub fn write_source_lock(
    path: &Path,
    generated_at_utc: &str,
    repos: &[RepoSpec<'_>],
) -> Result<()> {
    let mut yaml = format!(
        "meta:\n  schema_version: 3\n  generated_at_utc: {}\n  purpose: test source lock\nrepositories:\n",
        quote(generated_at_utc)
    );
    for repo in repos {
        yaml.push_str(&format!(
            "- id: {}\n  remote: {}\n  commit: {}\n  role: {}\n  optional_local_path: {}\n  producer_paths:\n",
            quote(repo.id),
            quote(&repo.remote),
            quote(&repo.commit),
            quote(repo.role),
            quote("<test-checkout>")
        ));
        for path in &repo.producer_paths {
            yaml.push_str(&format!("  - {}\n", quote(path)));
        }
    }
    yaml.push_str(
        "fixtures: []\nvalidation:\n  standalone_repo_contract: []\n  repo_local_publication_contract: []\n  pinned_upstream_provenance_contract: []\n  maintainer_refresh_contract: []\n",
    );
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(path, yaml).with_context(|| format!("failed to write {}", path.display()))
}

pub fn init_git_repo(path: &Path, remote: Option<&str>) -> Result<()> {
    fs::create_dir_all(path).with_context(|| format!("failed to create {}", path.display()))?;
    run_git(path, &["init"])?;
    run_git(path, &["config", "user.email", "tests@example.com"])?;
    run_git(path, &["config", "user.name", "Parity Maintainer Tests"])?;
    if let Some(remote) = remote {
        run_git(path, &["remote", "add", "origin", remote])?;
    }
    Ok(())
}

pub fn commit_all(path: &Path, message: &str) -> Result<String> {
    run_git(path, &["add", "."])?;
    run_git(path, &["commit", "-m", message])?;
    git_stdout(path, &["rev-parse", "HEAD"])
}

pub fn run_git(path: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
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

pub fn git_stdout(path: &Path, args: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(path)
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

pub fn output_text(output: &Output) -> String {
    format!(
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

pub fn write_file(path: impl Into<PathBuf>, contents: &str) -> Result<()> {
    let path = path.into();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    fs::write(&path, contents).with_context(|| format!("failed to write {}", path.display()))
}

fn quote(input: &str) -> String {
    format!("'{}'", input.replace('\'', "''"))
}
