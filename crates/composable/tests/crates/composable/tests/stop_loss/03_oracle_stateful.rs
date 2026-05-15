//! Reserved test file for the StopLoss conditional-order handler slot
//! `03_oracle_stateful`.
//!
//! This integration test exercises Chainlink oracle round probe with timestamp staleness.
//! The crate body lands in a later capability landing; the test stays gated
//! behind the `implementation` feature until then so the placeholder
//! integration target compiles cleanly without the not-yet-present helpers.

#![cfg(feature = "implementation")]

#[test]
fn stop_loss_oracle_stateful_reserved_until_helpers_ship() {
    assert!(true);
}
