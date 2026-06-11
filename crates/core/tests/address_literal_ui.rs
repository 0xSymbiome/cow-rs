//! Live `trybuild` harness for the pinned compile-fail witnesses that
//! prove the [`address!`](cow_sdk_core::address) literal macro stays
//! strict: it takes exactly one string literal and rejects non-string
//! literals, empty invocations, and mixed-case literals whose EIP-55
//! checksum does not hold.
//!
//! The witness sources live at `tests/ui/address_literal_*.rs` and their
//! captured diagnostic snapshots live alongside as `.stderr` files. On
//! every `cargo test` run the `trybuild::TestCases` harness compiles each
//! witness through `rustc` and asserts the captured stderr matches the
//! live diagnostic, so a regression that loosens the literal contract
//! fails the test rather than passing a stale snapshot.
//!
//! The captured `.stderr` snapshots reflect the CI baseline toolchain, which
//! does not install the optional `rust-src` component. A local run with
//! `rust-src` present renders the const-panic's `core/src/panic.rs` frame as
//! an inline source snippet instead of the `= note: the failure occurred here`
//! line, so re-bless only without `rust-src` or the snapshot will diverge from
//! CI.

#[test]
fn address_literal_macro_rejects_malformed_invocations_at_compile_time() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/address_literal_non_string.rs");
    cases.compile_fail("tests/ui/address_literal_empty.rs");
    cases.compile_fail("tests/ui/address_literal_bad_checksum.rs");
}
