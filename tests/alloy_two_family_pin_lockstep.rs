use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

use toml::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace test crate must live under the repository root")
        .to_path_buf()
}

fn read_toml(path: &Path) -> Value {
    let content = fs::read_to_string(path).unwrap_or_else(|error| {
        panic!("failed to read {}: {error}", path.display());
    });
    toml::from_str(&content).unwrap_or_else(|error| {
        panic!("failed to parse {}: {error}", path.display());
    })
}

fn dependency_version(value: &Value) -> &str {
    value
        .as_str()
        .or_else(|| value.as_table()?.get("version")?.as_str())
        .expect("workspace dependency must declare a version")
}

fn major(version: &str) -> &str {
    version
        .trim_start_matches('=')
        .split('.')
        .next()
        .expect("dependency version must carry a major")
}

#[test]
fn alloy_two_family_pin_lockstep() {
    let root_manifest = read_toml(&repo_root().join("Cargo.toml"));
    let workspace_dependencies = root_manifest["workspace"]["dependencies"]
        .as_table()
        .expect("root workspace dependencies must be a table");

    let mut families: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (name, value) in workspace_dependencies
        .iter()
        .filter(|(name, _)| name.starts_with("alloy-"))
    {
        families
            .entry(major(dependency_version(value)).to_owned())
            .or_default()
            .insert(name.clone());
    }

    let abi_family = BTreeSet::from([
        "alloy-dyn-abi".to_owned(),
        "alloy-json-abi".to_owned(),
        "alloy-primitives".to_owned(),
        "alloy-serde".to_owned(),
        "alloy-sol-macro".to_owned(),
        "alloy-sol-types".to_owned(),
    ]);
    let runtime_family = BTreeSet::from([
        "alloy-consensus".to_owned(),
        "alloy-json-rpc".to_owned(),
        "alloy-network".to_owned(),
        "alloy-provider".to_owned(),
        "alloy-rpc-client".to_owned(),
        "alloy-rpc-types-eth".to_owned(),
        "alloy-signer".to_owned(),
        "alloy-signer-local".to_owned(),
        "alloy-transport".to_owned(),
        "alloy-transport-http".to_owned(),
    ]);

    assert_eq!(
        families.keys().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from(["1".to_owned(), "2".to_owned()]),
        "workspace alloy pins must contain exactly the ABI and runtime families",
    );
    assert_eq!(
        families.get("1"),
        Some(&abi_family),
        "alloy 1.x workspace pins must stay limited to the ABI family",
    );
    assert_eq!(
        families.get("2"),
        Some(&runtime_family),
        "alloy 2.x workspace pins must stay limited to the runtime family",
    );
}
