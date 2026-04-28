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
    workspace::{self, PanicCall},
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
pub struct PanicAllowlist {
    pub version: u32,
    pub allowed: Vec<PanicAllowlistEntry>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PanicAllowlistEntry {
    pub file: String,
    pub item: String,
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
    let allowlist_path = args
        .allowlist
        .unwrap_or_else(|| args.repo_root.join(".github/config/panic-allowlist.yaml"));
    let allowlist: PanicAllowlist = fixtures::load_yaml(&allowlist_path)
        .with_context(|| format!("failed to load {}", allowlist_path.display()))?;
    let calls = workspace::collect_panic_calls(&args.repo_root)?;
    let errors = validate_allowlist(&allowlist, &calls);

    if errors.is_empty() {
        Diagnostic::info(
            "PM3000",
            format!(
                "panic allowlist covers {} panic-bearing call(s)",
                calls.len()
            ),
        )
        .emit(output_mode, writer)?;
        return Ok(());
    }

    for error in &errors {
        Diagnostic::error("PM3001", error).emit(output_mode, writer)?;
    }
    bail!("panic allowlist has {} error(s)", errors.len())
}

pub fn validate_allowlist(allowlist: &PanicAllowlist, calls: &[PanicCall]) -> Vec<String> {
    let mut errors = Vec::new();
    if allowlist.version != 1 {
        errors.push(format!(
            "panic-allowlist.yaml version must be 1, got {}",
            allowlist.version
        ));
    }

    let mut allowed = BTreeMap::new();
    for entry in &allowlist.allowed {
        if entry.reason.trim().is_empty() {
            errors.push(format!(
                "{}::{} has an empty panic allowlist rationale",
                entry.file, entry.item
            ));
        }
        let key = (normalize_manifest_path(&entry.file), entry.item.clone());
        if allowed.insert(key.clone(), entry).is_some() {
            errors.push(format!(
                "duplicate panic allowlist entry for {}::{}",
                key.0, key.1
            ));
        }
    }

    let mut matched = BTreeSet::new();
    for call in calls {
        let key = (call.file.clone(), call.item.clone());
        if allowed.contains_key(&key) {
            matched.insert(key);
        } else {
            errors.push(format!(
                "panic call `{}` in {}::{} is not allowlisted",
                call.kind, call.file, call.item
            ));
        }
    }

    for key in allowed.keys() {
        if !matched.contains(key) {
            errors.push(format!(
                "panic allowlist entry {}::{} has no matching panic-bearing call",
                key.0, key.1
            ));
        }
    }

    errors
}

fn normalize_manifest_path(path: &str) -> String {
    path.replace('\\', "/")
}
