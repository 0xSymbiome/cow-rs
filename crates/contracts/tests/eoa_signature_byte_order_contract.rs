#![cfg(feature = "cow-shed")]

//! EOA EIP-2098 compact-signature decoder parity contract.
//!
//! Drives the rows in
//! `parity/fixtures/cow_shed/eoa_signature_byte_order.json` against
//! [`cow_sdk_contracts::cow_shed::eoa_signature_from_compact`], which delegates to
//! [`alloy_primitives::Signature::from_erc2098`] and
//! [`alloy_primitives::Signature::as_bytes`]. Each row carries the
//! split `r` and `s` plus the canonical `v ∈ {27, 28}` and the
//! pre-composed 64-byte ERC-2098 input; the test asserts the cow
//! decoder emits the canonical 65-byte `r || s || v` form
//! byte-identically.

use cow_sdk_contracts::cow_shed::eoa_signature_from_compact;
use serde::Deserialize;

const FIXTURE: &str =
    include_str!("../../../parity/fixtures/cow_shed/eoa_signature_byte_order.json");

#[derive(Debug, Deserialize)]
struct Fixture {
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
    r: String,
    #[expect(
        dead_code,
        reason = "field participates in the serde deserialization shape that mirrors the parity fixture row layout but the contract assertion path only exercises the compact_2098 form and the r byte"
    )]
    s: String,
    #[expect(
        dead_code,
        reason = "field participates in the serde deserialization shape that mirrors the parity fixture row layout but the contract assertion path only exercises the compact_2098 form and the r byte"
    )]
    v: u8,
    compact_2098: String,
}

#[derive(Debug, Deserialize)]
struct Expected {
    packed_signature: String,
}

#[test]
fn eoa_signature_compact_fixture_rows_hold() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("fixture parses");
    assert!(
        !fixture.rows.is_empty(),
        "fixture must carry at least one row"
    );

    for row in &fixture.rows {
        let compact_bytes = decode_hex(&row.inputs.compact_2098);
        assert_eq!(
            compact_bytes.len(),
            64,
            "row {}: compact_2098 must decode to 64 bytes",
            row.name
        );

        // The cow function accepts (r, vs) split inputs; reconstruct
        // from the canonical compact_2098 byte string.
        let mut r_arr = [0_u8; 32];
        r_arr.copy_from_slice(&compact_bytes[..32]);
        let mut vs_arr = [0_u8; 32];
        vs_arr.copy_from_slice(&compact_bytes[32..]);

        let actual = eoa_signature_from_compact(&r_arr, &vs_arr);
        let expected = decode_hex(&row.expected.packed_signature);
        assert_eq!(expected.len(), 65, "row {}: expected length", row.name);
        assert_eq!(
            actual.as_slice(),
            expected.as_slice(),
            "row {}: packed_signature must match the fixture",
            row.name
        );

        // Sanity: r in compact_2098 matches inputs.r.
        let r_input = decode_hex(&row.inputs.r);
        assert_eq!(
            r_input.as_slice(),
            &compact_bytes[..32],
            "row {}: r",
            row.name
        );
    }
}

fn decode_hex(value: &str) -> Vec<u8> {
    alloy_primitives::hex::decode(value.trim_start_matches("0x")).expect("hex fixture parses")
}
