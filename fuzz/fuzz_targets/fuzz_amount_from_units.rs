#![no_main]

//! Fuzz target for the exact numeric (no-string) amount constructor.
//!
//! **Surface:** `cow_sdk_core::Amount::from_units` differentially against
//! `cow_sdk_core::Amount::parse_units` (the textual constructor) and the
//! `format_units` inverse.
//! **Property:** `PROP-CORE-021`.
//! **Seed contract:** corpus inputs cover the canonical one-ether and
//! 1000-USDC whole values, zero, and adversarial inputs driving the
//! out-of-range `decimals` and the over-`uint256` overflow paths.
//! **Corpus README:** `../corpus/fuzz_amount_from_units/README.md`.
//!
//! The first input byte selects the `decimals` scale (any `u8`, including
//! the out-of-range values above 77 the constructor must reject); the next
//! up-to-sixteen bytes are decoded little-endian into the `u128` whole-unit
//! count. The target invokes `from_units` twice for determinism and asserts
//! no panic on any input. `from_units(whole, d)` and
//! `parse_units(whole.to_string(), d)` are two doors to one value, so they
//! must agree on acceptance and on the produced typed amount for every
//! input. Every accepted value is rendered with `format_units` at the same
//! scale and re-parsed; the round-trip must recover the originating amount.

use cow_sdk_core::Amount;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // First byte = decimals scale; next up-to-16 bytes = u128 whole count.
    let (decimals, rest) = match data.split_first() {
        Some((first, rest)) => (*first, rest),
        None => (0u8, &[][..]),
    };
    let mut whole_bytes = [0u8; 16];
    let take = rest.len().min(16);
    whole_bytes[..take].copy_from_slice(&rest[..take]);
    let whole = u128::from_le_bytes(whole_bytes);

    let first = Amount::from_units(whole, decimals);
    let second = Amount::from_units(whole, decimals);
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "Amount::from_units must be deterministic on identical input",
    );
    if let (Some(left), Some(right)) = (first.as_ref().ok(), second.as_ref().ok()) {
        assert_eq!(
            left, right,
            "Amount::from_units must produce identical typed values for identical input",
        );
    }

    // Differential: the numeric and textual constructors must agree on
    // acceptance and value for the same whole number.
    let textual = Amount::parse_units(whole.to_string(), decimals);
    assert_eq!(
        first.is_ok(),
        textual.is_ok(),
        "from_units and parse_units must agree on acceptance: \
         whole = {whole}, decimals = {decimals}",
    );
    if let (Ok(numeric), Ok(textual_amount)) = (first.as_ref(), textual.as_ref()) {
        assert_eq!(
            numeric, textual_amount,
            "from_units must equal parse_units of the same whole number: \
             whole = {whole}, decimals = {decimals}",
        );
    }

    if let Ok(amount) = first {
        // `from_units` only returns `Ok` for `decimals <= 77`, so
        // `format_units` renders at the same scale (no clamp engaged) and
        // the rendered string must re-parse back to the originating amount.
        let rendered = amount.format_units(decimals);
        let roundtrip = Amount::parse_units(&rendered, decimals)
            .expect("format_units output of an accepted Amount must re-parse");
        assert_eq!(
            amount, roundtrip,
            "Amount::from_units round-trip through format_units must be stable: \
             whole = {whole}, decimals = {decimals}",
        );
    }
});
