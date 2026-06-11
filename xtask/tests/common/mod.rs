#![allow(
    dead_code,
    reason = "shared helpers across separate integration-test binaries; each binary uses a subset"
)]

use std::{
    fmt::Write as _,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use anyhow::{Context, Result, bail};

pub fn command() -> Command {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
}

pub struct RepoSpec<'a> {
    pub id: &'a str,
    pub remote: String,
    pub commit: String,
    pub producer_paths: Vec<&'a str>,
}

pub fn write_source_lock(path: &Path, repos: &[RepoSpec<'_>]) -> Result<()> {
    let mut yaml = String::from("repositories:\n");
    for repo in repos {
        write!(
            yaml,
            "- id: {}\n  remote: {}\n  commit: {}\n  producer_paths:\n",
            quote(repo.id),
            quote(&repo.remote),
            quote(&repo.commit),
        )
        .expect("writing to a String is infallible");
        for path in &repo.producer_paths {
            writeln!(yaml, "  - {}", quote(path)).expect("writing to a String is infallible");
        }
    }
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

// Shared scratch-directory helper for the policy-check suites.
use std::time::{SystemTime, UNIX_EPOCH};

pub struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub fn new(name: &str) -> Self {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "xtask-policy-{name}-{}-{nanos}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temp directory should be created");
        Self { path }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn write(&self, relative: &str, content: &str) {
        let path = self.path.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("parent directory should be created");
        }
        fs::write(path, content).expect("fixture should be written");
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}
