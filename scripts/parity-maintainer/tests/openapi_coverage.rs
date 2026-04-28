mod common;

use anyhow::Result;
use tempfile::tempdir;

use common::{RepoSpec, command, output_text, write_file, write_source_lock};

#[test]
fn openapi_coverage_generates_inventory_and_validates_rust_fixture() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    write_openapi_fixture(root)?;
    write_matching_rust_fixture(root)?;

    let generate = command()
        .current_dir(root)
        .args(["openapi-coverage", "--source-lock", "source-lock.yaml"])
        .output()?;
    assert!(generate.status.success(), "{}", output_text(&generate));

    let inventory =
        std::fs::read_to_string(root.join("parity/openapi/fixture-order-inventory.yaml"))?;
    assert!(inventory.contains("FixtureBase"));
    assert!(inventory.contains("optionalNote"));
    assert!(inventory.contains("limitPrice"));

    let validate = command()
        .current_dir(root)
        .args([
            "openapi-coverage",
            "--source-lock",
            "source-lock.yaml",
            "--validate",
        ])
        .output()?;
    assert!(validate.status.success(), "{}", output_text(&validate));
    Ok(())
}

#[test]
fn openapi_coverage_validate_reports_structured_field_mismatches() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    write_openapi_fixture(root)?;
    write_matching_rust_fixture(root)?;

    let generate = command()
        .current_dir(root)
        .args(["openapi-coverage", "--source-lock", "source-lock.yaml"])
        .output()?;
    assert!(generate.status.success(), "{}", output_text(&generate));

    write_file(
        root.join("crates/orderbook/src/lib.rs"),
        r#"
pub struct FixtureOrder {
    pub id: String,
    pub enabled: Option<bool>,
    pub amount: i64,
}
"#,
    )?;

    let validate = command()
        .current_dir(root)
        .args([
            "openapi-coverage",
            "--source-lock",
            "source-lock.yaml",
            "--validate",
        ])
        .output()?;
    assert!(!validate.status.success(), "{}", output_text(&validate));
    let text = output_text(&validate);
    assert!(text.contains("status: failed"));
    assert!(text.contains("missing_field"));
    assert!(text.contains("optionality_mismatch"));
    Ok(())
}

fn write_openapi_fixture(root: &std::path::Path) -> Result<()> {
    write_source_lock(
        &root.join("source-lock.yaml"),
        "2026-04-28T00:00:00Z",
        &[] as &[RepoSpec<'_>],
    )?;
    write_file(
        root.join("parity/openapi/coverage.yaml"),
        r#"
version: 1
dtos:
  - schema: components.schemas.FixtureOrder
    rust_type: cow_sdk_orderbook::FixtureOrder
    inventory: parity/openapi/fixture-order-inventory.yaml
    fixtures:
      - parity/fixtures/orderbook/fixture_order.json
"#,
    )?;
    write_file(
        root.join("parity/openapi/services-orderbook.yml"),
        r#"
openapi: 3.0.0
info:
  title: fixture
  version: 1.0.0
components:
  schemas:
    FixtureBase:
      type: object
      required: [id, enabled]
      properties:
        id:
          type: string
        enabled:
          type: boolean
    FixtureOrder:
      allOf:
        - $ref: '#/components/schemas/FixtureBase'
        - type: object
          required: [amount]
          properties:
            amount:
              type: integer
            optionalNote:
              type: string
            maybeOwner:
              type: string
              nullable: true
            mode:
              type: string
              default: limit
        - oneOf:
            - type: object
              properties:
                limitPrice:
                  type: string
            - type: object
              properties:
                marketPrice:
                  type: string
"#,
    )
}

fn write_matching_rust_fixture(root: &std::path::Path) -> Result<()> {
    write_file(
        root.join("crates/orderbook/src/lib.rs"),
        r#"
pub struct FixtureOrder {
    pub id: String,
    pub enabled: bool,
    pub amount: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optional_note: Option<String>,
    pub maybe_owner: Option<String>,
    #[serde(default)]
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit_price: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub market_price: Option<String>,
}
"#,
    )
}
