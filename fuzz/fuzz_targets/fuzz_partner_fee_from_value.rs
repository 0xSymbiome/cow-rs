#![no_main]

//! Fuzz target for the typed partner-fee JSON deserializer.
//!
//! **Surface:** `cow_sdk_app_data::PartnerFee::{from_value, to_value, validate}`
//! plus the custom `Deserialize` impl on `PartnerFeePolicy`.
//! **Property:** `PROP-APP-002`.
//! **Seed contract:** corpus inputs cover canonical single/multiple wire
//! shapes plus the legacy `{ bps, recipient }` promotion path, boundary
//! values for the `[1..=100]` and `[1..=9999]` basis-point ranges and the
//! zero-address recipient guard, and adversarial mixed-field combinations
//! and non-JSON payloads.
//!
//! The target invariants are:
//!
//! * `PartnerFee::from_value` never panics for any well-typed JSON value
//!   derived from raw fuzz bytes; non-JSON inputs return early without
//!   entering the deserializer.
//! * `PartnerFee::validate` never panics on any value the deserializer
//!   accepts; the parse-then-validate split is deliberate and `from_value`
//!   is documented as lenient on bounds.
//! * Every `Ok(parsed)` round-trips: `PartnerFee::from_value(parsed.to_value())`
//!   returns a value byte-equivalent to the original `parsed`.
//! * The deserializer is deterministic: parsing the same bytes twice produces
//!   the same `Ok`/`Err` classification and the same `Ok` value.

use cow_sdk_app_data::PartnerFee;
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

const MAX_FUZZ_INPUT: usize = 4096;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];

    let Ok(value) = serde_json::from_slice::<Value>(data) else {
        return;
    };

    let first = PartnerFee::from_value(value.clone());
    let second = PartnerFee::from_value(value.clone());

    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "PartnerFee::from_value must be deterministic on identical input",
    );

    let Ok(parsed) = first else {
        return;
    };
    let second_parsed = second.expect("determinism check above already proved Ok");
    assert_eq!(
        parsed, second_parsed,
        "PartnerFee::from_value must return the same value on identical input",
    );

    // Documented round-trip: from_value(to_value(fee)) == fee.
    let reserialized = parsed.to_value();
    let reparsed = PartnerFee::from_value(reserialized.clone())
        .expect("typed partner-fee value must reparse through from_value");
    assert_eq!(
        reparsed, parsed,
        "PartnerFee::from_value(to_value(fee)) must round-trip",
    );

    // Documented parse-then-validate split: from_value is lenient on bounds,
    // validate is the strict bounds gate. Both must be panic-free; validate
    // returns a typed Result that the fuzzer simply consumes.
    let validated = parsed.validate();
    // Calling validate a second time must return the same Result classification.
    let validated_again = parsed.validate();
    assert_eq!(
        validated.is_ok(),
        validated_again.is_ok(),
        "PartnerFee::validate must be deterministic on identical input",
    );
});
