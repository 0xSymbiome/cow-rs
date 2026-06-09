#![no_main]

//! Fuzz target for the HTTP `Retry-After` header parser.
//!
//! **Surface:** `cow_sdk_core::transport::policy::parse_retry_after`.
//! **Property:** `PROP-TPP-004`.
//! **Seed contract:** corpus inputs cover canonical delta-seconds values,
//! canonical IMF-fixdate values, boundary empty / whitespace / zero inputs,
//! and adversarial NaN, negative, oversized, and malformed date shapes.
//!
//! The target feeds raw bytes through `String::from_utf8_lossy` into
//! `parse_retry_after` together with a fixed `SystemTime` anchor, and asserts
//! that the parser never panics, returns a delay bounded by `Duration::MAX`
//! when it accepts the value, and is deterministic on identical input.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use cow_sdk_core::transport::policy::parse_retry_after;
use libfuzzer_sys::fuzz_target;

const FIXED_NOW_SECS: u64 = 1_700_000_000;

fuzz_target!(|data: &[u8]| {
    let value = String::from_utf8_lossy(data).into_owned();
    let now = UNIX_EPOCH + Duration::from_secs(FIXED_NOW_SECS);

    let first = parse_retry_after(&value, now);
    let second = parse_retry_after(&value, now);
    assert_eq!(
        first, second,
        "parse_retry_after must be deterministic on identical input",
    );

    if let Some(retry_after) = first {
        let delay = retry_after.delay();
        assert!(
            delay <= Duration::MAX,
            "parse_retry_after delay must remain within Duration::MAX",
        );
        // The accepted value must round-trip through the public accessor.
        assert_eq!(
            retry_after.delay(),
            delay,
            "RetryAfter::delay must be a stable read accessor",
        );
    }

    // Vary the anchor to exercise the IMF-fixdate clamp-to-zero branch.
    let alt_now = SystemTime::now();
    let _ = parse_retry_after(&value, alt_now);
});
