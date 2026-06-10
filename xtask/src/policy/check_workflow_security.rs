//! Workflow hardening checks over `.github/workflows/*.yml`.
//!
//! Two invariants:
//!
//! 1. Every external `uses:` action reference is pinned to a 40-character
//!    commit SHA (local `./` references are exempt).
//! 2. Any workflow with a `pull_request_target` trigger carries an explicit
//!    `# allow-pull-request-target: <rationale>` comment, so the elevated-token
//!    trigger is a reviewed decision rather than an accident.

use std::{fs, path::PathBuf};

use anyhow::{Context, Result, bail};
use regex::Regex;

use crate::policy::workspace;

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

pub fn run_default() -> Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
    })
}

pub fn run(args: &Args) -> Result<()> {
    let workflows = args.repo_root.join(".github/workflows");
    let mut paths = Vec::new();
    if workflows.is_dir() {
        for entry in fs::read_dir(&workflows)
            .with_context(|| format!("failed to read {}", workflows.display()))?
        {
            let path = entry?.path();
            if path.extension().is_some_and(|ext| ext == "yml") {
                paths.push(path);
            }
        }
    }
    paths.sort();

    let mut errors = Vec::new();
    for path in &paths {
        let text = workspace::read_to_string(path)?;
        let relative = workspace::relative_path(&args.repo_root, path);
        errors.extend(audit_workflow(&relative, &text));
    }

    if errors.is_empty() {
        println!(
            "workflow security invariants hold across {} workflow(s)",
            paths.len()
        );
        return Ok(());
    }
    for error in &errors {
        eprintln!("error: {error}");
    }
    bail!("workflow security has {} violation(s)", errors.len())
}

fn audit_workflow(relative: &str, text: &str) -> Vec<String> {
    // Compiled once per call; the workflow set is small.
    let uses = Regex::new(r"^\s*(?:-\s*)?uses:\s*(\S+)").expect("uses regex is valid");
    let sha_pinned = Regex::new(r"@[0-9a-f]{40}$").expect("sha regex is valid");
    let pull_request_target =
        Regex::new(r"^\s*(?:-\s*)?pull_request_target\s*:|^\s*on:[^#]*pull_request_target")
            .expect("pull_request_target regex is valid");
    let allow_comment =
        Regex::new(r"#\s*allow-pull-request-target:\s*.+").expect("allow regex is valid");

    let mut errors = Vec::new();
    let mut has_pull_request_target = false;
    let mut has_allow_comment = false;

    for (index, line) in text.lines().enumerate() {
        if let Some(capture) = uses.captures(line) {
            let reference = capture[1].split('#').next().unwrap_or_default().trim();
            if !reference.starts_with("./") && !sha_pinned.is_match(reference) {
                errors.push(format!(
                    "{relative}:{}: action ref must be pinned to a 40-character commit SHA: {reference}",
                    index + 1
                ));
            }
        }
        if pull_request_target.is_match(line) {
            has_pull_request_target = true;
        }
        if allow_comment.is_match(line) {
            has_allow_comment = true;
        }
    }

    if has_pull_request_target && !has_allow_comment {
        errors.push(format!(
            "{relative}: pull_request_target requires an explicit '# allow-pull-request-target: <rationale>' comment"
        ));
    }
    errors
}

#[cfg(test)]
mod tests {
    use super::audit_workflow;

    #[test]
    fn unpinned_ref_and_unreviewed_pull_request_target_are_rejected() {
        let workflow = "on:\n  pull_request_target:\njobs:\n  build:\n    steps:\n      - uses: actions/checkout@v4\n";
        let errors = audit_workflow("ci.yml", workflow);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("40-character commit SHA"))
        );
        assert!(
            errors
                .iter()
                .any(|error| error.contains("allow-pull-request-target"))
        );
    }

    #[test]
    fn pinned_ref_and_local_ref_and_reviewed_trigger_pass() {
        let workflow = "# allow-pull-request-target: reviewed for the fork-label gate\non:\n  pull_request_target:\njobs:\n  build:\n    steps:\n      - uses: actions/checkout@34e114876b0b11c390a56381ad16ebd13914f8d5\n      - uses: ./.github/workflows/_quality-gate.yml\n";
        assert!(audit_workflow("ci.yml", workflow).is_empty());
    }
}
