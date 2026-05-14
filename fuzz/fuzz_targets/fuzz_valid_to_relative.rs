#![no_main]

//! Fuzz target for the typed `ValidTo::relative` validity-window
//! constructor.
//!
//! **Surface:** `cow_sdk_core::ValidTo::relative`.
//! **Property:** `PROP-CORE-003`.
//! **Seed contract:** corpus inputs cover canonical mainnet-anchored
//! happy-path windows, boundary durations at the inclusive
//! `[VALID_TO_MIN_RELATIVE_SECONDS, VALID_TO_MAX_RELATIVE_SECONDS]`
//! endpoints, and adversarial inputs that exercise overflow saturation,
//! zero duration, and the documented out-of-range rejection path.
//! **Corpus README:** `../corpus/fuzz_valid_to_relative/README.md`.
//!
//! The target derives a structured `(now, duration)` pair through
//! `Arbitrary`, runs `ValidTo::relative` twice for determinism, and asserts
//! that any accepted timestamp fits the `u32` range and that any error path
//! corresponds to a duration outside the inclusive
//! `[VALID_TO_MIN_RELATIVE_SECONDS, VALID_TO_MAX_RELATIVE_SECONDS]` window.

use cow_sdk_core::{VALID_TO_MAX_RELATIVE_SECONDS, VALID_TO_MIN_RELATIVE_SECONDS, ValidTo};
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};

#[derive(Debug)]
struct ValidToInput {
    now: u64,
    duration: u64,
}

impl<'a> Arbitrary<'a> for ValidToInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        Ok(Self {
            now: u64::arbitrary(bytes).unwrap_or(0),
            duration: u64::arbitrary(bytes).unwrap_or(0),
        })
    }
}

fuzz_target!(|input: ValidToInput| {
    let first = ValidTo::relative(input.now, input.duration);
    let second = ValidTo::relative(input.now, input.duration);
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "ValidTo::relative must be deterministic on identical input",
    );
    if let (Some(left), Some(right)) = (first.as_ref().ok(), second.as_ref().ok()) {
        assert_eq!(
            left, right,
            "ValidTo::relative must produce identical typed values for identical input",
        );
    }

    let min = u64::from(VALID_TO_MIN_RELATIVE_SECONDS);
    let max = u64::from(VALID_TO_MAX_RELATIVE_SECONDS);
    let duration_in_window = (min..=max).contains(&input.duration);

    match first {
        Ok(valid_to) => {
            assert!(
                duration_in_window,
                "ValidTo::relative accepted a duration outside the documented \
                 [VALID_TO_MIN_RELATIVE_SECONDS, VALID_TO_MAX_RELATIVE_SECONDS] window: \
                 duration = {}",
                input.duration,
            );
            assert!(
                valid_to.as_u64() <= u64::from(u32::MAX),
                "ValidTo::relative output exceeded the documented `u32` ceiling",
            );
            assert_eq!(
                u64::from(valid_to.as_u32()),
                valid_to.as_u64(),
                "ValidTo accessors must agree on the stored `u32` epoch",
            );
        }
        Err(_) => {
            assert!(
                !duration_in_window,
                "ValidTo::relative rejected a duration inside the documented \
                 [VALID_TO_MIN_RELATIVE_SECONDS, VALID_TO_MAX_RELATIVE_SECONDS] window: \
                 duration = {}",
                input.duration,
            );
        }
    }
});
