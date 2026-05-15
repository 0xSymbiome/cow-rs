//! Integration test for the `validate-enum-policy` subcommand.
//!
//! The validator confirms that `.github/config/enum-policy.yaml` parses
//! cleanly as a versioned manifest with a non-empty enums list. The shipped
//! policy file must always pass; an invalid file must fail.

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

fn shipped_enum_policy() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join(".github")
        .join("config")
        .join("enum-policy.yaml")
}

#[test]
fn shipped_enum_policy_is_valid() {
    let output = Command::new(binary_path())
        .arg("validate-enum-policy")
        .arg("--policy")
        .arg(shipped_enum_policy())
        .output()
        .expect("run");
    assert!(
        output.status.success(),
        "shipped enum policy must validate; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn empty_enum_policy_fails() {
    let tmp = tempfile::NamedTempFile::new().expect("tempfile");
    std::fs::write(tmp.path(), "version: 1\nenums: []\n").expect("write");
    let output = Command::new(binary_path())
        .arg("validate-enum-policy")
        .arg("--policy")
        .arg(tmp.path())
        .output()
        .expect("run");
    assert!(
        !output.status.success(),
        "empty enum list must fail; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
