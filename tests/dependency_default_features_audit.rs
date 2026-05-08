use std::{collections::BTreeSet, fs, path::PathBuf};

use toml::Value;

const REVIEWED_DEFAULT_FEATURE_EXCEPTIONS: &[&str] = &[
    "alloy-sol-macro",
    "async-lock",
    "async-trait",
    "console_error_panic_hook",
    "futures-timer",
    "getrandom",
    "gloo-timers",
    "hex",
    "http",
    "js-sys",
    "multibase",
    "num-bigint",
    "parking_lot",
    "pin-project-lite",
    "proptest",
    "serde",
    "serde_json",
    "serde-wasm-bindgen",
    "serde_yaml",
    "sha3",
    "syn",
    "thiserror",
    "toml",
    "trybuild",
    "url",
    "wasm-bindgen",
    "wasm-bindgen-futures",
    "wasm-bindgen-test",
    "web-time",
    "wiremock",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace test crate must live under the repository root")
        .to_path_buf()
}

#[test]
fn dependency_default_features_audit() {
    let manifest = fs::read_to_string(repo_root().join("Cargo.toml"))
        .expect("root Cargo.toml must be readable");
    let manifest: Value = toml::from_str(&manifest).expect("root Cargo.toml must parse");
    let dependencies = manifest["workspace"]["dependencies"]
        .as_table()
        .expect("workspace dependencies must be a table");
    let exceptions: BTreeSet<_> = REVIEWED_DEFAULT_FEATURE_EXCEPTIONS
        .iter()
        .copied()
        .collect();
    let mut missing_policy = Vec::new();

    for (name, value) in dependencies {
        if let Some(table) = value.as_table()
            && let Some(default_features) = table.get("default-features")
        {
            assert_eq!(
                default_features.as_bool(),
                Some(false),
                "{name} must not enable default features from the workspace dependency table",
            );
            continue;
        }
        if !exceptions.contains(name.as_str()) {
            missing_policy.push(name.as_str());
        }
    }

    assert!(
        missing_policy.is_empty(),
        "workspace dependencies must either set default-features = false or be listed as reviewed exceptions: {missing_policy:?}",
    );
}
