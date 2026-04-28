use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Write},
    path::PathBuf,
};

use anyhow::{Context, bail};
use serde::Deserialize;

use crate::{
    diagnostics::{Diagnostic, OutputMode},
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
    #[allow(dead_code)]
    pub line: Option<u32>,
    pub category: String,
    pub expected_marker: String,
    pub reason: String,
}

pub fn run(args: Args, output_mode: OutputMode) -> anyhow::Result<()> {
    let mut stdout = io::stdout().lock();
    run_with_writer(args, output_mode, &mut stdout)
}

pub fn run_with_writer(
    args: Args,
    output_mode: OutputMode,
    writer: &mut impl Write,
) -> anyhow::Result<()> {
    let manifest_path = args
        .manifest
        .unwrap_or_else(|| args.repo_root.join(".github/config/enum-policy.yaml"));
    let policy: EnumPolicy = fixtures::load_yaml(&manifest_path)
        .with_context(|| format!("failed to load {}", manifest_path.display()))?;
    let discovered = workspace::collect_public_enums(&args.repo_root)?;
    let errors = validate_policy(&policy, &discovered);

    if errors.is_empty() {
        Diagnostic::info(
            "PM2000",
            format!("enum policy covers {} public enums", discovered.len()),
        )
        .emit(output_mode, writer)?;
        return Ok(());
    }

    for error in &errors {
        Diagnostic::error("PM2001", error).emit(output_mode, writer)?;
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
        let key = (normalize_manifest_path(&entry.file), entry.name.clone());
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

fn normalize_manifest_path(path: &str) -> String {
    path.replace('\\', "/")
}
