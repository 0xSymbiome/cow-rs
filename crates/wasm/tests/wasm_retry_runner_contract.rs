//! Browser-target regression tests for the shared retry driver and its wall
//! clock, exercised on the wasm32 target that `wasm.yml` runs.
//!
//! The standard `std::time::SystemTime::now()` aborts on
//! `wasm32-unknown-unknown`, so a retry-delay computation that read it would
//! abort the wasm instance on the first retryable response. The orderbook,
//! subgraph, and IPFS clients all route their retries through
//! [`cow_sdk_transport_policy::run_with_retry`], which reads the browser wall
//! clock through [`cow_sdk_transport_policy::system_now`]. These tests lock the
//! no-panic behavior of that path on the wasm target.

#![cfg(target_arch = "wasm32")]

use std::time::Duration;

use cow_sdk_transport_policy::{
    AttemptOutcome, JitterStrategy, LimiterKey, RequestRateLimiter, RetryPolicy, RetrySignal,
    run_with_retry, system_now,
};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn system_now_returns_a_wall_clock_value_without_panicking() {
    let now = system_now();
    assert!(
        now >= std::time::SystemTime::UNIX_EPOCH,
        "the wasm wall clock must resolve at or after the Unix epoch"
    );
}

#[wasm_bindgen_test]
async fn retryable_status_drives_backoff_without_panicking() {
    // A retryable status enters the backoff path, which reads the wall clock to
    // evaluate `Retry-After`. Zero delays keep the test fast; the point is that
    // the clock read no longer aborts the wasm instance and the retry succeeds.
    let policy = RetryPolicy::builder()
        .max_attempts(2)
        .base_delay(Duration::ZERO)
        .max_delay(Duration::ZERO)
        .jitter(JitterStrategy::none())
        .build();
    let limiter = RequestRateLimiter::unlimited();
    let mut attempts = 0u32;

    let result: Result<(), ()> =
        run_with_retry(&policy, &limiter, LimiterKey::Global, |_attempt_index| {
            attempts += 1;
            let first = attempts == 1;
            async move {
                if first {
                    AttemptOutcome::Failure {
                        error: (),
                        signal: RetrySignal::HttpStatus {
                            status: 503,
                            headers: Vec::new(),
                        },
                    }
                } else {
                    AttemptOutcome::Success(())
                }
            }
        })
        .await;

    assert!(result.is_ok(), "the second attempt should succeed");
    assert_eq!(attempts, 2, "exactly one retry should have occurred");
}
