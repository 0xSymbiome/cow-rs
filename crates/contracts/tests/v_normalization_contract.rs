//! ECDSA recovery-byte canonicalisation parity contract.
//!
//! Drives the canonical v-byte mapping rows and the rejection rows in
//! `parity/fixtures/ecdsa/v_normalization.json` against
//! [`cow_sdk_contracts::RecoverableSignature::parse_hex`]. The accept
//! rows pin `v ∈ {0, 1, 27, 28} → {27, 28}` per ADR 0022; the rejection
//! rows pin the typed [`ContractsError::InvalidSignatureRecoveryByte`]
//! discriminant for every other trailing byte, including the EIP-155
//! chain-encoded range `35..=255` that the wider alloy parity-normalization
//! path would otherwise admit.

use cow_sdk_contracts::{ContractsError, RecoverableSignature};
use serde::Deserialize;

const FIXTURE: &str = include_str!("../../../parity/fixtures/ecdsa/v_normalization.json");

#[derive(Debug, Deserialize)]
struct Fixture {
    schema_version: u32,
    rows: Vec<Row>,
    rejection_rows: Vec<RejectionRow>,
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

#[derive(Debug, Deserialize)]
struct RejectionRow {
    name: String,
    inputs: Inputs,
    expected: RejectionExpected,
}

#[derive(Debug, Deserialize)]
struct RejectionExpected {
    error_discriminant: String,
    value: u16,
}

#[test]
fn v_normalization_fixture_rows_hold() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("v_normalization fixture parses");
    assert_eq!(fixture.schema_version, 2, "fixture schema version pinned");
    assert!(
        !fixture.rows.is_empty(),
        "v_normalization fixture must carry at least one accept row"
    );
    assert!(
        !fixture.rejection_rows.is_empty(),
        "v_normalization fixture must carry at least one rejection row"
    );

    for row in &fixture.rows {
        let actual = RecoverableSignature::parse_hex(&row.inputs.raw_signature)
            .unwrap_or_else(|err| {
                panic!(
                    "row {}: RecoverableSignature::parse_hex rejected the canonical input: {err}",
                    row.name
                )
            })
            .to_hex_string();
        assert_eq!(
            actual, row.expected.normalized_signature,
            "row {}: normalized signature must match the fixture",
            row.name
        );
    }

    for row in &fixture.rejection_rows {
        let error = RecoverableSignature::parse_hex(&row.inputs.raw_signature)
            .expect_err("rejection row must fail through ContractsError");
        assert_eq!(
            row.expected.error_discriminant, "InvalidSignatureRecoveryByte",
            "row {}: only the v-byte rejection discriminant is supported by this contract",
            row.name
        );
        let expected_value =
            u8::try_from(row.expected.value).expect("fixture rejection v-byte must fit in u8");
        match error {
            ContractsError::InvalidSignatureRecoveryByte { value } => {
                assert_eq!(
                    value, expected_value,
                    "row {}: rejection v-byte must match the fixture",
                    row.name
                );
            }
            other => panic!(
                "row {}: expected InvalidSignatureRecoveryByte, got {other:?}",
                row.name
            ),
        }
    }
}
