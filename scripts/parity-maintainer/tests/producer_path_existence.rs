//! Integration test asserting that every producer path declared in the
//! shipped source lock under `parity/source-lock.yaml` either resolves
//! against a provisioned upstream checkout or is correctly absent in the
//! standalone-repo posture.
//!
//! The standalone-repo posture skips the producer-path-existence check
//! because the upstream checkouts are not provisioned. This test exercises
//! that fall-back behavior by invoking the CLI without `--cow-sdk-root`,
//! `--contracts-root`, or `--services-root` and asserting a clean exit.

use std::path::PathBuf;
use std::process::Command;

fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().expect("current_exe");
    path.pop();
    if path.ends_with("deps") {
        path.pop();
    }
    path.push(if cfg!(windows) {
        "parity-maintainer.exe"
    } else {
        "parity-maintainer"
    });
    path
}

fn workspace_source_lock() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("parity")
        .join("source-lock.yaml")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
}

#[test]
fn standalone_posture_accepts_missing_upstream_roots() {
    let output = Command::new(binary_path())
        .arg("validate-producer-paths")
        .arg("--source-lock")
        .arg(workspace_source_lock())
        .current_dir(workspace_root())
        .output()
        .expect("run");
    assert!(
        output.status.success(),
        "standalone posture should succeed without upstream roots; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
