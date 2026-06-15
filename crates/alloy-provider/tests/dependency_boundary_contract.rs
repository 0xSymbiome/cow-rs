use std::process::Command;

#[test]
fn provider_manifest_declares_no_signer_family_dependencies() {
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
        .find(|package| package["name"].as_str() == Some("cow-sdk-alloy-provider"))
        .expect("provider package should be present");

    let signer_dependencies: Vec<_> = package["dependencies"]
        .as_array()
        .expect("dependencies should be an array")
        .iter()
        .filter_map(|dependency| dependency["name"].as_str())
        .filter(|name| name.starts_with("alloy-signer"))
        .collect();

    assert!(signer_dependencies.is_empty(), "{signer_dependencies:?}");
}

#[test]
fn cargo_tree_for_provider_does_not_include_local_private_key_signer() {
    let output = Command::new("cargo")
        .args(["tree", "-p", "cow-sdk-alloy-provider", "--edges", "normal"])
        .output()
        .expect("cargo tree should run");

    assert!(
        output.status.success(),
        "cargo tree failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains("alloy-signer-local"), "{stdout}");
}
