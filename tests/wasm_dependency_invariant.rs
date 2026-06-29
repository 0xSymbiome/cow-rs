#![cfg(not(target_arch = "wasm32"))]

use std::{collections::BTreeSet, path::PathBuf, process::Command};

use serde_json::Value;

const FORBIDDEN: &[&str] = &[
    "cow-sdk-alloy-provider",
    "cow-sdk-alloy-signer",
    "cow-sdk-alloy",
    "alloy-provider",
    "alloy-transport-http",
    "alloy-network",
    "alloy-rpc-client",
    "alloy-rpc-types-eth",
    "reqwest",
    "hyper",
];

#[test]
fn cow_sdk_js_wasm32_tree_omits_forbidden_dependencies() {
    let metadata = Command::new("cargo")
        .args([
            "metadata",
            "--format-version",
            "1",
            "--filter-platform",
            "wasm32-unknown-unknown",
            "--no-deps",
        ])
        .current_dir(workspace_root())
        .output()
        .expect("cargo metadata should run");

    assert!(
        metadata.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&metadata.stderr)
    );

    let value: Value =
        serde_json::from_slice(&metadata.stdout).expect("cargo metadata should be valid JSON");
    let packages = value["packages"]
        .as_array()
        .expect("metadata packages should be an array");
    let wasm_package = packages
        .iter()
        .find(|package| package["name"] == "cow-sdk-js")
        .expect("cow-sdk-js should be a workspace package");
    let deps = wasm_package["dependencies"]
        .as_array()
        .expect("dependencies should be an array")
        .iter()
        .filter_map(|dep| dep["name"].as_str())
        .collect::<BTreeSet<_>>();

    for forbidden in FORBIDDEN {
        assert!(
            !deps.contains(forbidden),
            "cow-sdk-js must not depend on {forbidden} for wasm32"
        );
    }
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("tests crate should be inside workspace")
        .to_path_buf()
}
