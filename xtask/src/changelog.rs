//! `cargo changelog` — maintain CHANGELOG.md from the conventional-commit
//! history with git-cliff.
//!
//! Cargo-alias replacement for a shell wrapper (the `check-shell-wrappers`
//! policy keeps repository tooling in Rust behind cargo aliases). It resolves
//! the workspace root itself, so the changelog is always written at the repo
//! root regardless of the caller's cwd — which lets cargo-release call it from a
//! per-crate working directory.
//!
//! On a release, it renders only the commits since the previous tag (git-cliff
//! `--strip all`, so no header/footer) and splices that section into the
//! existing file directly above the most recent version. Prior sections — the
//! title header, the `## [Unreleased]` placeholder, and the hand-written
//! first-release section — are preserved exactly; the file is never rebuilt from
//! full history. We splice the section ourselves rather than use git-cliff's
//! `--prepend`, which re-emits the configured header and would duplicate the
//! `# Changelog` title.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result, bail};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Insert a section for the commits since the previous tag under this
    /// release tag (e.g. `v0.1.0-alpha.2`) into CHANGELOG.md, above the latest
    /// version. Omit to print the pending section to stdout without touching any
    /// file.
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
    let cliff_config = root.join("cliff.toml");

    if let Some(tag) = &args.tag {
        // Release path: render only the new section and splice it in ourselves.
        let section = render_section(&root, &cliff_config, tag)?;
        let changelog = root.join("CHANGELOG.md");
        let existing = fs::read_to_string(&changelog)
            .with_context(|| format!("failed to read {}", changelog.display()))?;
        let updated = splice_section(&existing, &section)?;
        fs::write(&changelog, updated)
            .with_context(|| format!("failed to write {}", changelog.display()))?;
        println!("inserted the {tag} section into {}", changelog.display());
    } else {
        // Preview path: print the pending section to stdout, touch no files.
        let status = Command::new("git-cliff")
            .current_dir(&root)
            .arg("--config")
            .arg(&cliff_config)
            .arg("--unreleased")
            .status()
            .context("failed to run git-cliff; install it with `cargo install git-cliff`")?;
        if !status.success() {
            bail!("git-cliff exited with {status}");
        }
    }
    Ok(())
}

/// Renders the commits since the previous tag as a single dated version section,
/// stripped of the configured header and footer.
fn render_section(root: &Path, cliff_config: &Path, tag: &str) -> Result<String> {
    let output = Command::new("git-cliff")
        .current_dir(root)
        .arg("--config")
        .arg(cliff_config)
        .arg("--unreleased")
        .arg("--tag")
        .arg(tag)
        .arg("--strip")
        .arg("all")
        .output()
        .context("failed to run git-cliff; install it with `cargo install git-cliff`")?;
    if !output.status.success() {
        bail!(
            "git-cliff exited with {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr).trim()
        );
    }
    let section = String::from_utf8(output.stdout)
        .context("git-cliff output was not UTF-8")?
        .trim()
        .to_string();
    if section.is_empty() {
        bail!("git-cliff produced no changelog entries for {tag}; nothing to release?");
    }
    Ok(section)
}

/// Splices a freshly rendered `section` into `existing` directly above the most
/// recent version entry (the first `## [x.y.z]` heading), leaving the title
/// header, any `## [Unreleased]` line, and all prior sections intact.
fn splice_section(existing: &str, section: &str) -> Result<String> {
    // The first heading that begins a dated version section starts with "## ["
    // followed by a digit; "## [Unreleased]" is intentionally skipped so the new
    // section lands below it.
    let anchor = existing.match_indices("## [").find_map(|(idx, _)| {
        existing[idx + 4..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
            .then_some(idx)
    });

    let Some(idx) = anchor else {
        bail!(
            "no existing version section (`## [x.y.z]`) found in CHANGELOG.md; \
             the first release section must be in place before incremental \
             generation is enabled"
        );
    };

    let mut out = String::with_capacity(existing.len() + section.len() + 2);
    out.push_str(&existing[..idx]);
    out.push_str(section);
    out.push_str("\n\n");
    out.push_str(&existing[idx..]);
    Ok(out)
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

#[cfg(test)]
mod tests {
    use super::splice_section;

    const EXISTING: &str = "\
# Changelog

All notable changes to `cow-rs` will be documented in this file.

## [Unreleased]

## [0.1.0-alpha.1] - 2026-06-15

### Added

- The first functional release.
";

    #[test]
    fn inserts_new_section_below_unreleased_and_above_latest_version() {
        let section = "## [0.1.0-alpha.2] - 2026-07-01\n\n### Bug Fixes\n\n- A fix.";
        let out = splice_section(EXISTING, section).unwrap();

        // Title header appears exactly once — no duplication.
        assert_eq!(out.matches("# Changelog").count(), 1);
        assert_eq!(out.matches("All notable changes").count(), 1);

        // The new section sits between [Unreleased] and the prior version.
        let unreleased = out.find("## [Unreleased]").unwrap();
        let new = out.find("## [0.1.0-alpha.2]").unwrap();
        let prior = out.find("## [0.1.0-alpha.1]").unwrap();
        assert!(unreleased < new && new < prior);

        // The hand-written first-release content is preserved verbatim.
        assert!(out.contains("- The first functional release."));
    }

    #[test]
    fn errors_when_no_version_section_exists() {
        let bare = "# Changelog\n\n## [Unreleased]\n";
        assert!(splice_section(bare, "## [9.9.9] - 2026-07-01\n\n### x\n\n- y").is_err());
    }
}
