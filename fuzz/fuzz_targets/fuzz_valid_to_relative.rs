#![no_main]

//! Fuzz target for the typed `ValidTo::relative` validity-window
//! constructor.
//!
//! **Surface:** `cow_sdk_core::ValidTo::relative`.
//! **Property:** `PROP-CORE-003`.
//! **Seed contract:** corpus inputs cover canonical mainnet-anchored
//! happy-path anchors, zero and short durations, and adversarial inputs that
//! exercise `u64` overflow saturation and the protocol-fixed `u32` epoch
//! ceiling rejection path.
//!
//! The target derives a structured `(now, duration)` pair through
//! `Arbitrary`, runs `ValidTo::relative` twice for determinism, and asserts
//! that any accepted timestamp equals the saturating anchor-plus-duration sum
//! within the `u32` range and that any error corresponds to a sum past the
//! protocol-fixed `u32` epoch ceiling.

use cow_sdk_core::ValidTo;
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

    let projected = input.now.saturating_add(input.duration);

    match first {
        Ok(valid_to) => {
            assert_eq!(
                valid_to.as_u64(),
                projected,
                "an accepted ValidTo must equal the saturating anchor-plus-duration sum",
            );
            assert!(
                valid_to.as_u64() <= u64::from(u32::MAX),
                "ValidTo::relative output exceeded the protocol u32 epoch ceiling",
            );
            assert_eq!(
                u64::from(valid_to.as_u32()),
                valid_to.as_u64(),
                "ValidTo accessors must agree on the stored u32 epoch",
            );
        }
        Err(_) => {
            assert!(
                projected > u64::from(u32::MAX),
                "ValidTo::relative only fails closed past the protocol u32 epoch ceiling: \
                 now = {}, duration = {}",
                input.now,
                input.duration,
            );
        }
    }
});
