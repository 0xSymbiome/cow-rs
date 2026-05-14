#![no_main]

//! Fuzz target for the `CoW` signature classifier and decoders.
//!
//! **Property:** `PROP-CON-009`.
//! Exercises three public entry points in
//! [`cow_sdk_contracts::signature`]:
//!
//! * [`SigningScheme::try_from(u8)`] and [`decode_signing_scheme`] —
//!   total functions from `u8` to `Result<SigningScheme,
//!   ContractsError>`. The fuzz target confirms both remain total
//!   across every one of the 256 possible byte values.
//! * [`decode_eip1271_signature_data`] — parses a `0x`-prefixed hex
//!   string into the compact verifier + signature layout. The fuzz
//!   target feeds arbitrary byte sequences (interpreted as candidate
//!   hex) and asserts the helper returns a typed [`ContractsError`]
//!   rather than panicking.
//! * `serde_json::from_slice::<Signature>` — the serde-derived
//!   decoder for the [`Signature`] enum. The fuzz target feeds
//!   arbitrary byte sequences as candidate UTF-8 JSON and asserts
//!   the decoder returns `Ok(Signature)` or a typed serde error
//!   without panicking.
//!
//! The target is deliberately narrow and bounded: the
//! `SigningScheme::try_from` surface is a 256-input total function,
//! and the two decoder paths receive the remaining bytes up to
//! `MAX_FUZZ_INPUT` so each run stays bounded.

use cow_sdk_contracts::{Signature, SigningScheme, decode_signing_scheme};
use cow_sdk_contracts::signature::decode_eip1271_signature_data;
use libfuzzer_sys::fuzz_target;

/// Maximum input width accepted by the target. The signature
/// classifier surfaces naturally cap at a few hundred bytes
/// (verifier plus short signature payload, or a short JSON
/// envelope) so a 256-byte cap keeps each run bounded without
/// starving coverage.
const MAX_FUZZ_INPUT: usize = 256;

fuzz_target!(|data: &[u8]| {
    if data.is_empty() {
        return;
    }
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];

    // 1. SigningScheme::try_from(u8) and decode_signing_scheme(u8) are
    //    total functions. Assert both return a Result on the first
    //    byte of the fuzz input — libFuzzer will drive every u8 value
    //    through this path as coverage expands.
    let scheme_byte = data[0];
    let via_try_from = SigningScheme::try_from(scheme_byte);
    let via_decode = decode_signing_scheme(scheme_byte);
    assert_eq!(
        via_try_from.is_ok(),
        via_decode.is_ok(),
        "SigningScheme::try_from and decode_signing_scheme must agree on acceptance",
    );
    if let (Ok(left), Ok(right)) = (&via_try_from, &via_decode) {
        assert_eq!(
            left, right,
            "SigningScheme::try_from and decode_signing_scheme must return the same variant",
        );
    }

    let rest = &data[1..];

    // 2. decode_eip1271_signature_data takes a `0x`-prefixed hex
    //    string. Feed the remaining bytes as both a hex-encoded
    //    candidate and a raw UTF-8 candidate so malformed-hex and
    //    malformed-length paths are both reached.
    let hex_candidate = format!("0x{}", hex::encode(rest));
    let _ = decode_eip1271_signature_data(&hex_candidate);
    if let Ok(text) = std::str::from_utf8(rest) {
        let _ = decode_eip1271_signature_data(text);
    }

    // 3. serde_json::from_slice::<Signature> receives the remaining
    //    bytes as a candidate JSON document. The serde-derived
    //    decoder must return `Ok(Signature)` or a typed serde error
    //    without panicking across the 256 possible first-byte shapes
    //    and every arbitrary suffix.
    let _ = serde_json::from_slice::<Signature>(rest);
});
