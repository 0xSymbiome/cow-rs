mod common;

use anyhow::Result;
use tempfile::tempdir;

use common::{command, output_text, write_file};

#[test]
fn openapi_coverage_validates_matching_rust_fixture() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    write_openapi_fixture(root)?;
    write_matching_rust_fixture(root)?;

    let validate = command()
        .current_dir(root)
        .args(["parity", "openapi-coverage"])
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

    write_file(
        root.join("crates/orderbook/src/lib.rs"),
        r"
pub struct FixtureOrder {
    pub id: String,
    pub enabled: Option<bool>,
    pub amount: i64,
}
",
    )?;

    let validate = command()
        .current_dir(root)
        .args(["parity", "openapi-coverage"])
        .output()?;
    assert!(!validate.status.success(), "{}", output_text(&validate));
    let text = output_text(&validate);
    assert!(text.contains("status: failed"));
    assert!(text.contains("missing_field"));
    assert!(text.contains("optionality_mismatch"));
    Ok(())
}

#[test]
fn openapi_coverage_validate_reports_required_field_drift() -> Result<()> {
    let temp = tempdir()?;
    let root = temp.path();
    write_openapi_fixture(root)?;
    write_matching_rust_fixture(root)?;

    write_file(
        root.join("parity/openapi/coverage.yaml"),
        r"
version: 1
dtos:
  - schema: components.schemas.FixtureOrder
    rust_type: cow_sdk_orderbook::FixtureOrder
    required_fields:
      - id
      - enabled
",
    )?;

    let validate = command()
        .current_dir(root)
        .args(["parity", "openapi-coverage"])
        .output()?;
    assert!(!validate.status.success(), "{}", output_text(&validate));
    assert!(output_text(&validate).contains("required_fields_mismatch"));
    Ok(())
}

fn write_openapi_fixture(root: &std::path::Path) -> Result<()> {
    write_file(
        root.join("parity/openapi/coverage.yaml"),
        r"
version: 1
dtos:
  - schema: components.schemas.FixtureOrder
    rust_type: cow_sdk_orderbook::FixtureOrder
    required_fields:
      - id
      - enabled
      - amount
",
    )?;
    write_file(
        root.join("parity/openapi/services-orderbook.yml"),
        r"
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
",
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
