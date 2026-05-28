//! Live `trybuild` harness for the pinned compile-fail witness that
//! proves the typed amount surface exposes no bare arithmetic operators
//! (`+` `-` `*` and the `*Assign` variants) and no `pow`.
//!
//! The witness source lives at
//! `tests/ui/amount_arithmetic_operators_removed.rs` and its captured
//! diagnostic snapshot lives alongside as
//! `tests/ui/amount_arithmetic_operators_removed.stderr`. On every
//! `cargo test` run the `trybuild::TestCases` harness compiles the
//! witness through `rustc` and asserts the captured stderr matches the
//! live diagnostic, so a regression that re-introduces a wrapping (or
//! debug-only panicking) operator on the typed amount surface fails the
//! test rather than passing a stale snapshot.

#[test]
fn typed_amount_arithmetic_surface_rejects_bare_operators_at_compile_time() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/amount_arithmetic_operators_removed.rs");
}
