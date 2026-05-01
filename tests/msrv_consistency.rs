use std::{fs, path::PathBuf};

use toml::Value;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace test crate must live under the repository root")
        .to_path_buf()
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).unwrap_or_else(|error| {
        panic!("failed to read {path}: {error}");
    })
}

#[test]
fn msrv_consistency() {
    let manifest: Value =
        toml::from_str(&read_repo_file("Cargo.toml")).expect("root Cargo.toml must parse");
    let workspace_msrv = manifest["workspace"]["package"]["rust-version"]
        .as_str()
        .expect("workspace.package.rust-version must be declared");
    let ci = read_repo_file(".github/workflows/ci.yml");
    let ci_msrv = ci
        .lines()
        .find_map(|line| line.trim().strip_prefix("RUST_MSRV:"))
        .map(str::trim)
        .map(|value| value.trim_matches('\'').trim_matches('"'))
        .expect("ci.yml must declare RUST_MSRV");

    assert_eq!(
        workspace_msrv, ci_msrv,
        "workspace MSRV must match the CI compatibility-floor input",
    );
}
