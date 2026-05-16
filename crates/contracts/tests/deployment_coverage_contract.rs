//! Deployment-coverage contract test: assert
//! `deployment-coverage.yaml` carries exactly 24 absence/exclusion
//! records (10 Ink `not_deployed` + 14 Optimism `not_supported`) and
//! that every record points at a real contract id.

use std::path::PathBuf;

#[derive(Debug)]
#[allow(
    dead_code,
    reason = "contract_id is read by Debug output in panic messages"
)]
struct CoverageRow {
    contract_id: String,
    chain_id: u64,
    status: String,
}

fn coverage_records() -> Vec<CoverageRow> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("deployment-coverage.yaml");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    let manifest: serde_yaml::Value = serde_yaml::from_str(&text).expect("valid yaml or json");
    let coverage = manifest
        .get("coverage")
        .and_then(|v| v.as_sequence())
        .expect("coverage array missing");
    coverage
        .iter()
        .map(|row| CoverageRow {
            contract_id: row
                .get("contract_id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            chain_id: row
                .get("chain_id")
                .and_then(serde_yaml::Value::as_u64)
                .unwrap_or(0),
            status: row
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
        })
        .collect()
}

#[test]
fn total_coverage_records_equals_twenty_four() {
    let rows = coverage_records();
    assert_eq!(
        rows.len(),
        24,
        "expected 24 coverage records (10 Ink not_deployed + 14 Optimism not_supported); got {}",
        rows.len()
    );
}

#[test]
fn ink_records_use_not_deployed_status() {
    let ink_rows: Vec<_> = coverage_records()
        .into_iter()
        .filter(|row| row.chain_id == 57_073)
        .collect();
    assert_eq!(
        ink_rows.len(),
        10,
        "expected 10 Ink coverage records; got {}",
        ink_rows.len()
    );
    for row in ink_rows {
        assert_eq!(
            row.status, "not_deployed",
            "Ink coverage records must be not_deployed; row {row:?}"
        );
    }
}

#[test]
fn optimism_records_use_not_supported_status() {
    let op_rows: Vec<_> = coverage_records()
        .into_iter()
        .filter(|row| row.chain_id == 10)
        .collect();
    assert_eq!(
        op_rows.len(),
        14,
        "expected 14 Optimism coverage records; got {}",
        op_rows.len()
    );
    for row in op_rows {
        assert_eq!(
            row.status, "not_supported",
            "Optimism coverage records must be not_supported; row {row:?}"
        );
    }
}
