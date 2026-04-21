//! Live `trybuild` harness for the pinned compile-fail witness that
//! proves the split between [`SellTokenSource`](cow_sdk_core::SellTokenSource)
//! and [`BuyTokenDestination`](cow_sdk_core::BuyTokenDestination)
//! rejects cross-side coercion at the type system.
//!
//! The witness source lives at
//! `tests/ui/token_balance_split_cross_side.rs` and its captured
//! diagnostic snapshot lives alongside as
//! `tests/ui/token_balance_split_cross_side.stderr`. On every
//! `cargo test` run the `trybuild::TestCases` harness compiles the
//! witness through `rustc` and asserts the captured stderr matches
//! the live diagnostic, so a regression that silently re-collapses
//! the two enums fails the test rather than passing a stale
//! snapshot.

#[test]
fn token_balance_split_rejects_cross_side_coercion_at_compile_time() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/token_balance_split_cross_side.rs");
}
