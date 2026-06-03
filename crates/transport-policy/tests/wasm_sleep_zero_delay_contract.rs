//! Source-level never-swap guard for the wasm retry sleep.
//!
//! The wasm32 `sleep` path must await the clamped `gloo` timeout for every
//! duration — including `Duration::ZERO` — so a zero-delay retry still yields
//! to the executor (cooperative cancellation and fairness depend on it). This
//! guard pins the implementation against the two regressions that would break
//! that contract: short-circuiting `Duration::ZERO` before yielding, and
//! swapping in an unpinned `yield_now` helper.
//!
//! It lives in `transport-policy` (which owns `src/time.rs`) rather than a
//! consuming crate, so a refactor of the sleep path fails the owning crate's
//! own suite instead of a surprising cross-crate test.

const TIME_SOURCE: &str = include_str!("../src/time.rs");

#[test]
fn wasm_zero_delay_path_yields_through_gloo_timeout_zero() {
    assert!(
        TIME_SOURCE.contains("TimeoutFuture::new(millis).await;"),
        "wasm32 retry sleep must await the clamped gloo timeout value"
    );
    assert!(
        TIME_SOURCE.contains("duration.as_millis().min(u128::from(u32::MAX))"),
        "wasm32 retry sleep must derive the gloo timeout from the requested duration"
    );
    assert!(
        !TIME_SOURCE.contains("if millis > 0"),
        "wasm32 retry sleep must not short-circuit Duration::ZERO before yielding"
    );
    assert!(
        !TIME_SOURCE.contains("wasm_bindgen_futures::yield_now"),
        "wasm32 retry sleep must not depend on an unpinned yield_now helper"
    );
}
