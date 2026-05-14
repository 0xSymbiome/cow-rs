#![no_main]

//! Fuzz target for the canonical-JSON renderer.
//!
//! **Surface:** `cow_sdk_app_data::stringify_deterministic`.
//! **Property:** `PROP-APP-003`.
//! **Seed contract:** corpus inputs cover canonical app-data document
//! shapes (objects, nested objects, arrays mixed with scalars), boundary
//! values (null, empty object, empty array, single-character strings,
//! large integers), and adversarial inputs (Unicode escapes, deeply
//! nested arrays, non-JSON bytes that exercise the early-return path).
//! **Corpus README:** `../corpus/fuzz_stringify_deterministic/README.md`.
//!
//! The target invariants are:
//!
//! * `stringify_deterministic` never panics for any well-formed
//!   `serde_json::Value` derived from raw fuzz bytes.
//! * For every `Ok(s)`, `serde_json::from_str::<Value>(&s)` re-parses
//!   successfully (the output is always well-formed JSON).
//! * The renderer is deterministic: invoking it twice on the same value
//!   produces the same string.
//!
//! We do not assert byte-level idempotence over the parse+render cycle
//! (`stringify(parse(stringify(v))) == stringify(v)`) for arbitrary
//! values: `serde_json::Value::Number` falls back to `f64` for any
//! non-integer-representable input, and `f64`'s `Display` chooses the
//! shortest representation among all `f64` bit patterns. A literal like
//! `3e+23` can render through one shortest-representation path and the
//! reparsed value can render through a slightly different path — a
//! universal IEEE-754 f64 precision limitation rather than an SDK
//! contract violation. The shipped canonical-form stability invariant is
//! covered by the parity fixture and unit tests in
//! `crates/app-data/tests/property_contract.rs`.

use cow_sdk_app_data::stringify_deterministic;
use libfuzzer_sys::fuzz_target;
use serde_json::Value;

const MAX_FUZZ_INPUT: usize = 4096;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];

    let Ok(value) = serde_json::from_slice::<Value>(data) else {
        return;
    };

    let first = stringify_deterministic(&value);
    let second = stringify_deterministic(&value);
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "stringify_deterministic must be deterministic on identical input",
    );

    let Ok(rendered) = first else {
        return;
    };
    let second_rendered = second.expect("determinism check above already proved Ok");
    assert_eq!(
        rendered, second_rendered,
        "stringify_deterministic must produce byte-identical output on identical input",
    );

    // Output is always well-formed JSON. The renderer must succeed on the
    // round-tripped value as well; we do not assert byte-level idempotence
    // because `f64` shortest-representation rendering can vary slightly
    // across parse+render cycles for numeric values outside the f64
    // safe-integer range. See the target-level rustdoc for the rationale.
    let reparsed: Value = serde_json::from_str(&rendered)
        .expect("stringify_deterministic output must reparse as valid JSON");
    let _ = stringify_deterministic(&reparsed)
        .expect("re-rendering a reparsed canonical value must succeed");
});
