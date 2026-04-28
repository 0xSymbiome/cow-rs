mod common;

use anyhow::Result;
use tempfile::tempdir;

use common::{
    RepoSpec, command, commit_all, init_git_repo, output_text, write_file, write_source_lock,
};

#[test]
fn diff_upstreams_reports_synthetic_two_commit_producer_diff() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    let upstream = root.join("services-upstream");
    init_git_repo(&upstream, None)?;
    write_file(
        upstream.join("crates/orderbook/openapi.yml"),
        "components:\n  schemas:\n    Order:\n      type: object\n",
    )?;
    let pinned = commit_all(&upstream, "initial openapi")?;
    write_file(
        upstream.join("crates/orderbook/openapi.yml"),
        "components:\n  schemas:\n    Order:\n      type: object\n      properties:\n        newOptional:\n          type: string\n",
    )?;
    commit_all(&upstream, "add optional field")?;

    write_source_lock(
        &root.join("source-lock.yaml"),
        "2026-04-28T00:00:00Z",
        &[RepoSpec {
            id: "services",
            remote: upstream.display().to_string(),
            commit: pinned,
            role: "reference-only",
            producer_paths: vec!["crates/orderbook/openapi.yml"],
        }],
    )?;

    let output = command()
        .current_dir(root)
        .args([
            "diff-upstreams",
            "--source-lock",
            "source-lock.yaml",
            "--output",
            "report.md",
        ])
        .output()?;
    assert!(output.status.success(), "{}", output_text(&output));
    let report = std::fs::read_to_string(root.join("report.md"))?;
    assert!(report.contains("crates/orderbook/openapi.yml"));
    assert!(report.contains("additive-safe"));
    Ok(())
}

#[test]
fn diff_upstreams_rejects_unreachable_upstream_remote() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    write_source_lock(
        &root.join("source-lock.yaml"),
        "2026-04-28T00:00:00Z",
        &[RepoSpec {
            id: "services",
            remote: root.join("missing-upstream").display().to_string(),
            commit: "0000000000000000000000000000000000000000".to_string(),
            role: "reference-only",
            producer_paths: vec!["crates/orderbook/openapi.yml"],
        }],
    )?;

    let output = command()
        .current_dir(root)
        .args([
            "diff-upstreams",
            "--source-lock",
            "source-lock.yaml",
            "--output",
            "report.md",
        ])
        .output()?;
    assert!(!output.status.success(), "{}", output_text(&output));
    assert!(output_text(&output).contains("failed to query upstream HEAD"));
    Ok(())
}
