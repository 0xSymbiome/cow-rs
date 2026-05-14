#![no_main]

//! Fuzz target for the typed schema-version validator.
//!
//! **Surface:** `cow_sdk_app_data::SchemaVersion::new`. The internal
//! `is_semver` and `is_non_empty_digits` helpers are crate-private so the
//! target exercises them through the public `SchemaVersion::new`
//! constructor, which is the sole production entry point and the only
//! surface the public API contract pins.
//! **Property:** `PROP-APP-002`.
//! **Seed contract:** corpus inputs cover canonical three-part decimal
//! semvers, boundary inputs (empty string, two-part `1.0`, four-part
//! `1.0.0.1`, leading-`v` prefix `v1.0.0`, empty middle segment `1..3`),
//! and adversarial inputs (alpha middle segment, leading whitespace, very
//! large digit strings, non-UTF-8 bytes).
//! **Corpus README:** `../corpus/fuzz_schema_version_is_semver/README.md`.
//!
//! The target invariants are:
//!
//! * `SchemaVersion::new` never panics for any input including non-UTF-8
//!   bytes coerced via `String::from_utf8_lossy`.
//! * `SchemaVersion::new(x).is_ok()` iff `x` matches the documented regex
//!   `^\d+\.\d+\.\d+$` (three non-empty ASCII-digit segments separated by
//!   `.` with nothing trailing).
//! * The constructor is deterministic on identical input.

use cow_sdk_app_data::SchemaVersion;
use libfuzzer_sys::fuzz_target;

/// Maximum input width accepted by the target. Schema versions in the wild
/// fit well under 32 bytes; capping at 4 KiB keeps the fuzzer bounded while
/// still letting it explore long-digit-segment inputs.
const MAX_FUZZ_INPUT: usize = 4096;

fuzz_target!(|data: &[u8]| {
    let data = &data[..data.len().min(MAX_FUZZ_INPUT)];
    let candidate = String::from_utf8_lossy(data).into_owned();

    let first = SchemaVersion::new(candidate.clone());
    let second = SchemaVersion::new(candidate.clone());

    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "SchemaVersion::new must be deterministic on identical input: {candidate:?}",
    );

    // Documented contract: Ok iff the input matches three non-empty ASCII
    // digit segments separated by '.' with nothing trailing.
    let expected_ok = is_three_part_decimal_semver(&candidate);
    assert_eq!(
        first.is_ok(),
        expected_ok,
        "SchemaVersion::new(x).is_ok() must agree with the documented regex contract for x = {candidate:?}",
    );

    if let Ok(version) = first {
        // Reference: the inner string must reproduce the original input
        // exactly so downstream serde transparently round-trips it.
        assert_eq!(
            version.as_str(),
            candidate,
            "SchemaVersion must preserve the original input string verbatim",
        );
    }
});

/// Reference implementation of the documented semver regex
/// `^\d+\.\d+\.\d+$`. Returns `true` exactly when the input consists of
/// three non-empty ASCII-digit segments separated by literal `.` with no
/// leading, trailing, or interior whitespace and no fourth segment.
fn is_three_part_decimal_semver(value: &str) -> bool {
    let mut parts = value.split('.');
    let Some(major) = parts.next() else {
        return false;
    };
    let Some(minor) = parts.next() else {
        return false;
    };
    let Some(patch) = parts.next() else {
        return false;
    };
    if parts.next().is_some() {
        return false;
    }
    is_non_empty_ascii_digits(major)
        && is_non_empty_ascii_digits(minor)
        && is_non_empty_ascii_digits(patch)
}

fn is_non_empty_ascii_digits(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
}
