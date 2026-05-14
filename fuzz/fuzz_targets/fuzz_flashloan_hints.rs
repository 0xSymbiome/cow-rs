#![no_main]

//! Fuzz target for the typed flash-loan hints deserializer.
//!
//! **Surface:** `cow_sdk_app_data::FlashloanHints` deserializer and
//! `FlashloanHints::validate`.
//! **Property:** `PROP-APP-002`.
//! **Seed contract:** corpus inputs cover the canonical five-field
//! camelCase shape, boundary values for the zero-amount and zero-address
//! guards, and adversarial payloads including unknown top-level keys,
//! missing required fields, and non-JSON bytes.
//! **Corpus README:** `../corpus/fuzz_flashloan_hints/README.md`.
//!
//! The target invariants are:
//!
//! * The deserializer never panics on any candidate JSON value derived from
//!   raw fuzz bytes; non-JSON inputs return early without entering serde.
//! * `FlashloanHints::validate` never panics on any value the deserializer
//!   accepts; the derived `Deserialize` impl is shape-only and the strict
//!   non-zero-amount and non-zero-address gates live in `validate`.
//! * `from_value(to_value(hints)) == hints` round-trips byte-identically
//!   for every successfully parsed hint.
//! * The `deny_unknown_fields` attribute is enforced: any extra top-level
//!   key forces an `Err`. The target asserts this by re-running the parse
//!   after injecting a sentinel key.

use cow_sdk_app_data::FlashloanHints;
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

const MAX_FUZZ_INPUT: usize = 4096;
const UNKNOWN_KEY: &str = "__fuzz_unknown_field__";

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];

    let Ok(value) = serde_json::from_slice::<Value>(data) else {
        return;
    };

    // Determinism: parsing the same value twice must produce the same outcome.
    let first = serde_json::from_value::<FlashloanHints>(value.clone());
    let second = serde_json::from_value::<FlashloanHints>(value.clone());
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "FlashloanHints deserializer must be deterministic on identical input",
    );

    let Ok(hints) = first else {
        return;
    };
    let second_hints = second.expect("determinism check above already proved Ok");
    assert_eq!(
        hints, second_hints,
        "FlashloanHints deserializer must return the same value on identical input",
    );

    // Documented shape-vs-bounds split: the derived Deserialize impl is
    // shape-only and validate() is the strict gate. Both must be panic-free;
    // validate returns a typed Result that the fuzzer simply consumes.
    let validated = hints.validate();
    let validated_again = hints.validate();
    assert_eq!(
        validated.is_ok(),
        validated_again.is_ok(),
        "FlashloanHints::validate must be deterministic on identical input",
    );

    // Round-trip: from_value(to_value(hints)) == hints.
    let reserialized = serde_json::to_value(&hints)
        .expect("typed FlashloanHints must serialize through serde_json");
    let reparsed = serde_json::from_value::<FlashloanHints>(reserialized.clone())
        .expect("typed FlashloanHints value must reparse through serde_json");
    assert_eq!(
        reparsed, hints,
        "FlashloanHints to_value/from_value must round-trip",
    );

    // deny_unknown_fields: injecting an extra top-level key must reject.
    if let Value::Object(mut map) = reserialized {
        let injected = Value::String("sentinel".to_owned());
        // Loop in case the sentinel key collides with an existing field name
        // on some future schema expansion.
        let mut sentinel_key = UNKNOWN_KEY.to_owned();
        let mut counter: u32 = 0;
        while map.contains_key(&sentinel_key) {
            counter = counter.wrapping_add(1);
            sentinel_key = format!("{UNKNOWN_KEY}_{counter}");
        }
        map.insert(sentinel_key, injected);
        let with_unknown = Value::Object(map);
        let denied = serde_json::from_value::<FlashloanHints>(with_unknown);
        assert!(
            denied.is_err(),
            "deny_unknown_fields must reject any extra top-level key",
        );
    } else {
        // Successful parse must have produced an object on serialization.
        panic!("FlashloanHints serialize must produce a JSON object");
    }

    // Determinism guard: stringify and re-parse from the canonical text form.
    let serialized = serde_json::to_string(&hints)
        .expect("typed FlashloanHints must serialize to a JSON string");
    let from_string = serde_json::from_str::<FlashloanHints>(&serialized).expect(
        "typed FlashloanHints serialized form must reparse through serde_json::from_str",
    );
    assert_eq!(
        from_string, hints,
        "FlashloanHints to_string/from_str must round-trip",
    );
});
