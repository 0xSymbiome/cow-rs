mod common;

use std::path::{Path, PathBuf};

use anyhow::Result;
use tempfile::tempdir;

use common::{command, output_text, write_file};

const FIXTURE_V2: &str = include_str!("fixtures/source-lock-v2.yaml");
const FIXTURE_V3: &str = include_str!("fixtures/source-lock-v3.yaml");
const FIXTURE_V4: &str = include_str!("fixtures/source-lock-v4.yaml");

const STABLE_DIAGNOSTIC: &str = "expected source-lock schema_version 3";

#[test]
fn source_lock_with_schema_v2_is_rejected_with_stable_diagnostic() -> Result<()> {
    assert_unsupported_schema_is_rejected(FIXTURE_V2)
}

#[test]
fn source_lock_with_schema_v3_is_accepted() -> Result<()> {
    let temp = tempdir()?;
    write_file(temp.path().join("source-lock.yaml"), FIXTURE_V3)?;
    write_schema_bundle_fixture(temp.path())?;

    let output = command()
        .current_dir(temp.path())
        .args(["validate", "--source-lock", "source-lock.yaml"])
        .output()?;

    assert!(output.status.success(), "{}", output_text(&output));
    Ok(())
}

#[test]
fn source_lock_with_schema_v4_is_rejected_with_stable_diagnostic() -> Result<()> {
    assert_unsupported_schema_is_rejected(FIXTURE_V4)
}

fn assert_unsupported_schema_is_rejected(fixture: &str) -> Result<()> {
    let temp = tempdir()?;
    write_file(temp.path().join("source-lock.yaml"), fixture)?;

    let output = command()
        .current_dir(temp.path())
        .args(["validate", "--source-lock", "source-lock.yaml"])
        .output()?;

    assert!(!output.status.success(), "{}", output_text(&output));
    let text = output_text(&output);
    assert!(
        text.contains(STABLE_DIAGNOSTIC),
        "output did not contain stable diagnostic: {text}"
    );
    Ok(())
}

fn write_schema_bundle_fixture(root: &Path) -> Result<()> {
    write_file(
        PathBuf::from(root).join("crates/app-data/schemas/definitions.json"),
        "{\"type\":\"object\"}\n",
    )
}
