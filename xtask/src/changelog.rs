//! `cargo changelog` — regenerate CHANGELOG.md from the conventional-commit
//! history with git-cliff.
//!
//! Cargo-alias replacement for a shell wrapper (the `check-shell-wrappers`
//! policy keeps repository tooling in Rust behind cargo aliases). It resolves
//! the workspace root itself, so the changelog is always written at the repo
//! root regardless of the caller's cwd — which lets cargo-release call it from a
//! per-crate working directory.
//!
//! The changelog is fully derived from commit history: each run rebuilds the
//! whole file from the conventional commits, so it is never hand-maintained.

use std::{path::PathBuf, process::Command};

use anyhow::{Context, Result, bail};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Render the commits under this release tag (e.g. `v0.1.0-alpha.1`) and
    /// rewrite CHANGELOG.md. Omit to print the pending section to stdout without
    /// touching any file.
    #[arg(long)]
    pub tag: Option<String>,
    /// Repository root. Defaults to the git toplevel.
    #[arg(long)]
    pub repo_root: Option<PathBuf>,
}

pub fn run(args: &Args) -> Result<()> {
    let root = match &args.repo_root {
        Some(root) => root.clone(),
        None => git_toplevel()?,
    };
    let changelog = root.join("CHANGELOG.md");

    let mut cmd = Command::new("git-cliff");
    cmd.current_dir(&root)
        .arg("--config")
        .arg(root.join("cliff.toml"));

    match &args.tag {
        // Release path: rebuild the whole changelog with the new tag.
        Some(tag) => {
            cmd.arg("--tag").arg(tag).arg("--output").arg(&changelog);
        }
        // Preview path: print the pending section to stdout, touch no files.
        None => {
            cmd.arg("--unreleased");
        }
    }

    let status = cmd
        .status()
        .context("failed to run git-cliff; install it with `cargo install git-cliff`")?;
    if !status.success() {
        bail!("git-cliff exited with {status}");
    }
    if args.tag.is_some() {
        println!("regenerated {}", changelog.display());
    }
    Ok(())
}

/// Resolves the repository root via `git rev-parse --show-toplevel`.
fn git_toplevel() -> Result<PathBuf> {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to invoke git rev-parse")?;
    if !output.status.success() {
        bail!(
            "git rev-parse --show-toplevel failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let path = String::from_utf8(output.stdout).context("git rev-parse output was not UTF-8")?;
    Ok(PathBuf::from(path.trim()))
}
