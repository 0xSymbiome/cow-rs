mod common;

use anyhow::Result;
use tempfile::tempdir;

use common::{
    RepoSpec, command, commit_all, init_git_repo, output_text, write_file, write_source_lock,
};

#[test]
fn vendor_openapi_stamps_synthetic_services_openapi() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let services_root = root.join("services");
    init_git_repo(&services_root, Some(&services_root.display().to_string()))?;
    write_file(
        services_root.join("crates/orderbook/openapi.yml"),
        "openapi: 3.0.0\ninfo:\n  title: fixture\n  version: 1.0.0\n",
    )?;
    let commit = commit_all(&services_root, "initial openapi")?;

    write_source_lock(
        &root.join("source-lock.yaml"),
        &[RepoSpec {
            id: "services",
            remote: services_root.display().to_string(),
            commit: commit.clone(),
            producer_paths: vec!["crates/orderbook/openapi.yml"],
        }],
    )?;

    let output = command()
        .current_dir(root)
        .args([
            "parity",
            "vendor-openapi",
            "--source-lock",
            "source-lock.yaml",
            "--root",
            root.to_str().expect("utf8 temp path"),
        ])
        .output()?;
    assert!(output.status.success(), "{}", output_text(&output));

    // The output defaults to `openapi/services-orderbook.yml` next to the lock.
    let vendored = std::fs::read_to_string(root.join("openapi/services-orderbook.yml"))?;
    assert!(vendored.contains(&format!("# Vendored from cowprotocol/services @ {commit}")));
    assert!(vendored.contains("# Path: crates/orderbook/openapi.yml"));
    // The header carries no wall-clock timestamp so re-vendoring an unchanged
    // upstream commit is byte-for-byte deterministic.
    assert!(!vendored.contains("# Generated:"));
    assert!(vendored.contains("# DO NOT EDIT - regenerate via `cargo parity-vendor-openapi`."));
    assert!(vendored.contains("title: fixture"));
    Ok(())
}

#[test]
fn vendor_openapi_pins_an_ahead_checkout_before_vendoring() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let services_root = root.join("services");
    init_git_repo(&services_root, Some(&services_root.display().to_string()))?;
    write_file(
        services_root.join("crates/orderbook/openapi.yml"),
        "openapi: 3.0.0\ninfo:\n  title: first\n  version: 1.0.0\n",
    )?;
    let pinned = commit_all(&services_root, "first openapi")?;
    write_file(
        services_root.join("crates/orderbook/openapi.yml"),
        "openapi: 3.0.0\ninfo:\n  title: second\n  version: 1.0.0\n",
    )?;
    commit_all(&services_root, "second openapi")?;

    write_source_lock(
        &root.join("source-lock.yaml"),
        &[RepoSpec {
            id: "services",
            remote: services_root.display().to_string(),
            commit: pinned.clone(),
            producer_paths: vec!["crates/orderbook/openapi.yml"],
        }],
    )?;

    // The checkout sits ahead of the pin; the command re-detaches it at the
    // pinned commit and vendors the pinned bytes, not the checkout's HEAD.
    let output = command()
        .current_dir(root)
        .args([
            "parity",
            "vendor-openapi",
            "--source-lock",
            "source-lock.yaml",
            "--root",
            root.to_str().expect("utf8 temp path"),
        ])
        .output()?;
    assert!(output.status.success(), "{}", output_text(&output));

    let vendored = std::fs::read_to_string(root.join("openapi/services-orderbook.yml"))?;
    assert!(vendored.contains(&format!("# Vendored from cowprotocol/services @ {pinned}")));
    assert!(vendored.contains("title: first"));
    assert!(!vendored.contains("title: second"));
    Ok(())
}
