#![no_main]

//! Fuzz target for the string-typed `RecoverableSignature` constructor.
//!
//! **Surface:** `cow_sdk_contracts::RecoverableSignature::parse_hex(&str)`.
//! **Property:** `PROP-CON-005`.
//! **Corpus README:** `../corpus/fuzz_recoverable_signature_parse_hex/README.md`.
//!
//! Drives arbitrary bytes as a UTF-8 lossy string into the typestate
//! constructor. Asserts:
//!
//! * `RecoverableSignature::parse_hex(&str)` never panics for any input.
//! * Accepted canonical outputs are exactly 65 bytes, lowercase,
//!   `0x`-prefixed hex, and carry `v ∈ {27, 28}`.
//! * `RecoverableSignature::parse_hex` is idempotent on its own
//!   canonical output: `parse_hex(parse_hex(x)?.to_hex_string())`
//!   yields the same canonical string.

use cow_sdk_contracts::RecoverableSignature;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let candidate = String::from_utf8_lossy(data).into_owned();

    let Ok(parsed) = RecoverableSignature::parse_hex(&candidate) else {
        return;
    };
    let first = parsed.to_hex_string();

    assert!(
        first.starts_with("0x"),
        "accepted canonical signature must keep the 0x prefix, got {first:?}",
    );
    let stripped = &first[2..];
    assert_eq!(
        stripped.len(),
        130,
        "accepted signature hex body must be 130 ASCII chars (65 bytes), got {} chars",
        stripped.len(),
    );
    assert!(
        stripped
            .bytes()
            .all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f')),
        "accepted canonical signature must be lowercase hex, got {first:?}",
    );

    let bytes = parsed.to_bytes();
    assert!(
        matches!(bytes[64], 27 | 28),
        "canonical signature v byte must be 27 or 28, got {}",
        bytes[64],
    );

    let second = RecoverableSignature::parse_hex(&first)
        .expect("canonical output must round-trip through parse_hex")
        .to_hex_string();
    assert_eq!(
        first, second,
        "RecoverableSignature::parse_hex must be idempotent on its own canonical output",
    );
});
