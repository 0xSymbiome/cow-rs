//! PROTOTYPE / SIMULATION contract for the `fluent-preview` swap chain.
//!
//! Proves the fluent `swap().…quote().submit()` façade is runtime-equivalent to
//! the flat `post_swap_order` path it delegates to, and that the quote can be
//! inspected between `quote` and `submit`. Compiled only with the
//! `fluent-preview` feature.
#![cfg(feature = "fluent-preview")]

mod common;

use std::sync::Arc;

use cow_sdk_core::{OrderKind, SupportedChainId};
use cow_sdk_trading::{
    TradeParameters, Trading, TradingBuilder, TradingOptions, post_swap_order,
};

use crate::common::{
    MockOrderbook, MockSigner, sample_trade_parameters, sample_trader_parameters,
    sell_quote_response,
};

#[tokio::test]
async fn fluent_swap_chain_matches_flat_post_swap_order() {
    let trader = sample_trader_parameters();
    let signer = MockSigner::default();

    // Identical minimal inputs on both sides so the comparison is exact.
    let sample = sample_trade_parameters(OrderKind::Sell);
    let sell_token = sample.sell_token;
    let buy_token = sample.buy_token;
    let amount = sample.amount;

    // Flat reference path.
    let flat_orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let flat_trade = TradeParameters::new(OrderKind::Sell, sell_token, buy_token, amount);
    let flat = post_swap_order(&flat_trade, &trader, &signer, None, &flat_orderbook)
        .await
        .expect("flat post_swap_order should succeed");
    let flat_sent = flat_orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("flat order must be recorded");

    // Fluent chain path over the same inputs, against the SDK facade.
    let fluent_orderbook = Arc::new(MockOrderbook::new(trader.chain_id, sell_quote_response()));
    let trading = TradingBuilder::ready(
        trader.clone(),
        TradingOptions::new().with_orderbook_client(fluent_orderbook.clone()),
    )
    .expect("sdk construction should succeed");

    let quoted = trading
        .swap()
        .sell_token(sell_token)
        .buy_token(buy_token)
        .sell_amount(amount)
        .quote(&signer)
        .await
        .expect("fluent quote should succeed");

    // Inspect-before-submit is available and the quote is real.
    let inspected_quote_id = quoted.results().quote_response.id;
    assert!(
        inspected_quote_id.is_some(),
        "the fluent chain exposes the quote for inspection before submission"
    );

    let fluent = quoted
        .submit(&signer)
        .await
        .expect("fluent submit should succeed");
    let fluent_sent = fluent_orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("fluent order must be recorded");

    // The fluent chain submits the same order the flat path does.
    assert_eq!(fluent_sent.sell_amount, flat_sent.sell_amount);
    assert_eq!(fluent_sent.buy_amount, flat_sent.buy_amount);
    assert_eq!(fluent.signing_scheme, flat.signing_scheme);
    assert_eq!(fluent.order_to_sign.sell_token, flat.order_to_sign.sell_token);
    assert_eq!(fluent.order_to_sign.buy_token, flat.order_to_sign.buy_token);
}

/// Proves the ergonomic win: the whole chain runs from `Trading::builder()`
/// with `.orderbook(client)` — NO `Arc::new(...)` at the call site — and the
/// async `Signer` (MockSigner is async) drives `quote` and `submit`.
#[tokio::test]
async fn fluent_chain_builds_without_arc_and_drives_async_signer() {
    let signer = MockSigner::default();
    let sample = sample_trade_parameters(OrderKind::Sell);

    // `.orderbook(MockOrderbook::new(...))` — owned client, no `Arc::new` here.
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-fluent-preview")
        .orderbook(MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response()))
        .build()
        .expect("build should succeed without an Arc at the call site");

    let posted = trading
        .swap()
        .sell_token(sample.sell_token)
        .buy_token(sample.buy_token)
        .sell_amount(sample.amount)
        .quote(&signer) // async signer drives owner resolution
        .await
        .expect("quote should succeed")
        .submit(&signer) // async signer signs + posts
        .await
        .expect("submit should succeed");

    assert!(
        !posted.signature.is_empty(),
        "the chain completed end to end and produced a signature"
    );
}
