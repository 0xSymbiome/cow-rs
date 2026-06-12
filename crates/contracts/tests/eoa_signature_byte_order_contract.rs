#![cfg(feature = "cow-shed")]

//! ERC-2098 compact ↔ canonical 65-byte signature parity.
//!
//! Drives the rows in `parity/fixtures/cow_shed/eoa_signature_byte_order.json`
//! against [`cow_sdk_contracts::RecoverableSignature`]'s alloy-backed pair
//! ([`to_erc2098`](cow_sdk_contracts::RecoverableSignature::to_erc2098) /
//! [`parse_erc2098`](cow_sdk_contracts::RecoverableSignature::parse_erc2098)):
//! encoding normalizes `s` to low-s per BIP-62 first — a high-s input maps to
//! its canonical twin `(r, n − s, !y_parity)`, which verifies for the same
//! digest and signer under ECDSA malleability — and decoding emits the
//! canonical `r || s || v` with `v ∈ {27, 28}`, the only EOA shape the
//! on-chain `decodeEOASignature` accepts.

use cow_sdk_contracts::RecoverableSignature;
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
    packed_signature: String,
    compact_2098: String,
    canonical_packed_signature: String,
}

#[test]
fn compact_round_trips_through_the_canonical_twin() {
    let fixture: Fixture = serde_json::from_str(FIXTURE).expect("fixture parses");
    assert!(
        !fixture.rows.is_empty(),
        "fixture must carry at least one row"
    );

    for row in &fixture.rows {
        let packed = decode_hex(&row.packed_signature);
        let compact = decode_hex(&row.compact_2098);
        let canonical = decode_hex(&row.canonical_packed_signature);
        assert_eq!(packed.len(), 65, "row {}: packed length", row.name);
        assert_eq!(compact.len(), 64, "row {}: compact length", row.name);

        let signature = RecoverableSignature::parse_bytes(&packed)
            .unwrap_or_else(|err| panic!("row {}: packed signature parses: {err:?}", row.name));
        assert_eq!(
            signature.to_erc2098().as_slice(),
            compact.as_slice(),
            "row {}: encode normalizes to the pinned ERC-2098 bytes",
            row.name
        );

        let twin = RecoverableSignature::parse_erc2098(&compact)
            .unwrap_or_else(|err| panic!("row {}: compact parses: {err:?}", row.name));
        assert_eq!(
            twin.to_bytes().as_slice(),
            canonical.as_slice(),
            "row {}: decode emits the canonical 65-byte form",
            row.name
        );
        assert_eq!(
            twin.to_erc2098().as_slice(),
            compact.as_slice(),
            "row {}: the canonical twin re-encodes to the same compact bytes",
            row.name
        );
    }
}

fn decode_hex(value: &str) -> Vec<u8> {
    alloy_primitives::hex::decode(value.trim_start_matches("0x")).expect("hex fixture parses")
}
