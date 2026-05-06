#![cfg(not(target_arch = "wasm32"))]

use std::process::Command;

#[test]
fn signer_manifest_declares_no_provider_dependencies() {
    let output = Command::new("cargo")
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .expect("cargo metadata should run");

    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("cargo metadata should be valid JSON");
    let package = metadata["packages"]
        .as_array()
        .expect("packages should be an array")
        .iter()
        .find(|package| package["name"].as_str() == Some("cow-sdk-alloy-signer"))
        .expect("signer package should be present");

    let provider_dependencies: Vec<_> = package["dependencies"]
        .as_array()
        .expect("dependencies should be an array")
        .iter()
        .filter_map(|dependency| dependency["name"].as_str())
        .filter(|name| name.starts_with("alloy-provider") || name.starts_with("alloy-transport"))
        .collect();

    assert!(
        provider_dependencies.is_empty(),
        "{provider_dependencies:?}"
    );
}

#[test]
fn cargo_tree_for_signer_does_not_include_alloy_provider() {
    let output = Command::new("cargo")
        .args(["tree", "-p", "cow-sdk-alloy-signer", "--edges", "normal"])
        .output()
        .expect("cargo tree should run");

    assert!(
        output.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("alloy-provider"), "{stdout}");
}
