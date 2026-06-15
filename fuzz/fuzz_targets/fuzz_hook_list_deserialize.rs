#![no_main]

//! Fuzz target for the typed hooks metadata deserializer.
//!
//! **Surface:** `cow_sdk_app_data::HookList` deserializer and the embedded
//! `gas_limit_serde::deserialize` helper that promotes a decimal `gasLimit`
//! string into a `u64`.
//! **Property:** `PROP-APP-002`.
//! **Seed contract:** corpus inputs cover the canonical pre/post envelope
//! shape, boundary `gasLimit` values (zero, decimal max u64, leading
//! whitespace), and adversarial payloads including unknown top-level keys,
//! oversized gas limits (greater than `u64::MAX`), and non-JSON bytes.
//!
//! The target invariants are:
//!
//! * The deserializer never panics on any candidate JSON value derived from
//!   raw fuzz bytes; non-JSON inputs return early without entering serde.
//! * Determinism: parsing the same value twice produces the same outcome.
//! * Round-trip: every `Ok(list)` re-serializes to a `Value` that
//!   deserializes back to the same `list`.
//! * `deny_unknown_fields` is enforced on both the outer `HookList` and the
//!   inner `Hook` struct: any extra top-level key forces an `Err`.

use cow_sdk_app_data::HookList;
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
    let first = serde_json::from_value::<HookList>(value.clone());
    let second = serde_json::from_value::<HookList>(value.clone());
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "HookList deserializer must be deterministic on identical input",
    );

    let Ok(list) = first else {
        return;
    };
    let second_list = second.expect("determinism check above already proved Ok");
    assert_eq!(
        list, second_list,
        "HookList deserializer must return the same value on identical input",
    );

    // Round-trip: every Ok(list) re-serializes to a Value that deserializes
    // back to the same list.
    let reserialized = serde_json::to_value(&list).expect(
        "typed HookList must serialize through serde_json (gasLimit u64 -> decimal string)",
    );
    let reparsed = serde_json::from_value::<HookList>(reserialized.clone())
        .expect("typed HookList value must reparse through serde_json");
    assert_eq!(
        reparsed, list,
        "HookList to_value/from_value must round-trip",
    );

    // deny_unknown_fields on the outer struct: injecting an extra top-level
    // key must reject.
    if let Value::Object(mut map) = reserialized {
        let injected = Value::String("sentinel".to_owned());
        let mut sentinel_key = UNKNOWN_KEY.to_owned();
        let mut counter: u32 = 0;
        while map.contains_key(&sentinel_key) {
            counter = counter.wrapping_add(1);
            sentinel_key = format!("{UNKNOWN_KEY}_{counter}");
        }
        map.insert(sentinel_key, injected);
        let with_unknown = Value::Object(map);
        let denied = serde_json::from_value::<HookList>(with_unknown);
        assert!(
            denied.is_err(),
            "deny_unknown_fields must reject any extra top-level key on HookList",
        );
    }

    // Stringified round-trip: ensures the gas-limit string form is canonical.
    let serialized =
        serde_json::to_string(&list).expect("typed HookList must serialize to a JSON string");
    let from_string = serde_json::from_str::<HookList>(&serialized)
        .expect("typed HookList serialized form must reparse through serde_json::from_str");
    assert_eq!(
        from_string, list,
        "HookList to_string/from_str must round-trip",
    );
});
