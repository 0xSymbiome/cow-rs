//! Live `trybuild` harness for the pinned compile-fail witnesses that prove the
//! [`address!`](cow_sdk_core::address) macro stays strict: it takes exactly one
//! string literal and rejects non-string literals and empty invocations.
//!
//! Mixed-case rejection is covered separately by the runtime unit tests on
//! `is_lowercase_address_literal` (see `crates/core/src/lib.rs`). That contract
//! is intentionally not a `trybuild` witness: it fails through a const-evaluation
//! panic whose diagnostic rendering changes between rustc versions and with the
//! presence of the `rust-src` component, so a captured `.stderr` snapshot would
//! drift on every toolchain bump.

#[test]
fn address_literal_macro_rejects_malformed_invocations_at_compile_time() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/address_literal_non_string.rs");
    cases.compile_fail("tests/ui/address_literal_empty.rs");
}
