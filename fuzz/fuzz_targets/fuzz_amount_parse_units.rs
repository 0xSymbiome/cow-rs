#![no_main]

//! Fuzz target for the exact human-decimal amount constructor.
//!
//! **Surface:** `cow_sdk_core::Amount::parse_units` and its inverse
//! `cow_sdk_core::Amount::format_units`.
//! **Property:** `PROP-CORE-004`.
//! **Seed contract:** corpus inputs cover the canonical one and one-ether
//! values, a fractional literal, and adversarial inputs containing an
//! empty payload, a leading sign, and the smallest-representable
//! `1e-18` fractional magnitude.
//! **Corpus README:** `../corpus/fuzz_amount_parse_units/README.md`.
//!
//! The first input byte selects the `decimals` scale (any `u8`, including
//! the out-of-range values above 77 the constructor must reject); the
//! remaining bytes are decoded through `String::from_utf8_lossy` into the
//! candidate decimal string. The target invokes `Amount::parse_units`
//! twice for determinism and asserts no panic on any input. Every accepted
//! value is rendered with `format_units` at the same scale and re-parsed;
//! the round-trip must recover the originating typed amount exactly,
//! because `format_units` preserves the full `decimals`-wide fractional
//! substring with no trimming.

use cow_sdk_core::Amount;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // First byte = decimals scale; remaining bytes = candidate string.
    let (decimals, rest) = match data.split_first() {
        Some((first, rest)) => (*first, rest),
        None => (0u8, &[][..]),
    };
    let value = String::from_utf8_lossy(rest).into_owned();

    let first = Amount::parse_units(&value, decimals);
    let second = Amount::parse_units(&value, decimals);
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "Amount::parse_units must be deterministic on identical input",
    );
    if let (Some(left), Some(right)) = (first.as_ref().ok(), second.as_ref().ok()) {
        assert_eq!(
            left, right,
            "Amount::parse_units must produce identical typed values for identical input",
        );
    }

    if let Ok(amount) = first {
        // `parse_units` only returns `Ok` for `decimals <= 77`, so
        // `format_units` renders at the same scale (no clamp engaged) and
        // preserves the full fractional width. The rendered string must
        // therefore re-parse back to the originating amount exactly.
        let rendered = amount.format_units(decimals);
        let roundtrip = Amount::parse_units(&rendered, decimals)
            .expect("format_units output of an accepted Amount must re-parse");
        assert_eq!(
            amount, roundtrip,
            "Amount::parse_units round-trip through format_units must be stable: \
             value = {value}, decimals = {decimals}",
        );
    }
});
