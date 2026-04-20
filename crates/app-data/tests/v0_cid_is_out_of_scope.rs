//! Runtime enforcement of the CID parity-scope invariant: `CIDv0`
//! (dag-pb + sha2-256) values are out of scope for the app-data
//! helpers.
//!
//! The cow-protocol services backend emits `CIDv1` with the raw
//! multicodec (`0x55`) over a keccak-256 multihash (`0x1b`) as the
//! only supported CID shape. The Rust decoder rejects every other
//! version, codec, or hash combination at the boundary with a typed
//! `AppDataError::InvalidCid`. This test asserts that the canonical
//! `Qm`-prefixed v0 sample surfaces the typed rejection, so future
//! contributors cannot reintroduce a v0 code path by mistaking a
//! missing positive fixture for an intentional parity gap.

use cow_sdk_app_data::{AppDataError, cid_to_app_data_hex};

#[test]
fn v0_cid_is_rejected_by_cid_to_app_data_hex() {
    let v0_cid = "QmYwAPJzv5CZsnA625s3Xf2nemtYgPpHdWEz79ojWnPbdG";

    let error = cid_to_app_data_hex(v0_cid).expect_err(
        "v0 CIDs must be rejected at the decoder boundary with a typed AppDataError::InvalidCid",
    );

    assert_eq!(
        error,
        AppDataError::InvalidCid,
        "v0 CIDs must surface AppDataError::InvalidCid, got {error:?}",
    );
}

#[test]
fn additional_v0_sample_is_rejected() {
    // Secondary reproducible v0 CID sample: the classic "Hello World"
    // example used throughout IPFS documentation.
    let v0_cid = "QmfM2r8seH2GiRaC4esTjeraXEachRt8ZsSeGaWTPLyMoG";

    let error = cid_to_app_data_hex(v0_cid)
        .expect_err("v0 CIDs must be rejected with a typed AppDataError::InvalidCid");

    assert_eq!(error, AppDataError::InvalidCid);
}
