//! ECDSA recovery-byte normalization parity contract.
//!
//! Drives the canonical v-byte mapping rows in
//! `parity/fixtures/ecdsa/v_normalization.json` against
//! [`cow_sdk_contracts::normalized_ecdsa_signature`]. The helper
//! delegates to `alloy_primitives::Signature::from_raw` plus
//! `Signature::as_bytes`; this contract pins the public byte mapping
//! ADR 0022 specifies (`v ∈ {0, 1, 27, 28} → {27, 28}`) and rejects
//! any future drift in the alloy primitive's parity-encoding rules.

use cow_sdk_contracts::normalized_ecdsa_signature;
use serde::Deserialize;

const FIXTURE: &str = include_str!("../../../parity/fixtures/ecdsa/v_normalization.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    schema_version: u32,
    rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
struct Row {
    name: String,
    inputs: Inputs,
    expected: Expected,
}

#[derive(Debug, Deserialize)]
struct Inputs {
    raw_signature: String,
}

#[derive(Debug, Deserialize)]
struct Expected {
    normalized_signature: String,
}

#[test]
fn v_normalization_fixture_rows_hold() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("v_normalization fixture parses");
    assert_eq!(fixture.schema_version, 1, "fixture schema version pinned");
    assert!(
        !fixture.rows.is_empty(),
        "v_normalization fixture must carry at least one row"
    );

    for row in &fixture.rows {
        let actual = normalized_ecdsa_signature(&row.inputs.raw_signature).unwrap_or_else(|err| {
            panic!(
                "row {}: normalized_ecdsa_signature rejected the canonical input: {err}",
                row.name
            )
        });
        assert_eq!(
            actual, row.expected.normalized_signature,
            "row {}: normalized signature must match the fixture",
            row.name
        );
    }
}
