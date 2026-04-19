#![no_main]

//! Fuzz target for the app-data CID pipeline.
//!
//! Exercises three public helpers in [`cow_sdk_app_data::cid`]:
//!
//! * [`app_data_hex_to_cid`] — `CIDv1` over the keccak-256 multihash
//!   code `0x1b`.
//! * [`app_data_hex_to_cid_legacy`] — legacy `CIDv0` over the
//!   sha2-256 multihash code `0x12`.
//! * [`cid_to_app_data_hex`] — inverse that parses a multibase CID
//!   string back into the 32-byte digest hex form.
//!
//! The target invariants are:
//!
//! * On every supported 32-byte hex digest the round-trip
//!   `cid_to_app_data_hex(app_data_hex_to_cid(x)?)? == x` holds for
//!   both the latest and legacy CID paths, and the CID strings the
//!   helpers emit are parseable by the inverse helper.
//! * On every malformed input both `app_data_hex_to_cid*` helpers
//!   return a typed [`AppDataError`] without panicking, and
//!   `cid_to_app_data_hex` returns a typed [`AppDataError`] without
//!   panicking on every multibase-invalid or codec-unsupported CID
//!   string.
//!
//! The first byte of the input selects which path to exercise so a
//! single fuzz target covers both adversarial hex digests and
//! adversarial CID strings while staying panic-free.

use cow_sdk_app_data::cid::{
    app_data_hex_to_cid, app_data_hex_to_cid_legacy, cid_to_app_data_hex,
};
use libfuzzer_sys::fuzz_target;

/// Maximum input width accepted by the target. The CID helpers both
/// parse short fixed-length inputs (32-byte digests or ~60-byte CID
/// strings) in practice, so anything past the cap would only tail-pad
/// with unused bytes.
const MAX_FUZZ_INPUT: usize = 4096;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];
    let (discriminant, rest) = (data[0], &data[1..]);

    match discriminant % 3 {
        0 => fuzz_hex_to_cid_latest(rest),
        1 => fuzz_hex_to_cid_legacy(rest),
        _ => fuzz_cid_to_hex(rest),
    }
});

/// Latest-CID path: exercise the keccak-256 multihash encoder.
///
/// When `rest` carries at least 32 bytes, the first 32 bytes become a
/// well-formed `0x`-prefixed digest and the round-trip must hold.
/// Otherwise the helper is fed a malformed hex candidate and must
/// return a typed error.
fn fuzz_hex_to_cid_latest(rest: &[u8]) {
    if rest.len() >= 32 {
        let digest = &rest[..32];
        let hex = format!("0x{}", hex::encode(digest));
        let cid = app_data_hex_to_cid(&hex)
            .expect("well-formed 32-byte hex must round-trip through the latest CID helper");
        let decoded = cid_to_app_data_hex(&cid)
            .expect("latest-CID output must decode through cid_to_app_data_hex");
        assert_eq!(
            decoded.to_lowercase(),
            hex.to_lowercase(),
            "latest CID round-trip must preserve the 32-byte digest",
        );
    } else {
        // Malformed hex candidate (or truncated): helper must not panic.
        let candidate = format!("0x{}", hex::encode(rest));
        let _ = app_data_hex_to_cid(&candidate);
    }
    // Exercise with the raw bytes as a candidate UTF-8 string too so
    // the helper's hex-prefix and hex-character validation is also
    // reached from a non-canonical shape.
    if let Ok(text) = std::str::from_utf8(rest) {
        let _ = app_data_hex_to_cid(text);
    }
}

/// Legacy-CID path: exercise the sha2-256 multihash encoder.
///
/// Same shape as the latest path, routed through the alternate helper.
fn fuzz_hex_to_cid_legacy(rest: &[u8]) {
    if rest.len() >= 32 {
        let digest = &rest[..32];
        let hex = format!("0x{}", hex::encode(digest));
        let cid = app_data_hex_to_cid_legacy(&hex)
            .expect("well-formed 32-byte hex must round-trip through the legacy CID helper");
        let decoded = cid_to_app_data_hex(&cid)
            .expect("legacy-CID output must decode through cid_to_app_data_hex");
        assert_eq!(
            decoded.to_lowercase(),
            hex.to_lowercase(),
            "legacy CID round-trip must preserve the 32-byte digest",
        );
    } else {
        let candidate = format!("0x{}", hex::encode(rest));
        let _ = app_data_hex_to_cid_legacy(&candidate);
    }
    if let Ok(text) = std::str::from_utf8(rest) {
        let _ = app_data_hex_to_cid_legacy(text);
    }
}

/// Reverse path: feed arbitrary bytes as a candidate CID string and
/// assert the helper returns a typed [`AppDataError`] without
/// panicking on malformed multibase, unsupported codec, or
/// non-32-byte digest shapes.
fn fuzz_cid_to_hex(rest: &[u8]) {
    if let Ok(text) = std::str::from_utf8(rest) {
        let _ = cid_to_app_data_hex(text);
    }
}
