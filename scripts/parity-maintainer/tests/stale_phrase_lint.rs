//! Integration test for the `strategy-doc-lint` subcommand.
//!
//! Exercises the canonical stale-phrase catalog from the strategy pack
//! plus the `<!-- audit-trail-stale: ID -->` quarantine-block mechanism.

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

fn run_lint_on(text: &str) -> std::process::Output {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::write(tmp.path().join("note.md"), text).expect("write");
    Command::new(binary_path())
        .arg("strategy-doc-lint")
        .arg("--root")
        .arg(tmp.path())
        .output()
        .expect("run")
}

#[test]
fn clean_markdown_tree_passes() {
    let output = run_lint_on("# Heading\n\nNormal English prose discussing protocol invariants.\n");
    assert!(
        output.status.success(),
        "clean markdown should pass; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn corrected_strategy_phrase_outside_quarantine_fails() {
    // `Workspace already supports Lens` is one of the strategy-correction
    // phrases the lint catches.
    let output =
        run_lint_on("Workspace already supports Lens (chain 232) on the live orderbook.\n");
    assert!(
        !output.status.success(),
        "corrected phrase must fail outside a quarantine block; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn corrected_phrase_inside_audit_trail_quarantine_is_tolerated() {
    let output = run_lint_on(concat!(
        "Header line.\n",
        "<!-- audit-trail-stale: F4-001 -->\n",
        "> Workspace already supports Lens (verbatim quote for traceability).\n",
        "<!-- /audit-trail-stale -->\n",
        "Body line.\n",
    ));
    assert!(
        output.status.success(),
        "audit-trail quote should pass; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn legacy_gpt_stale_marker_is_rejected() {
    let output =
        run_lint_on("<!-- gpt-stale -->\nWorkspace already supports Lens\n<!-- /gpt-stale -->\n");
    assert!(
        !output.status.success(),
        "legacy gpt-stale marker must be rejected; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn nested_quarantine_is_rejected() {
    let output = run_lint_on(concat!(
        "<!-- audit-trail-stale: A -->\n",
        "<!-- audit-trail-stale: B -->\n",
        "inner quote\n",
        "<!-- /audit-trail-stale -->\n",
        "<!-- /audit-trail-stale -->\n",
    ));
    assert!(
        !output.status.success(),
        "nested quarantine must be rejected; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn unbalanced_open_marker_is_rejected() {
    let output = run_lint_on("<!-- audit-trail-stale: F4-001 -->\nbody with no closing marker\n");
    assert!(
        !output.status.success(),
        "unbalanced open quarantine must be rejected; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn close_without_open_is_rejected() {
    let output = run_lint_on("body\n<!-- /audit-trail-stale -->\n");
    assert!(
        !output.status.success(),
        "close marker without open must be rejected; stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
}
