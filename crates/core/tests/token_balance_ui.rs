//! Harness guarding the pinned compile-fail witness for the split
//! between `SellTokenSource` and `BuyTokenDestination`.
//!
//! The witness source lives at
//! `tests/ui/token_balance_split_cross_side.rs` and its captured
//! diagnostic snapshot lives alongside as
//! `tests/ui/token_balance_split_cross_side.stderr`. Cargo's default
//! discovery only picks up `tests/*.rs`, so the witness source under
//! `tests/ui/` is NOT compiled by `cargo test --workspace`. This harness
//! runs on every platform and asserts both witness artifacts remain in
//! the tree with the expected invariants so drift is caught at test
//! time rather than in reviewer diffs.
//!
//! The compile failure the witness records is: assigning a
//! `SellTokenSource` value into a `BuyTokenDestination`-typed field on
//! `UnsignedOrder` is a type mismatch. The reviewed services contract
//! types model the sell-side allowance path (`Erc20`, `External`,
//! `Internal`) and the buy-side payout path (`Erc20`, `Internal`) as
//! distinct enums, so no silent cross-side coercion survives at the
//! public boundary.

use std::path::Path;

const WITNESS_SOURCE: &str = "tests/ui/token_balance_split_cross_side.rs";
const WITNESS_STDERR: &str = "tests/ui/token_balance_split_cross_side.stderr";

#[test]
fn token_balance_split_witness_source_is_present() {
    assert!(
        Path::new(WITNESS_SOURCE).exists(),
        "token-balance split compile-fail witness source must remain \
         pinned at {WITNESS_SOURCE}",
    );
    let body = std::fs::read_to_string(WITNESS_SOURCE)
        .expect("reading the pinned witness source must succeed");
    assert!(
        body.contains("SellTokenSource") && body.contains("BuyTokenDestination"),
        "witness source must reference both split enums so the \
         intended cross-side rejection remains exercised",
    );
    assert!(
        body.contains("buy_token_balance: sell_source"),
        "witness source must still attempt to assign a \
         `SellTokenSource` value into the buy-side field on \
         `UnsignedOrder` so the intended compile error is preserved",
    );
    assert!(
        body.contains("let _mismatch: BuyTokenDestination = sell_source"),
        "witness source must still assert that a `SellTokenSource` is \
         not assignable to a `BuyTokenDestination`-typed binding",
    );
}

#[test]
fn token_balance_split_witness_stderr_captures_the_expected_error() {
    assert!(
        Path::new(WITNESS_STDERR).exists(),
        "token-balance split compile-fail stderr snapshot must remain \
         pinned at {WITNESS_STDERR}",
    );
    let stderr = std::fs::read_to_string(WITNESS_STDERR)
        .expect("reading the pinned witness stderr must succeed");
    assert!(
        stderr.contains("mismatched types"),
        "pinned stderr must record the `mismatched types` diagnostic \
         so reviewers can verify the intended shape",
    );
    assert!(
        stderr.contains("expected `BuyTokenDestination`")
            && stderr.contains("found `SellTokenSource`"),
        "pinned stderr must name both expected and found types so the \
         compile error is scoped to the cross-side coercion the split \
         is designed to reject",
    );
}
