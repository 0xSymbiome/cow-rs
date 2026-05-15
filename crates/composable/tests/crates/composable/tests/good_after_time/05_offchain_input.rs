//! Reserved test file for the GoodAfterTime conditional-order handler slot
//! `05_offchain_input`.
//!
//! This integration test exercises offchainInput payload decoding from the conditional-order parameters.
//! The crate body lands in a later capability landing; the test stays gated
//! behind the `implementation` feature until then so the placeholder
//! integration target compiles cleanly without the not-yet-present helpers.

#![cfg(feature = "implementation")]

#[test]
fn good_after_time_offchain_input_reserved_until_helpers_ship() {
    assert!(true);
}
