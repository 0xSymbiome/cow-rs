#![no_main]

//! Fuzz target for the CID-to-app-data-hex inverse decoder.
//!
//! **Surface:** `cow_sdk_app_data::cid::cid_to_app_data_hex` (with
//! `cow_sdk_app_data::cid::app_data_hex_to_cid` used to validate that every
//! `Ok` return re-encodes to the same CID).
//! **Property:** `PROP-APP-001`.
//! **Seed contract:** corpus inputs cover canonical CIDv1 strings derived
//! from a 32-byte digest, multibase boundaries (empty, single byte, all-`0xff`
//! bytes, non-UTF-8 bytes), and adversarial shapes including CIDv0
//! (dag-pb + sha2-256), mismatched codecs, and non-32-byte digests.
//! **Corpus README:** `../corpus/fuzz_cid_to_app_data_hex/README.md`.
//!
//! The target invariants are:
//!
//! * `cid_to_app_data_hex` never panics on any candidate UTF-8 string built
//!   from the input bytes; CIDv0, mismatched codec, and non-32-byte digest
//!   inputs all return `Err(AppDataError::*)`.
//! * Every `Ok(hex)` return is a `0x`-prefixed 66-character ASCII string.
//! * Every `Ok(hex)` round-trips through `app_data_hex_to_cid` back to a CID
//!   string whose own `cid_to_app_data_hex` decode matches the first hex
//!   value (lowercase-normalized), and the inverse-decoded hex matches the
//!   original (so the parser is deterministic on identical input).

use cow_sdk_app_data::cid::{app_data_hex_to_cid, cid_to_app_data_hex};
use libfuzzer_sys::fuzz_target;

/// Maximum input width accepted by the target. CIDv1 strings produced by the
/// helper are ~70 ASCII bytes, so anything past the cap would only tail-pad
/// with unused bytes.
const MAX_FUZZ_INPUT: usize = 4096;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];

    // Exercise the helper as a candidate UTF-8 string. Non-UTF-8 inputs are
    // rejected before reaching the helper so the panic-free contract still
    // covers them via the early-return path.
    let Ok(candidate) = std::str::from_utf8(data) else {
        return;
    };

    let first = cid_to_app_data_hex(candidate);
    // Determinism on identical input.
    let second = cid_to_app_data_hex(candidate);
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "cid_to_app_data_hex must be deterministic on identical input",
    );

    let Ok(hex) = first else {
        return;
    };
    let second_hex = second.expect("determinism check above already proved Ok");
    assert_eq!(
        hex, second_hex,
        "cid_to_app_data_hex must return the same hex string on identical input",
    );

    // Documented shape: `0x` + 64-char hex.
    assert_eq!(
        hex.len(),
        66,
        "cid_to_app_data_hex Ok output must be a 0x-prefixed 66-char string: {hex}",
    );
    assert!(
        hex.starts_with("0x"),
        "cid_to_app_data_hex Ok output must carry the 0x prefix: {hex}",
    );
    assert!(
        hex.bytes().all(|byte| byte.is_ascii()),
        "cid_to_app_data_hex Ok output must be ASCII: {hex}",
    );

    // Round-trip: feed the extracted hex back through the encoder, then
    // decode the produced CID again and check the lower-cased hex matches.
    let cid = app_data_hex_to_cid(&hex)
        .expect("decoded 32-byte digest must round-trip through app_data_hex_to_cid");
    let decoded =
        cid_to_app_data_hex(&cid).expect("re-encoded CID must decode through cid_to_app_data_hex");
    assert_eq!(
        decoded.to_lowercase(),
        hex.to_lowercase(),
        "CID round-trip must preserve the 32-byte digest exactly",
    );
});
