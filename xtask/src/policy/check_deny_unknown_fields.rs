use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::policy::{
    fixtures,
    workspace::{self, DenyUnknownFields},
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Override allowlist path.
    #[arg(long)]
    pub allowlist: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct DenyUnknownFieldsAllowlist {
    pub version: u32,
    pub allowed: Vec<DenyUnknownFieldsEntry>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DenyUnknownFieldsEntry {
    pub file: String,
    pub item: String,
    pub reason: String,
}

pub fn run_default() -> anyhow::Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
        allowlist: None,
    })
}

pub fn run(args: &Args) -> anyhow::Result<()> {
    let allowlist_path = args.allowlist.clone().unwrap_or_else(|| {
        args.repo_root
            .join(".github/config/deny-unknown-fields-allowlist.yaml")
    });
    let allowlist: DenyUnknownFieldsAllowlist = fixtures::load_yaml(&allowlist_path)
        .with_context(|| format!("failed to load {}", allowlist_path.display()))?;
    let occurrences = workspace::collect_deny_unknown_fields(&args.repo_root)?;
    let errors = validate_allowlist(&allowlist, &occurrences);

    if errors.is_empty() {
        println!(
            "deny_unknown_fields allowlist covers {} occurrence(s)",
            occurrences.len()
        );
        return Ok(());
    }

    for error in &errors {
        eprintln!("error: {error}");
    }
    bail!(
        "deny_unknown_fields allowlist has {} error(s)",
        errors.len()
    )
}

pub fn validate_allowlist(
    allowlist: &DenyUnknownFieldsAllowlist,
    occurrences: &[DenyUnknownFields],
) -> Vec<String> {
    let mut errors = Vec::new();
    if allowlist.version != 1 {
        errors.push(format!(
            "deny-unknown-fields-allowlist.yaml version must be 1, got {}",
            allowlist.version
        ));
    }

    let mut allowed = BTreeMap::new();
    for entry in &allowlist.allowed {
        if entry.reason.trim().is_empty() {
            errors.push(format!(
                "{}::{} has an empty deny_unknown_fields rationale",
                entry.file, entry.item
            ));
        }
        let key = (workspace::normalize_manifest_path(&entry.file), entry.item.clone());
        if allowed.insert(key.clone(), entry).is_some() {
            errors.push(format!(
                "duplicate deny_unknown_fields allowlist entry for {}::{}",
                key.0, key.1
            ));
        }
    }

    let mut matched = BTreeSet::new();
    for occurrence in occurrences {
        let key = (occurrence.file.clone(), occurrence.item.clone());
        if allowed.contains_key(&key) {
            matched.insert(key);
        } else {
            errors.push(format!(
                "serde(deny_unknown_fields) on {}::{} is not allowlisted",
                occurrence.file, occurrence.item
            ));
        }
    }

    for key in allowed.keys() {
        if !matched.contains(key) {
            errors.push(format!(
                "deny_unknown_fields allowlist entry {}::{} has no matching item",
                key.0, key.1
            ));
        }
    }

    errors
}
