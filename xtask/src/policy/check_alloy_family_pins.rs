//! Verifies the workspace `alloy-*` pins stay internally consistent per release
//! train.
//!
//! Upstream alloy publishes two coordinated trains:
//!
//! - alloy-core (1.x): `alloy-primitives`, `alloy-sol-types`, `alloy-sol-macro`,
//!   `alloy-dyn-abi`, `alloy-json-abi`, `alloy-serde`
//! - alloy runtime (2.x): `alloy-consensus`, `alloy-network`, `alloy-provider`,
//!   `alloy-signer*`, `alloy-transport*`, `alloy-rpc-types-eth`, `alloy-json-rpc`
//!
//! Each family must agree on `major.minor`; the cross-family lockstep is owned
//! by `tests/alloy_two_family_lockfile_invariant.rs`. Every `alloy-*` workspace
//! dependency must fall into one of the two families, so a newly added crate
//! fails closed until it is classified. A small forbidden list rejects outright
//! the `alloy-*` crates that would duplicate a cow-owned authority — e.g.
//! `alloy-chains`, whose role is owned by `SupportedChainId` and the
//! cow-specific URL grammar (ADR 0005, ADR 0011).

use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::policy::workspace;

const CORE_FAMILY: &[&str] = &[
    "alloy-primitives",
    "alloy-sol-types",
    "alloy-sol-macro",
    "alloy-dyn-abi",
    "alloy-json-abi",
    "alloy-serde",
];

const RUNTIME_FAMILY: &[&str] = &[
    "alloy-consensus",
    "alloy-network",
    "alloy-provider",
    "alloy-signer-local",
    "alloy-signer",
    "alloy-transport-http",
    "alloy-transport",
    "alloy-rpc-client",
    "alloy-rpc-types-eth",
    "alloy-json-rpc",
];

/// `alloy-*` crates that must never be a workspace dependency, paired with the
/// rejection reason. These duplicate a cow-owned authority rather than belonging
/// to either release train.
const FORBIDDEN: &[(&str, &str)] = &[(
    "alloy-chains",
    "alloy-chains is forbidden as a workspace dependency; cow-rs uses SupportedChainId as the chain authority plus the cow-specific URL grammar (ADR 0005, ADR 0011)",
)];

#[derive(Debug, clap::Args)]
pub struct Args {
    /// Repository root.
    #[arg(long, default_value = ".")]
    pub repo_root: PathBuf,
}

#[derive(Deserialize)]
struct Manifest {
    workspace: WorkspaceTable,
}

#[derive(Deserialize)]
struct WorkspaceTable {
    dependencies: BTreeMap<String, toml::Value>,
}

pub fn run_default() -> Result<()> {
    run(&Args {
        repo_root: PathBuf::from("."),
    })
}

pub fn run(args: &Args) -> Result<()> {
    let manifest: Manifest = toml::from_str(&workspace::read_to_string(
        &args.repo_root.join("Cargo.toml"),
    )?)
    .context("failed to parse workspace Cargo.toml")?;
    let errors = validate(&manifest.workspace.dependencies);

    if errors.is_empty() {
        println!("alloy-* family pins are internally consistent");
        return Ok(());
    }
    for error in &errors {
        eprintln!("error: {error}");
    }
    bail!("alloy family pins have {} error(s)", errors.len())
}

fn validate(dependencies: &BTreeMap<String, toml::Value>) -> Vec<String> {
    let mut errors = Vec::new();
    let mut core = BTreeMap::new();
    let mut runtime = BTreeMap::new();
    let mut unclassified = Vec::new();

    for (name, value) in dependencies {
        if !name.starts_with("alloy-") {
            continue;
        }
        if let Some((_, reason)) = FORBIDDEN.iter().find(|(forbidden, _)| *forbidden == name) {
            errors.push((*reason).to_owned());
            continue;
        }
        let Some(minor) = dependency_minor(value) else {
            continue;
        };
        if CORE_FAMILY.contains(&name.as_str()) {
            core.insert(name.clone(), minor);
        } else if RUNTIME_FAMILY.contains(&name.as_str()) {
            runtime.insert(name.clone(), minor);
        } else {
            unclassified.push(name.clone());
        }
    }

    check_family("alloy-core", &core, &mut errors);
    check_family("alloy runtime", &runtime, &mut errors);
    if !unclassified.is_empty() {
        errors.push(format!(
            "unclassified alloy-* pin(s) ({}); add them to the core or runtime family in check_alloy_family_pins",
            unclassified.join(", ")
        ));
    }
    errors
}

fn check_family(label: &str, pins: &BTreeMap<String, String>, errors: &mut Vec<String>) {
    let minors: std::collections::BTreeSet<&String> = pins.values().collect();
    match minors.len() {
        0 => errors.push(format!(
            "no {label} alloy-* pins found in the workspace manifest"
        )),
        1 => {}
        _ => errors.push(format!(
            "{label} pins disagree on major.minor: {}",
            pins.iter()
                .map(|(name, minor)| format!("{name}={minor}"))
                .collect::<Vec<_>>()
                .join(", ")
        )),
    }
}

/// Extracts the `major.minor` of a dependency declared either as a bare version
/// string or as a `{ version = "..." }` table. Git/path dependencies without a
/// version are skipped.
fn dependency_minor(value: &toml::Value) -> Option<String> {
    let version = match value {
        toml::Value::String(version) => version.as_str(),
        toml::Value::Table(table) => table.get("version")?.as_str()?,
        _ => return None,
    };
    major_minor(version)
}

fn major_minor(version: &str) -> Option<String> {
    let trimmed = version.trim_start_matches(['^', '~', '=', '>', '<', ' ']);
    let mut parts = trimmed.split('.');
    let major = parts.next()?;
    let minor = parts.next()?;
    if major.chars().all(|c| c.is_ascii_digit()) && minor.chars().all(|c| c.is_ascii_digit()) {
        Some(format!("{major}.{minor}"))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::validate;

    fn deps(pairs: &[(&str, &str)]) -> BTreeMap<String, toml::Value> {
        pairs
            .iter()
            .map(|(name, version)| {
                (
                    (*name).to_owned(),
                    toml::Value::String((*version).to_owned()),
                )
            })
            .collect()
    }

    #[test]
    fn consistent_families_pass() {
        let dependencies = deps(&[
            ("alloy-primitives", "1.5.7"),
            ("alloy-sol-types", "1.5.0"),
            ("alloy-provider", "2.0.4"),
            ("alloy-network", "2.0.4"),
        ]);
        assert!(validate(&dependencies).is_empty());
    }

    #[test]
    fn a_family_minor_mismatch_and_an_unclassified_pin_are_rejected() {
        let dependencies = deps(&[
            ("alloy-primitives", "1.5.7"),
            ("alloy-sol-types", "1.6.0"),
            ("alloy-provider", "2.0.4"),
            ("alloy-brand-new", "3.0.0"),
        ]);
        let errors = validate(&dependencies);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("disagree on major.minor"))
        );
        assert!(errors.iter().any(|error| error.contains("unclassified")));
    }

    #[test]
    fn a_forbidden_alloy_crate_is_rejected_with_its_reason() {
        let dependencies = deps(&[
            ("alloy-primitives", "1.5.7"),
            ("alloy-provider", "2.0.4"),
            ("alloy-chains", "0.1.0"),
        ]);
        let errors = validate(&dependencies);
        assert!(
            errors
                .iter()
                .any(|error| error.contains("alloy-chains is forbidden"))
        );
        // A forbidden crate must give its specific reason, not the generic
        // "unclassified" message.
        assert!(!errors.iter().any(|error| error.contains("unclassified")));
    }
}
