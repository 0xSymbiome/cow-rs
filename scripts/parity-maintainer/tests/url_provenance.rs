//! Integration test for the `url-provenance-check` subcommand.

use std::path::PathBuf;
use std::process::Command;

fn binary_path() -> PathBuf {
    let mut path = std::env::current_exe().expect("current_exe");
    path.pop(); // drop the test binary name
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

#[test]
fn rejects_credentialed_url_via_cli() {
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    std::fs::write(
        tmp.path(),
        "meta:\n  schema_version: 3\nrepositories:\n- id: example\n  remote: https://user:token@github.com/example/repo.git\n  commit: 0\n",
    )
    .expect("write");
    let output = Command::new(binary_path())
        .arg("url-provenance-check")
        .arg("--source-lock")
        .arg(tmp.path())
        .output()
        .expect("run");
    assert!(
        !output.status.success(),
        "credential-shaped URL must fail; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn accepts_clean_source_lock_via_cli() {
    let output = Command::new(binary_path())
        .arg("url-provenance-check")
        .arg("--source-lock")
        .arg(workspace_source_lock())
        .output()
        .expect("run");
    assert!(
        output.status.success(),
        "shipped source lock should pass; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
