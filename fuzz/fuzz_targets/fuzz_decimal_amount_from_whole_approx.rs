#![no_main]

//! Fuzz target for the approximate decimal-amount constructor.
//!
//! **Surface:** `cow_sdk_core::DecimalAmount::from_whole_approx`.
//! **Property:** `PROP-CORE-004`.
//! **Seed contract:** corpus inputs cover the canonical zero and one-ether
//! values, decimal-scale endpoints, and adversarial inputs containing
//! NaN, negative magnitudes, and the documented `f64` extremes the
//! constructor must clamp to the documented zero-atoms output.
//! **Corpus README:** `../corpus/fuzz_decimal_amount_from_whole_approx/README.md`.
//!
//! The target derives a structured `(whole_units, decimals)` pair through
//! `Arbitrary`, invokes the constructor twice for determinism, and asserts
//! that NaN, infinite, and negative inputs produce the documented
//! zero-atoms output while every other accepted input keeps the decimals
//! scale verbatim.

use cow_sdk_core::DecimalAmount;
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};

#[derive(Debug)]
struct DecimalInput {
    whole_units: f64,
    decimals: u8,
}

impl<'a> Arbitrary<'a> for DecimalInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        Ok(Self {
            whole_units: f64::arbitrary(bytes).unwrap_or(0.0),
            decimals: u8::arbitrary(bytes).unwrap_or(0),
        })
    }
}

fuzz_target!(|input: DecimalInput| {
    // `DecimalAmount::from_whole_approx` returns `Err` when
    // `decimals > MAX_DECIMALS == 77`. Skip those inputs: the
    // construction-time boundary contract is pinned by the dedicated
    // contract test, and the determinism + clamping properties this
    // fuzz target exercises only apply to inputs the constructor
    // accepts.
    let Ok(first) = DecimalAmount::from_whole_approx(input.whole_units, input.decimals) else {
        return;
    };
    let second = DecimalAmount::from_whole_approx(input.whole_units, input.decimals)
        .expect("identical input must succeed when the first call succeeded");
    assert_eq!(
        first, second,
        "DecimalAmount::from_whole_approx must be deterministic on identical input",
    );

    assert_eq!(
        first.decimals(),
        input.decimals,
        "DecimalAmount::from_whole_approx must preserve the supplied decimals scale verbatim",
    );

    if !input.whole_units.is_finite() || input.whole_units < 0.0 {
        let zero = DecimalAmount::from_whole_approx(0.0, input.decimals)
            .expect("identical decimals must succeed when the first call succeeded");
        assert_eq!(
            first.atoms(),
            zero.atoms(),
            "DecimalAmount::from_whole_approx must clamp NaN, Inf, and negative \
             magnitudes to the documented zero-atoms output: whole_units = {}, \
             decimals = {}",
            input.whole_units,
            input.decimals,
        );
    }
});
