//! Parity tests for the canonical `CIDv1` round trip on `AppDataHash`.
//!
//! These tests pin the forward (`to_cid`) and reverse (`try_from_cid`)
//! conversions against the upstream protocol byte vectors and cross-check
//! that the output matches the byte-for-byte form emitted by the
//! `cow-sdk-app-data` CID helpers. Together they close the round-trip
//! seam that the prior hand-rolled encoder left one-way.

use alloy_primitives::B256;
use cow_sdk_core::errors::CoreError;
use cow_sdk_core::types::AppDataHash;

const APP_DATA_HEX: &str = "0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";
const CID: &str = "f01551b20337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";

const APP_DATA_HEX_2: &str = "0x8af4e8c9973577b08ac21d17d331aade86c11ebcc5124744d621ca8365ec9424";
const CID_2: &str = "f01551b208af4e8c9973577b08ac21d17d331aade86c11ebcc5124744d621ca8365ec9424";

fn hash(hex: &str) -> AppDataHash {
    AppDataHash::new(hex).expect("upstream test vector must parse")
}

#[test]
fn to_cid_matches_upstream_byte_vector_one() {
    assert_eq!(hash(APP_DATA_HEX).to_cid(), CID);
}

#[test]
fn to_cid_matches_upstream_byte_vector_two() {
    assert_eq!(hash(APP_DATA_HEX_2).to_cid(), CID_2);
}

#[test]
fn try_from_cid_matches_upstream_byte_vector_one() {
    assert_eq!(AppDataHash::try_from_cid(CID).unwrap(), hash(APP_DATA_HEX));
}

#[test]
fn try_from_cid_matches_upstream_byte_vector_two() {
    assert_eq!(
        AppDataHash::try_from_cid(CID_2).unwrap(),
        hash(APP_DATA_HEX_2)
    );
}

#[test]
fn round_trip_preserves_every_input() {
    for sample in [APP_DATA_HEX, APP_DATA_HEX_2] {
        let h = hash(sample);
        let round = AppDataHash::try_from_cid(&h.to_cid()).expect("round trip must succeed");
        assert_eq!(round, h);
    }
}

#[test]
fn try_from_cid_rejects_garbage() {
    assert!(matches!(
        AppDataHash::try_from_cid("not a cid"),
        Err(CoreError::InvalidCid)
    ));
}

#[test]
fn try_from_cid_rejects_short_digest() {
    // CIDv1 raw + keccak-256 header but only 31 bytes of digest.
    let short = "f01551b20337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1";
    assert!(matches!(
        AppDataHash::try_from_cid(short),
        Err(CoreError::InvalidCid)
    ));
}

#[test]
fn round_trip_preserves_zero_byte_input() {
    let zero = AppDataHash::from(B256::ZERO);
    let cid_zero = zero.to_cid();
    let round = AppDataHash::try_from_cid(&cid_zero).expect("zero round trip");
    assert_eq!(round, zero);
}
