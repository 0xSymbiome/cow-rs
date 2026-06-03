//! Shared scaffold for the alloy-family published-crate dependency-coverage
//! invariants.
//!
//! `alloy_provider_invariant_covers_every_published_crate.rs` and
//! `alloy_signer_invariant_covers_every_published_crate.rs` previously each
//! carried a byte-identical copy of these four helpers. They now live here once
//! and are shared via `#[path = "support/published_crates.rs"] mod support;`,
//! so the two invariants stay independently named (and independently cited in
//! `PROPERTIES.md`) without duplicating the scaffold.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use toml::Value;

pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace test crate must live under the repository root")
        .to_path_buf()
}

pub fn read_toml(path: &Path) -> Value {
    let content = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    toml::from_str(&content).unwrap_or_else(|error| {
        panic!("failed to parse {}: {error}", path.display());
    })
}

pub fn published_workspace_crates(root: &Path) -> BTreeMap<String, PathBuf> {
    let root_manifest = read_toml(&root.join("Cargo.toml"));
    let members = root_manifest["workspace"]["members"]
        .as_array()
        .expect("workspace members must be declared");
    let mut published_crates = BTreeMap::new();

    for member in members.iter().filter_map(Value::as_str) {
        if !member.starts_with("crates/") {
            continue;
        }
        let manifest_path = root.join(member).join("Cargo.toml");
        let manifest = read_toml(&manifest_path);
        let package = manifest["package"]
            .as_table()
            .expect("crate member must declare [package]");
        if package.get("publish").and_then(Value::as_bool) == Some(false) {
            continue;
        }
        let name = package["name"]
            .as_str()
            .expect("crate member must declare package.name");
        published_crates.insert(name.to_owned(), manifest_path);
    }

    published_crates
}

pub fn dependency_names(manifest: &Value) -> BTreeSet<String> {
    let mut names: BTreeSet<_> = manifest
        .get("dependencies")
        .and_then(Value::as_table)
        .into_iter()
        .flat_map(|dependencies| dependencies.keys().cloned())
        .collect();

    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            if let Some(dependencies) = target.get("dependencies").and_then(Value::as_table) {
                names.extend(dependencies.keys().cloned());
            }
        }
    }

    names
}
