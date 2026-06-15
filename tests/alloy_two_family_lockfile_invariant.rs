use std::{collections::BTreeMap, fs, path::PathBuf};

use toml::Value;

const ALLOY_RUNTIME_CRATES: &[&str] = &[
    "alloy-consensus",
    "alloy-json-rpc",
    "alloy-network",
    "alloy-provider",
    "alloy-rpc-types-eth",
    "alloy-signer",
    "alloy-signer-local",
    "alloy-transport",
    "alloy-transport-http",
];

const ALLOY_RUNTIME_PINNED_VERSION: &str = "2.0.4";

const ALLOY_CORE_CRATES: &[&str] = &[
    "alloy-dyn-abi",
    "alloy-json-abi",
    "alloy-primitives",
    "alloy-sol-macro",
    "alloy-sol-types",
];

const ALLOY_CORE_PINNED_VERSION: &str = "1.5.7";

#[test]
fn alloy_runtime_and_core_pin_lockstep_in_cargo_lock() {
    let lockfile_path = workspace_root().join("Cargo.lock");
    let raw = fs::read_to_string(&lockfile_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", lockfile_path.display()));
    let parsed: Value = toml::from_str(&raw)
        .unwrap_or_else(|error| panic!("parse {}: {error}", lockfile_path.display()));
    let packages = parsed
        .get("package")
        .and_then(Value::as_array)
        .expect("Cargo.lock package array");

    let mut resolved: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for package in packages {
        let name = package
            .get("name")
            .and_then(Value::as_str)
            .expect("package name");
        if !ALLOY_RUNTIME_CRATES.contains(&name) && !ALLOY_CORE_CRATES.contains(&name) {
            continue;
        }
        let version = package
            .get("version")
            .and_then(Value::as_str)
            .expect("package version");
        resolved
            .entry(name.to_owned())
            .or_default()
            .push(version.to_owned());
    }

    let mut failures = Vec::new();
    assert_family(
        &resolved,
        ALLOY_RUNTIME_CRATES,
        ALLOY_RUNTIME_PINNED_VERSION,
        "alloy runtime",
        &mut failures,
    );
    assert_family(
        &resolved,
        ALLOY_CORE_CRATES,
        ALLOY_CORE_PINNED_VERSION,
        "alloy-core ABI",
        &mut failures,
    );

    assert!(
        failures.is_empty(),
        "Cargo.lock alloy lockstep invariant violated:\n{}",
        failures.join("\n")
    );
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace test crate must live below repository root")
        .to_path_buf()
}

fn assert_family(
    resolved: &BTreeMap<String, Vec<String>>,
    expected_crates: &[&str],
    pinned_version: &str,
    family_label: &str,
    failures: &mut Vec<String>,
) {
    for &crate_name in expected_crates {
        let versions = resolved.get(crate_name).cloned().unwrap_or_default();
        match versions.as_slice() {
            [] => failures.push(format!(
                "missing {family_label} crate `{crate_name}` in Cargo.lock"
            )),
            [version] if version == pinned_version => {}
            [version] => failures.push(format!(
                "{family_label} crate `{crate_name}` resolves to `{version}`, expected pinned `{pinned_version}`. Run `cargo update -p {crate_name} --precise {pinned_version}` to restore the pin."
            )),
            _ => failures.push(format!(
                "{family_label} crate `{crate_name}` resolves to multiple versions: {versions:?}"
            )),
        }
    }
}
