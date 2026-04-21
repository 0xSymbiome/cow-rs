//! Pinned compile-fail witness for the split contract types.
//!
//! The reviewed services contract types `SellTokenSource` and
//! `BuyTokenDestination` are distinct enums in `cow_sdk_core`. The
//! witness below proves that a `SellTokenSource` value cannot be
//! assigned into a field typed as `BuyTokenDestination`: the buy-side
//! destination type admits only `Erc20` and `Internal`, and any
//! cross-side coercion is rejected at compile time so quote-derived and
//! direct trading-order construction cannot silently rewrite the
//! buy-side destination.
//!
//! Cargo's default integration-test discovery only picks up `tests/*.rs`,
//! so this source is never compiled by `cargo test --workspace`. The
//! sibling harness `crates/core/tests/order_balance_ui.rs` asserts the
//! source and its captured `stderr` remain pinned in the tree.

use cow_sdk_core::{
    Address, Amount, AppDataHex, BuyTokenDestination, OrderKind, SellTokenSource, UnsignedOrder,
};

fn main() {
    let sell_source: SellTokenSource = SellTokenSource::External;

    // Cross-side assignment of a `SellTokenSource` into a
    // `BuyTokenDestination`-typed field must fail to compile.
    let _order = UnsignedOrder {
        sell_token: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
        buy_token: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
        receiver: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        sell_amount: Amount::new("1").unwrap(),
        buy_amount: Amount::new("1").unwrap(),
        valid_to: 0,
        app_data: AppDataHex::new(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap(),
        fee_amount: Amount::zero(),
        kind: OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: SellTokenSource::Erc20,
        buy_token_balance: sell_source,
    };

    let _mismatch: BuyTokenDestination = sell_source;
}
