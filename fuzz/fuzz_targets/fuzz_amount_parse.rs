#![no_main]

//! Fuzz target for the typed amount parser.
//!
//! **Surface:** `cow_sdk_core::Amount::new` plus its `serde::Deserialize`
//! impl through `serde_json`.
//! **Property:** `PROP-CORE-004`.
//! **Seed contract:** corpus inputs cover canonical decimal and `0x`-hex
//! literals, zero, the `u256` boundary, and adversarial inputs that include
//! whitespace, sign characters, and oversized digit strings.
//! **Corpus README:** `../corpus/fuzz_amount_parse/README.md`.
//!
//! The target feeds raw bytes through `String::from_utf8_lossy` into both
//! `Amount::new` and the serde JSON deserialization path, asserting that
//! every accepted value round-trips through its string form, stays within
//! the `uint256` bit-width, and is deterministic on identical input.

use cow_sdk_core::Amount;
use libfuzzer_sys::fuzz_target;

const U256_BITS: u64 = 256;

fuzz_target!(|data: &[u8]| {
    let raw = String::from_utf8_lossy(data).into_owned();

    let first = Amount::new(raw.clone());
    let second = Amount::new(raw.clone());
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "Amount::new must be deterministic on identical input",
    );

    if let (Some(left), Some(right)) = (first.as_ref().ok(), second.as_ref().ok()) {
        assert_eq!(
            left, right,
            "Amount::new must produce identical typed values for identical input",
        );
    }

    if let Ok(amount) = first {
        assert!(
            amount.as_biguint().bits() <= U256_BITS,
            "Amount::new accepted a value that exceeds the documented 256-bit boundary",
        );

        let canonical = amount.to_string();
        let roundtrip = Amount::new(canonical.clone())
            .expect("decimal-string canonical form of an accepted Amount must re-parse");
        assert_eq!(
            amount, roundtrip,
            "Amount::new round-trip through canonical decimal string must be stable",
        );

        // The hex literal `0x{biguint:x}` and the canonical decimal form must
        // parse to the same typed amount when the canonical form is non-empty.
        let hex_literal = format!("0x{}", amount.as_biguint().to_str_radix(16));
        let from_hex = Amount::new(hex_literal)
            .expect("hex literal derived from an accepted Amount must re-parse");
        assert_eq!(
            amount, from_hex,
            "Amount::new must accept hex and decimal forms of the same value identically",
        );
    }

    // Exercise the serde Deserialize path with the input wrapped as a JSON
    // string. Failures are acceptable for malformed input; no panic is allowed.
    let json_payload = serde_json::to_string(&raw)
        .expect("string serde encoding of arbitrary bytes-as-string must not panic");
    let _ = serde_json::from_str::<Amount>(&json_payload);
});
