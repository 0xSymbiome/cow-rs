use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, bail};

use crate::policy::{classify_release, workspace};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Base ref used to compute the PR diff.
    #[arg(long, default_value = "main")]
    pub base_ref: String,
    /// Head ref used to classify whether the lane is a patch release.
    #[arg(long, default_value = "HEAD")]
    pub head_ref: String,
    /// Workspace Cargo.toml to read for the head side when classifying the release.
    #[arg(long)]
    pub workspace_cargo_toml: Option<PathBuf>,
    /// Force enforcement without release classification.
    #[arg(long)]
    pub enforce: bool,
    /// Override unified diff path for tests or local investigation.
    #[arg(long)]
    pub diff_file: Option<PathBuf>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChainPatchReport {
    pub added_chain_ids: BTreeSet<u64>,
    pub source_lock_changed: bool,
}

pub fn run(args: &Args) -> anyhow::Result<()> {
    let should_enforce = args.enforce || args.diff_file.is_some() || is_patch_lane(args)?;
    if !should_enforce {
        println!("chain patch eligibility skipped outside a patch release lane");
        return Ok(());
    }

    let diff = match &args.diff_file {
        Some(path) => workspace::read_to_string(path)?,
        None => git_diff(&args.repo_root, &args.base_ref)?,
    };
    let source_lock = workspace::read_to_string(&args.repo_root.join("parity/source-lock.yaml"))?;
    let errors = validate_diff(&diff, &source_lock);
    if errors.is_empty() {
        let report = analyze_diff(&diff);
        println!(
            "chain patch eligibility holds; {} added chain id(s) detected",
            report.added_chain_ids.len()
        );
        return Ok(());
    }

    for error in &errors {
        eprintln!("error: {error}");
    }
    bail!("chain patch eligibility has {} error(s)", errors.len())
}

fn is_patch_lane(args: &Args) -> anyhow::Result<bool> {
    let classification = classify_release::classify_refs(
        &args.repo_root,
        Some(&args.base_ref),
        &args.head_ref,
        args.workspace_cargo_toml.as_deref(),
    )?;
    Ok(classification.release_kind == classify_release::ReleaseKind::Patch)
}

pub fn validate_diff(diff: &str, source_lock: &str) -> Vec<String> {
    let report = analyze_diff(diff);
    let mut errors = Vec::new();
    if report.added_chain_ids.is_empty() {
        return errors;
    }
    if report.source_lock_changed {
        errors.push(
            "chain additions in a patch lane cannot be paired with a source-lock refresh"
                .to_owned(),
        );
    }
    for chain_id in report.added_chain_ids {
        if !source_lock_contains_chain_id(source_lock, chain_id) {
            errors.push(format!(
                "chain id {chain_id} is added but is not visible in the source-lock authority text"
            ));
        }
    }
    errors
}

pub fn analyze_diff(diff: &str) -> ChainPatchReport {
    let mut current_file = String::new();
    let mut added_chain_ids = BTreeSet::new();
    let mut source_lock_changed = false;

    for line in diff.lines() {
        if let Some(path) = line.strip_prefix("+++ b/") {
            current_file = path.replace('\\', "/");
            if current_file == "parity/source-lock.yaml" {
                source_lock_changed = true;
            }
            continue;
        }
        if !line.starts_with('+') || line.starts_with("+++") {
            continue;
        }
        let added = line.trim_start_matches('+').trim();
        match current_file.as_str() {
            "crates/core/src/config/chains.rs" => {
                if let Some(chain_id) = parse_supported_chain_variant(added) {
                    added_chain_ids.insert(chain_id);
                }
            }
            "crates/contracts/registry.toml" => {
                if let Some(chain_id) = parse_registry_chain_id(added) {
                    added_chain_ids.insert(chain_id);
                }
            }
            "parity/source-lock.yaml" => {
                source_lock_changed = true;
            }
            _ => {}
        }
    }

    ChainPatchReport {
        added_chain_ids,
        source_lock_changed,
    }
}

fn parse_supported_chain_variant(line: &str) -> Option<u64> {
    let (_, value) = line.split_once('=')?;
    let value = value.trim().trim_end_matches(',');
    parse_u64_literal(value)
}

fn parse_registry_chain_id(line: &str) -> Option<u64> {
    let (key, value) = line.split_once('=')?;
    if key.trim() != "chain_id" {
        return None;
    }
    parse_u64_literal(value.trim())
}

fn parse_u64_literal(value: &str) -> Option<u64> {
    value.replace('_', "").parse::<u64>().ok()
}

fn source_lock_contains_chain_id(source_lock: &str, chain_id: u64) -> bool {
    let compact = chain_id.to_string();
    let underscored = underscore_chain_id(chain_id);
    source_lock.contains(&format!("chain_id = {compact}"))
        || source_lock.contains(&format!("chain_id: {compact}"))
        || source_lock.contains(&format!("chainId: {compact}"))
        || source_lock.contains(&format!("chain_id = {underscored}"))
        || source_lock.contains(&format!("chain_id: {underscored}"))
        || source_lock.contains(&format!("chainId: {underscored}"))
}

fn underscore_chain_id(chain_id: u64) -> String {
    let raw = chain_id.to_string();
    let mut out = String::new();
    for (index, ch) in raw.chars().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            out.push('_');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

fn git_diff(repo_root: &Path, base_ref: &str) -> anyhow::Result<String> {
    let output = Command::new("git")
        .current_dir(repo_root)
        .args([
            "diff",
            "--unified=0",
            &format!("{base_ref}...HEAD"),
            "--",
            "crates/core/src/config/chains.rs",
            "crates/contracts/registry.toml",
            "parity/source-lock.yaml",
        ])
        .output()
        .context("failed to invoke git diff")?;
    if !output.status.success() {
        bail!(
            "git diff failed while reading chain patch eligibility: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    String::from_utf8(output.stdout).context("git diff output was not UTF-8")
}
