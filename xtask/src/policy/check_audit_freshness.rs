//! `check-audit-freshness` — a non-blocking advisory (never fails CI).
//!
//! Given a base git ref, it reports when a change touches a path mapped to an
//! audit in `.github/config/audit-refresh-map.yml` without that audit's file
//! being part of the same change (a proxy for "the `timestamp` was not
//! refreshed"). It is intentionally informational: it always exits `0`, and it
//! is excluded from the blocking `cargo check-policies` sweep (it needs a
//! pull-request diff, like `check-chain-patch-eligibility`).

use std::{collections::BTreeSet, path::Path, path::PathBuf, process::Command};

use anyhow::Result;

use crate::policy::check_audit_lane::{entry_slug, load_refresh_map};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Git ref to diff the working tree against (for example `origin/main`).
    #[arg(long)]
    pub base: String,
}

pub fn run(args: &Args) -> Result<()> {
    let map = match load_refresh_map(&args.repo_root) {
        Ok(map) => map,
        Err(error) => {
            println!("check-audit-freshness: skipped ({error})");
            return Ok(());
        }
    };
    let Some(changed) = changed_paths(&args.repo_root, &args.base) else {
        println!(
            "check-audit-freshness: skipped (could not diff against {:?})",
            args.base
        );
        return Ok(());
    };

    let mut advisories = Vec::new();
    for entry in &map.entries {
        let Some(slug) = entry_slug(entry) else {
            continue;
        };
        let audit_path = format!("docs/audit/{slug}-audit.md");
        let audit_touched = changed.contains(&audit_path);
        let touched_trigger = entry
            .refresh_triggers
            .paths
            .iter()
            .find(|trigger| changed.iter().any(|path| path_matches(path, trigger)));
        match touched_trigger {
            Some(trigger) if !audit_touched => advisories.push(format!(
                "{audit_path}: `{trigger}` changed (surface: {}) but the audit was not refreshed",
                entry.owning_surface
            )),
            _ => {}
        }
    }

    if advisories.is_empty() {
        println!("check-audit-freshness: no mapped audit needs a refresh for this change.");
    } else {
        println!(
            "check-audit-freshness: {} audit(s) may need a refresh (advisory only):",
            advisories.len()
        );
        for advisory in &advisories {
            println!("  - {advisory}");
        }
    }
    Ok(())
}

/// A changed path matches a trigger when it equals the trigger or lies under it.
fn path_matches(path: &str, trigger: &str) -> bool {
    path == trigger
        || path
            .strip_prefix(trigger)
            .is_some_and(|rest| rest.starts_with('/'))
}

fn changed_paths(repo_root: &Path, base: &str) -> Option<BTreeSet<String>> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args(["diff", "--name-only", base])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_owned)
            .collect(),
    )
}
