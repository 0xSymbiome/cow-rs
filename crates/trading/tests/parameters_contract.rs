//! Builder-level same-token policy contract for `TradeParameters` and
//! `LimitTradeParameters`.
//!
//! The builder validators are chain-agnostic, so they pin exact same-token
//! `OrderKind` semantics here while the order-level validator owns the
//! chain-specific WETH/native-sentinel pair.

#![allow(
    clippy::doc_markdown,
    reason = "pedantic lint group acceptable inside integration test code"
)]

use cow_sdk_core::{Address, Amount, EVM_NATIVE_CURRENCY_ADDRESS, OrderKind};
use cow_sdk_trading::{ClientRejection, LimitTradeParameters, TradeParameters};

const SELL_TOKEN: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
// Canonical lowercase 0x-prefixed wire form (PROP-WB-004 / ADR 0052).
const WETH: &str = "0xc02aaa39b223fe8d0a0e5c4f27ead9083c756cc2";

fn address(hex: &str) -> Address {
    Address::new(hex).expect("fixture address must be valid")
}

fn amount(value: &str) -> Amount {
    Amount::new(value).expect("fixture amount must be valid")
}

fn trade_parameters(kind: OrderKind, sell_token: &str, buy_token: &str) -> TradeParameters {
    TradeParameters::new(
        kind,
        address(sell_token),
        address(buy_token),
        amount("1000000"),
    )
}

fn limit_trade_parameters(
    kind: OrderKind,
    sell_token: &str,
    buy_token: &str,
) -> LimitTradeParameters {
    LimitTradeParameters::new(
        kind,
        address(sell_token),
        address(buy_token),
        amount("1000000"),
        amount("2000000"),
    )
}

#[derive(Clone, Copy)]
enum Outcome {
    Accept,
    Reject,
}

#[test]
fn tradeparameters_validate_mirrors_services_allow_sell() {
    let cases = [
        (
            "same-token sell",
            SELL_TOKEN,
            SELL_TOKEN,
            OrderKind::Sell,
            Outcome::Accept,
        ),
        (
            "same-token buy",
            SELL_TOKEN,
            SELL_TOKEN,
            OrderKind::Buy,
            Outcome::Reject,
        ),
        (
            "WETH-native sell",
            WETH,
            EVM_NATIVE_CURRENCY_ADDRESS,
            OrderKind::Sell,
            Outcome::Accept,
        ),
        (
            "WETH-native buy",
            WETH,
            EVM_NATIVE_CURRENCY_ADDRESS,
            OrderKind::Buy,
            Outcome::Accept,
        ),
    ];

    for (label, sell_token, buy_token, kind, expected) in cases {
        let result = trade_parameters(kind, sell_token, buy_token).validate();
        match (expected, result) {
            (Outcome::Accept, Ok(()))
            | (Outcome::Reject, Err(ClientRejection::SameBuyAndSellToken { .. })) => {}
            (_, actual) => panic!("{label}: unexpected outcome: {actual:?}"),
        }
    }
}

#[test]
fn limittradeparameters_validate_mirrors_services_allow_sell() {
    let cases = [
        (
            "same-token sell",
            SELL_TOKEN,
            SELL_TOKEN,
            OrderKind::Sell,
            Outcome::Accept,
        ),
        (
            "same-token buy",
            SELL_TOKEN,
            SELL_TOKEN,
            OrderKind::Buy,
            Outcome::Reject,
        ),
        (
            "WETH-native sell",
            WETH,
            EVM_NATIVE_CURRENCY_ADDRESS,
            OrderKind::Sell,
            Outcome::Accept,
        ),
        (
            "WETH-native buy",
            WETH,
            EVM_NATIVE_CURRENCY_ADDRESS,
            OrderKind::Buy,
            Outcome::Accept,
        ),
    ];

    for (label, sell_token, buy_token, kind, expected) in cases {
        let result = limit_trade_parameters(kind, sell_token, buy_token).validate();
        match (expected, result) {
            (Outcome::Accept, Ok(()))
            | (Outcome::Reject, Err(ClientRejection::SameBuyAndSellToken { .. })) => {}
            (_, actual) => panic!("{label}: unexpected outcome: {actual:?}"),
        }
    }
}
