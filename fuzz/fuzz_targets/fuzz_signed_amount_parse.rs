#![no_main]

//! Fuzz target for the typed signed-amount parser.
//!
//! **Surface:** `cow_sdk_core::SignedAmount::new`.
//! **Property:** `PROP-CORE-004`.
//! **Seed contract:** corpus inputs cover canonical zero and positive
//! decimal literals, an explicit negative literal, a very large absolute
//! magnitude beyond `i256` bounds, and adversarial inputs containing
//! whitespace, a `0x`-hex literal (signed amount is decimal-only), and the
//! empty payload.
//! **Corpus README:** `../corpus/fuzz_signed_amount_parse/README.md`.
//!
//! The target maps raw bytes through `String::from_utf8_lossy` into
//! `SignedAmount::new`, asserts no panic on any input, and asserts that
//! every accepted value round-trips through `to_string` and stays
//! deterministic on identical input.

use cow_sdk_core::SignedAmount;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let raw = String::from_utf8_lossy(data).into_owned();

    let first = SignedAmount::new(raw.clone());
    let second = SignedAmount::new(raw.clone());
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "SignedAmount::new must be deterministic on identical input",
    );
    if let (Some(left), Some(right)) = (first.as_ref().ok(), second.as_ref().ok()) {
        assert_eq!(
            left, right,
            "SignedAmount::new must produce identical typed values for identical input",
        );
    }

    if let Ok(value) = first {
        let canonical = value.to_string();
        let roundtrip = SignedAmount::new(canonical.clone())
            .expect("decimal-string canonical form of an accepted SignedAmount must re-parse");
        assert_eq!(
            value, roundtrip,
            "SignedAmount::new round-trip through canonical decimal string must be stable",
        );
    }
});
