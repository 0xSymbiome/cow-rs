//! Rejects shell scripts (`*.sh`, `*.ps1`) outside the allowed lanes.
//!
//! Repository tooling is Rust behind the `cargo xtask` aliases; shell stays
//! confined to git hooks (`.githooks/`) and the npm package's build scripts
//! (`crates/wasm/npm/scripts/`). New shell wrappers anywhere else must become a
//! cargo alias instead.

use std::{path::PathBuf, process::Command};

use anyhow::{Context, Result, bail};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

/// Lanes where shell tooling is permitted.
const ALLOWED_PREFIXES: &[&str] = &[".githooks/", "crates/wasm/npm/scripts/"];

pub fn run_default() -> Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
    })
}

pub fn run(args: &Args) -> Result<()> {
    let output = Command::new("git")
        .current_dir(&args.repo_root)
        .args(["ls-files", "*.sh", "*.ps1"])
        .output()
        .context("failed to invoke git ls-files")?;
    if !output.status.success() {
        bail!(
            "git ls-files failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let listing = String::from_utf8(output.stdout).context("git ls-files output was not UTF-8")?;
    let forbidden = forbidden_scripts(&listing);

    if forbidden.is_empty() {
        println!("no shell scripts outside the allowed lanes");
        return Ok(());
    }
    for path in &forbidden {
        eprintln!("error: forbidden shell script: {path}");
    }
    bail!(
        "{} shell script(s) outside the allowed lanes; use a cargo xtask alias instead",
        forbidden.len()
    )
}

/// A tracked `*.sh`/`*.ps1` is a stray unless it lives in an allowed lane or
/// under a hidden top-level directory (separately-governed tooling and scratch).
fn forbidden_scripts(listing: &str) -> Vec<&str> {
    listing
        .lines()
        .map(str::trim)
        .filter(|path| !path.is_empty())
        .filter(|path| {
            !ALLOWED_PREFIXES
                .iter()
                .any(|prefix| path.starts_with(prefix))
        })
        .filter(|path| !is_hidden_top_level(path))
        .collect()
}

fn is_hidden_top_level(path: &str) -> bool {
    path.split('/')
        .next()
        .is_some_and(|first| first.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use super::forbidden_scripts;

    #[test]
    fn allowed_and_hidden_lanes_pass_and_shipped_strays_fail() {
        let listing = ".githooks/commit-msg\ncrates/wasm/npm/scripts/build.sh\n.private-tooling/dev.ps1\nscripts/rogue.sh\ntools/setup.ps1\n";
        assert_eq!(
            forbidden_scripts(listing),
            ["scripts/rogue.sh", "tools/setup.ps1"]
        );
    }
}
