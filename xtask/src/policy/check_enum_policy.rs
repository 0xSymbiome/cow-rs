use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::policy::{
    fixtures,
    workspace::{self, PublicEnum},
};

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
    /// Override manifest path for tests or local investigation.
    #[arg(long)]
    pub manifest: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
pub struct EnumPolicy {
    pub version: u32,
    pub enums: Vec<EnumPolicyEntry>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EnumPolicyEntry {
    pub name: String,
    pub file: String,
    pub category: String,
    pub expected_marker: String,
    pub reason: String,
}

pub fn run_default() -> anyhow::Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
        manifest: None,
    })
}

pub fn run(args: &Args) -> anyhow::Result<()> {
    let manifest_path = args
        .manifest
        .clone()
        .unwrap_or_else(|| args.repo_root.join(".github/config/enum-policy.yaml"));
    let policy: EnumPolicy = fixtures::load_yaml(&manifest_path)
        .with_context(|| format!("failed to load {}", manifest_path.display()))?;
    let discovered = workspace::collect_public_enums(&args.repo_root)?;
    let errors = validate_policy(&policy, &discovered);

    if errors.is_empty() {
        println!("enum policy covers {} public enums", discovered.len());
        return Ok(());
    }

    for error in &errors {
        eprintln!("error: {error}");
    }
    bail!("enum policy has {} error(s)", errors.len())
}

pub fn validate_policy(policy: &EnumPolicy, discovered: &[PublicEnum]) -> Vec<String> {
    let mut errors = Vec::new();
    if policy.version != 1 {
        errors.push(format!(
            "enum-policy.yaml version must be 1, got {}",
            policy.version
        ));
    }

    let mut manifest = BTreeMap::new();
    for entry in &policy.enums {
        if entry.reason.trim().is_empty() {
            errors.push(format!(
                "{}::{} has an empty enum policy rationale",
                entry.file, entry.name
            ));
        }
        if !matches!(
            entry.category.as_str(),
            "protocol-fixed-exhaustive" | "upstream-growing" | "sdk-local-state" | "private-leak"
        ) {
            errors.push(format!(
                "{}::{} has unknown category `{}`",
                entry.file, entry.name, entry.category
            ));
        }
        let key = (workspace::normalize_manifest_path(&entry.file), entry.name.clone());
        if manifest.insert(key.clone(), entry).is_some() {
            errors.push(format!(
                "duplicate enum policy entry for {}::{}",
                key.0, key.1
            ));
        }
    }

    let mut seen = BTreeSet::new();
    for item in discovered {
        let key = (item.file.clone(), item.name.clone());
        seen.insert(key.clone());
        let Some(entry) = manifest.get(&key) else {
            errors.push(format!(
                "missing enum policy entry for {}::{}",
                item.file, item.item
            ));
            continue;
        };
        match entry.expected_marker.as_str() {
            "non_exhaustive" if !item.is_non_exhaustive => errors.push(format!(
                "{}::{} must carry #[non_exhaustive]",
                item.file, item.item
            )),
            "exhaustive" if item.is_non_exhaustive => errors.push(format!(
                "{}::{} must remain exhaustive but carries #[non_exhaustive]",
                item.file, item.item
            )),
            "non_exhaustive" | "exhaustive" => {}
            other => errors.push(format!(
                "{}::{} has unknown expected_marker `{other}`",
                entry.file, entry.name
            )),
        }
    }

    for key in manifest.keys() {
        if !seen.contains(key) {
            errors.push(format!(
                "enum policy entry {}::{} has no matching public enum",
                key.0, key.1
            ));
        }
    }
    errors
}
