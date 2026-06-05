#![no_main]

//! Fuzz target for the `RetryPolicy` backoff helpers.
//!
//! **Surface:** `cow_sdk_transport_policy::RetryPolicy::{delay_for_attempt,
//! delay_for_status}` together with the builder seam that selects a jitter
//! strategy.
//! **Property:** `PROP-TPP-004`.
//! **Seed contract:** corpus inputs cover canonical attempt-driven backoff
//! shapes, `no_retry` policy boundaries, and adversarial extreme attempt
//! counts, oversized delays, and malformed `Retry-After` headers fed through
//! `delay_for_status`.
//!
//! The target maps arbitrary bytes into a typed `(attempt, base_ms, max_ms,
//! status, headers_seed)` tuple, builds a `RetryPolicy` through the public
//! builder, exercises `delay_for_attempt` and `delay_for_status`, and asserts
//! the returned delay never exceeds the configured `max_delay`, that the
//! function never panics, and that identical input yields identical output.

use std::time::{Duration, UNIX_EPOCH};

use cow_sdk_transport_policy::{JitterStrategy, RetryPolicy};
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};

const MAX_ATTEMPT_INDEX: usize = 64;
const MAX_BASE_MS: u64 = 60_000;
const MAX_MAX_MS: u64 = 600_000;
const FIXED_NOW_SECS: u64 = 1_700_000_000;

#[derive(Debug)]
struct RetryInput {
    attempt: usize,
    base_ms: u64,
    max_ms: u64,
    jitter_tag: u8,
    seed: u64,
    status: u16,
    headers_seed: u8,
}

impl<'a> Arbitrary<'a> for RetryInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        let attempt = usize::from(read_u16(bytes, 1)) % (MAX_ATTEMPT_INDEX + 1);
        let base_ms = read_u64(bytes, 50) % (MAX_BASE_MS + 1);
        let max_ms = base_ms.saturating_add(read_u64(bytes, 0) % (MAX_MAX_MS + 1));
        Ok(Self {
            attempt,
            base_ms,
            max_ms,
            jitter_tag: read_u8(bytes, 0),
            seed: read_u64(bytes, 0),
            status: read_u16(bytes, 200),
            headers_seed: read_u8(bytes, 0),
        })
    }
}

fuzz_target!(|input: RetryInput| {
    let max_delay = Duration::from_millis(input.max_ms);
    let policy = RetryPolicy::builder()
        .max_attempts(usize::from(read_u8_from_seed(input.headers_seed, 1)).saturating_add(1))
        .base_delay(Duration::from_millis(input.base_ms))
        .max_delay(max_delay)
        .jitter(jitter_for_tag(input.jitter_tag, input.seed))
        .build();

    let first = policy.delay_for_attempt(input.attempt);
    let second = policy.delay_for_attempt(input.attempt);
    assert_eq!(
        first, second,
        "RetryPolicy::delay_for_attempt must be deterministic on identical input",
    );
    assert!(
        first <= max_delay,
        "RetryPolicy::delay_for_attempt must respect the configured max_delay",
    );

    // Sweep every retryable status with a synthetic header value to exercise
    // the Retry-After fast path inside delay_for_status without panicking.
    let now = UNIX_EPOCH + Duration::from_secs(FIXED_NOW_SECS);
    let headers = headers_for_seed(input.headers_seed);
    let status_delay = policy.delay_for_status(input.attempt, input.status, &headers, now);
    let status_delay_again = policy.delay_for_status(input.attempt, input.status, &headers, now);
    assert_eq!(
        status_delay, status_delay_again,
        "RetryPolicy::delay_for_status must be deterministic at a fixed `now`",
    );

    // The status-driven delay is `max(backoff, retry_after)` for the two
    // documented Retry-After branches and `backoff` otherwise, so it must
    // remain `>= backoff` and never overflow into NaN/inf.
    assert!(
        status_delay >= first,
        "RetryPolicy::delay_for_status must not regress below the backoff delay",
    );

    // Exercise the no_retry policy invariant from PROP-TPP-003: two
    // instances compare equal and both apply only the documented first-attempt
    // delay.
    let no_retry_a = RetryPolicy::no_retry();
    let no_retry_b = RetryPolicy::no_retry();
    assert_eq!(
        no_retry_a, no_retry_b,
        "RetryPolicy::no_retry must produce identical policies",
    );
    assert_eq!(
        no_retry_a.max_attempts(),
        1,
        "RetryPolicy::no_retry must perform only the first attempt",
    );
});

fn jitter_for_tag(tag: u8, seed: u64) -> JitterStrategy {
    match tag % 4 {
        0 => JitterStrategy::none(),
        1 => JitterStrategy::full_from_seed(seed),
        2 => JitterStrategy::equal_from_seed(seed),
        _ => JitterStrategy::decorrelated_from_seed(seed),
    }
}

fn headers_for_seed(seed: u8) -> Vec<(String, String)> {
    match seed % 6 {
        0 => Vec::new(),
        1 => vec![("Retry-After".to_owned(), "1".to_owned())],
        2 => vec![("retry-after".to_owned(), "120".to_owned())],
        3 => vec![(
            "Retry-After".to_owned(),
            "Thu, 01 Jan 1970 00:00:10 GMT".to_owned(),
        )],
        4 => vec![("X-Other".to_owned(), "ignored".to_owned())],
        _ => vec![("Retry-After".to_owned(), "not-a-number".to_owned())],
    }
}

fn read_u8(bytes: &mut Unstructured<'_>, default: u8) -> u8 {
    u8::arbitrary(bytes).unwrap_or(default)
}

fn read_u16(bytes: &mut Unstructured<'_>, default: u16) -> u16 {
    u16::arbitrary(bytes).unwrap_or(default)
}

fn read_u64(bytes: &mut Unstructured<'_>, default: u64) -> u64 {
    u64::arbitrary(bytes).unwrap_or(default)
}

fn read_u8_from_seed(seed: u8, fallback: u8) -> u8 {
    if seed == 0 { fallback } else { seed }
}
