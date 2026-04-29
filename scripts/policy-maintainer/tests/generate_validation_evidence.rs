mod common;

use std::fs;

use policy_maintainer::{
    diagnostics::OutputMode,
    generate_validation_evidence::{Args, run_with_writer},
};

use common::TempDir;

fn fixture_args(temp: &TempDir) -> Args {
    Args {
        repo_root: temp.path().to_path_buf(),
        release_version: "0.1.0".to_owned(),
        output: Some(temp.path().join("validation.md")),
        check: false,
        source_lock: None,
        openapi: None,
        wasm_versions: None,
        deployment_provenance: None,
        lane_status: None,
    }
}

fn write_fixtures(temp: &TempDir) {
    temp.write(
        "parity/source-lock.yaml",
        "meta:\n  generated_at_utc: 2026-04-29T00:00:00Z\nrepositories:\n  - id: services\n    remote: https://github.com/cowprotocol/services.git\n    commit: bf40548684828ad72c1e10fbe8fe3467c90eba45\n    role: reference-only\n  - id: cow-sdk\n    remote: https://github.com/cowprotocol/cow-sdk.git\n    commit: 00c3dbd41c086ff9a51d5e5a30648615d4c66d0d\n    role: primary\n",
    );
    temp.write(
        "parity/openapi/services-orderbook.yml",
        "# Vendored from cowprotocol/services @ bf40548684828ad72c1e10fbe8fe3467c90eba45\n# Path: crates/orderbook/openapi.yml\n# Generated: 2026-04-29T16:55:47Z\nopenapi: 3.0.3\n",
    );
    temp.write(
        ".github/config/wasm-test-versions.yaml",
        "version: 1\nchrome_for_testing:\n  channel: Stable\n  version: '148.0.7778.56'\n  revision: '1610480'\n  released_at: '2026-04-28T20:36:36.653Z'\n",
    );
    temp.write(
        "crates/contracts/deployment-provenance.yaml",
        "version: 1\ngenerated_at_utc: 2026-04-29T00:00:00Z\nprovenance:\n  - contract_id: Settlement\n    chain_id: 100\n    env: prod\n    address: 0x2\n    live_confirmation:\n      kind: code_hash\n      code_hash: 0xbbb\n      confirmed_at: 2026-04-29T17:26:25Z\n  - contract_id: Settlement\n    chain_id: 1\n    env: prod\n    address: 0x1\n    live_confirmation:\n      kind: code_hash\n      code_hash: 0xaaa\n      confirmed_at: 2026-04-29T17:26:25Z\n",
    );
    temp.write(
        ".github/release-evidence/release-readiness-status-0.1.0.yaml",
        "version: 1\ngenerated_at_utc: 2026-04-29T17:30:00Z\nworkflow:\n  name: release-readiness\n  file: .github/workflows/release-readiness.yml\n  run_url:\n  commit_sha: abcdef\nlanes:\n  - lane: test\n    status: pass\n    step_id: quality-gate/test\n    notes: cargo test --workspace --all-features\n  - lane: clippy\n    status: pass\n    step_id: quality-gate/clippy\n    notes: cargo clippy -D warnings\n",
    );
}

#[test]
fn generates_deterministic_markdown_from_fixture_inputs() {
    let temp = TempDir::new("generate-validation-evidence-happy");
    write_fixtures(&temp);
    let mut output = Vec::new();

    run_with_writer(fixture_args(&temp), OutputMode::Text, &mut output).unwrap();

    let markdown = fs::read_to_string(temp.path().join("validation.md")).unwrap();
    assert!(markdown.contains("# Validation Evidence - cow-rs 0.1.0"));
    assert!(markdown.contains("Workflow file: .github/workflows/release-readiness.yml"));
    assert!(markdown.contains("Workflow run: pending final run"));
    assert!(markdown.contains("Release classification: first_functional (semver-checks: skip)"));
    assert!(
        markdown.find("| clippy | pass | quality-gate/clippy | cargo clippy -D warnings |")
            < markdown.find(
                "| test | pass | quality-gate/test | cargo test --workspace --all-features |"
            )
    );
    assert!(
        markdown.find("| 1 | prod | Settlement | 0x1 | 0xaaa | 2026-04-29T17:26:25Z |")
            < markdown.find("| 100 | prod | Settlement | 0x2 | 0xbbb | 2026-04-29T17:26:25Z |")
    );
    assert!(
        String::from_utf8(output)
            .unwrap()
            .contains("wrote validation evidence")
    );
}

#[test]
fn check_mode_fails_on_one_byte_diff() {
    let temp = TempDir::new("generate-validation-evidence-check");
    write_fixtures(&temp);
    let mut output = Vec::new();
    run_with_writer(fixture_args(&temp), OutputMode::Text, &mut output).unwrap();

    let mut args = fixture_args(&temp);
    args.check = true;
    let mut check_output = Vec::new();
    run_with_writer(args, OutputMode::Text, &mut check_output).unwrap();

    let path = temp.path().join("validation.md");
    let mut markdown = fs::read_to_string(&path).unwrap();
    markdown.push('x');
    fs::write(&path, markdown).unwrap();

    let mut args = fixture_args(&temp);
    args.check = true;
    let error = run_with_writer(args, OutputMode::Text, &mut Vec::new()).unwrap_err();

    assert!(error.to_string().contains("validation evidence differs"));
}
