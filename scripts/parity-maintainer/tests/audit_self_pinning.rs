//! Integration tests for the `audit-self-pinning` subcommand.
//!
//! The subcommand walks every JSON fixture under a supplied root, classifies
//! each by authority shape, cross-checks against the supplied allowlist, and
//! prints a per-fixture report. In report-only mode the subcommand exits zero
//! regardless of findings. In blocking mode the subcommand exits non-zero
//! when at least one rejected fixture is not covered by the allowlist.
//!
//! These tests exercise both modes against synthetic fixture trees and
//! against the shipped tree under `parity/fixtures/` plus the shipped
//! allowlist at `parity/self-pinning-allowlist.yaml`.

use std::fs;
use std::path::{Path, PathBuf};
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

fn shipped_fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("parity")
        .join("fixtures")
}

fn shipped_allowlist() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("parity")
        .join("self-pinning-allowlist.yaml")
}

fn run_against(
    fixtures_root: &Path,
    allowlist: &Path,
    blocking: bool,
) -> std::process::Output {
    let mut command = Command::new(binary_path());
    command
        .arg("audit-self-pinning")
        .arg("--fixtures-root")
        .arg(fixtures_root)
        .arg("--allowlist")
        .arg(allowlist);
    if blocking {
        command.arg("--blocking");
    }
    command.output().expect("audit-self-pinning runs")
}

fn write(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, content).expect("write fixture");
}

fn stderr_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn stdout_of(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn shipped_tree_in_report_mode_exits_zero() {
    let output = run_against(&shipped_fixtures_root(), &shipped_allowlist(), false);
    assert!(
        output.status.success(),
        "report-only mode must exit zero; stderr=\n{}\nstdout=\n{}",
        stderr_of(&output),
        stdout_of(&output)
    );
    let stdout = stdout_of(&output);
    assert!(
        stdout.contains("audit-self-pinning audited"),
        "summary line must be present; stdout=\n{stdout}"
    );
    assert!(
        stdout.contains("report-only"),
        "summary line must declare report-only mode; stdout=\n{stdout}"
    );
}

#[test]
fn shipped_allowlist_parses_with_documented_schema_version() {
    let allowlist = shipped_allowlist();
    let raw = fs::read_to_string(&allowlist).expect("read allowlist");
    assert!(
        raw.contains("schema_version: 1"),
        "shipped allowlist must declare schema_version 1"
    );
}

#[test]
fn empty_fixture_tree_exits_zero() {
    let temp = tempfile::tempdir().expect("tempdir");
    let allowlist_path = temp.path().join("allowlist.yaml");
    fs::write(
        &allowlist_path,
        "schema_version: 1\ngrandfathered: []\nin_flight_upgrade: []\n",
    )
    .expect("write allowlist");

    let output = run_against(temp.path(), &allowlist_path, true);
    assert!(
        output.status.success(),
        "empty tree must exit zero; stderr=\n{}",
        stderr_of(&output)
    );
}

#[test]
fn attributed_fixture_passes_blocking_mode() {
    let temp = tempfile::tempdir().expect("tempdir");
    let fixture = temp.path().join("attributed.json");
    write(
        &fixture,
        r#"{
          "source_refs": [
            { "repo": "cowprotocol/contracts", "commit": "c94c595a791681cf8ba7495117dcde397b932885", "path": "src/contracts/libraries/GPv2Order.sol" }
          ],
          "cases": []
        }"#,
    );
    let allowlist_path = temp.path().join("allowlist.yaml");
    fs::write(
        &allowlist_path,
        "schema_version: 1\ngrandfathered: []\nin_flight_upgrade: []\n",
    )
    .expect("write allowlist");

    let output = run_against(temp.path(), &allowlist_path, true);
    assert!(
        output.status.success(),
        "attributed fixture must pass blocking mode; stderr=\n{}",
        stderr_of(&output)
    );
}

#[test]
fn rust_self_pin_without_allowlist_fails_blocking_mode() {
    let temp = tempfile::tempdir().expect("tempdir");
    let fixture = temp.path().join("self_pin.json");
    write(
        &fixture,
        r#"{
          "source": "alloy_sol_types::SolStruct::eip712_signing_hash on GPv2 Order",
          "rows": []
        }"#,
    );
    let allowlist_path = temp.path().join("allowlist.yaml");
    fs::write(
        &allowlist_path,
        "schema_version: 1\ngrandfathered: []\nin_flight_upgrade: []\n",
    )
    .expect("write allowlist");

    let output = run_against(temp.path(), &allowlist_path, true);
    assert!(
        !output.status.success(),
        "self-pinned fixture without allowlist must fail blocking mode; stdout=\n{}\nstderr=\n{}",
        stdout_of(&output),
        stderr_of(&output)
    );
    assert!(
        stderr_of(&output).contains("not covered by the self-pinning allowlist"),
        "diagnostic must name the missing allowlist coverage; stderr=\n{}",
        stderr_of(&output)
    );
}

#[test]
fn rust_self_pin_with_allowlist_passes_blocking_mode() {
    let temp = tempfile::tempdir().expect("tempdir");
    let fixture = temp.path().join("self_pin.json");
    write(
        &fixture,
        r#"{
          "source": "alloy_sol_types::SolStruct::eip712_signing_hash on GPv2 Order",
          "rows": []
        }"#,
    );
    // The audit emits the fixture path with forward-slash normalization; the
    // allowlist entry must match that canonical form byte for byte.
    let allowlist_path = temp.path().join("allowlist.yaml");
    let allowlist_body = format!(
        "schema_version: 1\n\
         grandfathered:\n  \
         - path: {}\n    \
         class: RustSelfPin\n    \
         justification: |\n      Test grandfather entry for the self-pin case.\n    \
         review_trigger: never (test fixture)\n\
         in_flight_upgrade: []\n",
        canonical_fixture_path(&fixture)
    );
    fs::write(&allowlist_path, allowlist_body).expect("write allowlist");

    let output = run_against(temp.path(), &allowlist_path, true);
    assert!(
        output.status.success(),
        "allowlisted self-pin must pass blocking mode; stderr=\n{}",
        stderr_of(&output)
    );
}

#[test]
fn free_form_prose_is_reported_as_rejection() {
    let temp = tempfile::tempdir().expect("tempdir");
    let fixture = temp.path().join("prose.json");
    write(
        &fixture,
        r#"{
          "source": "ComposableCoW.hash(ConditionalOrderParams)",
          "rows": []
        }"#,
    );
    let allowlist_path = temp.path().join("allowlist.yaml");
    fs::write(
        &allowlist_path,
        "schema_version: 1\ngrandfathered: []\nin_flight_upgrade: []\n",
    )
    .expect("write allowlist");

    let report_output = run_against(temp.path(), &allowlist_path, false);
    assert!(
        report_output.status.success(),
        "report-only mode must exit zero on rejections; stderr=\n{}",
        stderr_of(&report_output)
    );
    let stdout = stdout_of(&report_output);
    assert!(
        stdout.contains("free-form-prose"),
        "report must mention the free-form-prose class; stdout=\n{stdout}"
    );

    let blocking_output = run_against(temp.path(), &allowlist_path, true);
    assert!(
        !blocking_output.status.success(),
        "blocking mode must fail on un-allowlisted free-form prose"
    );
}

#[test]
fn missing_authority_is_reported_for_expected_bearing_fixture() {
    let temp = tempfile::tempdir().expect("tempdir");
    let fixture = temp.path().join("missing.json");
    write(
        &fixture,
        r#"{
          "rows": [
            { "chain_id": 1, "digest": "0xe489e6d7ce9431d0131bb4bf6a5b2919ad6e8da96b6130ff3a93f3bc806eb952" }
          ]
        }"#,
    );
    let allowlist_path = temp.path().join("allowlist.yaml");
    fs::write(
        &allowlist_path,
        "schema_version: 1\ngrandfathered: []\nin_flight_upgrade: []\n",
    )
    .expect("write allowlist");

    let output = run_against(temp.path(), &allowlist_path, false);
    assert!(
        output.status.success(),
        "report-only mode must exit zero; stderr=\n{}",
        stderr_of(&output)
    );
    let stdout = stdout_of(&output);
    assert!(
        stdout.contains("missing"),
        "report must mention the missing class; stdout=\n{stdout}"
    );

    let blocking_output = run_against(temp.path(), &allowlist_path, true);
    assert!(
        !blocking_output.status.success(),
        "blocking mode must fail on un-allowlisted missing authority"
    );
}

#[test]
fn input_only_fixture_passes_blocking_mode() {
    let temp = tempfile::tempdir().expect("tempdir");
    let fixture = temp.path().join("input.json");
    write(
        &fixture,
        r#"{
          "sender": "0x0000000000000000000000000000000000000005",
          "placementError": "none"
        }"#,
    );
    let allowlist_path = temp.path().join("allowlist.yaml");
    fs::write(
        &allowlist_path,
        "schema_version: 1\ngrandfathered: []\nin_flight_upgrade: []\n",
    )
    .expect("write allowlist");

    let output = run_against(temp.path(), &allowlist_path, true);
    assert!(
        output.status.success(),
        "input-only fixture must pass blocking mode; stdout=\n{}\nstderr=\n{}",
        stdout_of(&output),
        stderr_of(&output)
    );
}

#[test]
fn spec_anchored_row_passes_blocking_mode() {
    let temp = tempfile::tempdir().expect("tempdir");
    let fixture = temp.path().join("spec.json");
    write(
        &fixture,
        r#"{
          "rows": [
            { "@source_ref": "RFC 7231 section 7.1.1.1", "expected": { "value": "Wed, 21 Oct 2026 07:28:00 GMT" } }
          ]
        }"#,
    );
    let allowlist_path = temp.path().join("allowlist.yaml");
    fs::write(
        &allowlist_path,
        "schema_version: 1\ngrandfathered: []\nin_flight_upgrade: []\n",
    )
    .expect("write allowlist");

    let output = run_against(temp.path(), &allowlist_path, true);
    assert!(
        output.status.success(),
        "spec-anchored fixture must pass blocking mode; stderr=\n{}",
        stderr_of(&output)
    );
}

#[test]
fn allowlist_with_unknown_schema_version_is_rejected() {
    let temp = tempfile::tempdir().expect("tempdir");
    let allowlist_path = temp.path().join("allowlist.yaml");
    fs::write(
        &allowlist_path,
        "schema_version: 99\ngrandfathered: []\nin_flight_upgrade: []\n",
    )
    .expect("write allowlist");

    let output = run_against(temp.path(), &allowlist_path, false);
    assert!(
        !output.status.success(),
        "unknown allowlist schema version must surface as an error"
    );
    assert!(
        stderr_of(&output).contains("schema_version 99"),
        "diagnostic must name the unexpected schema version; stderr=\n{}",
        stderr_of(&output)
    );
}

fn canonical_fixture_path(fixture: &Path) -> String {
    fixture.display().to_string().replace('\\', "/")
}
