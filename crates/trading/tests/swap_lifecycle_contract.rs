//! Contract for the fluent swap lifecycle (`Trading::swap`).
//!
//! Proves the typed builder is an additive façade over the flat surface: it
//! posts the same order as `post_swap_order`, the quote can be inspected between
//! `quote` and `submit`, the builder injects an orderbook client without an
//! `Arc` at the call site, and the one-shot `execute` path is transposition-safe.

mod common;

use cow_sdk_core::{OrderKind, SupportedChainId};
use cow_sdk_trading::{TradeParameters, Trading, TradingBuilder, TradingOptions, post_swap_order};

use crate::common::{
    MockOrderbook, MockSigner, sample_trade_parameters, sample_trader_parameters,
    sell_quote_response,
};

#[tokio::test]
async fn swap_builder_quote_then_submit_matches_flat_post_swap_order() {
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

    // Fluent builder path against the SDK facade.
    let fluent_orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let trading = TradingBuilder::ready(
        trader.clone(),
        TradingOptions::new().with_orderbook(fluent_orderbook.clone()),
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
    assert!(
        quoted.results().quote_response.id.is_some(),
        "the quote is exposed for inspection before submission"
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

    // The fluent builder submits the same order the flat path does.
    assert_eq!(fluent_sent.sell_amount, flat_sent.sell_amount);
    assert_eq!(fluent_sent.buy_amount, flat_sent.buy_amount);
    assert_eq!(fluent.signing_scheme, flat.signing_scheme);
    assert_eq!(fluent.order_to_sign.sell_token, flat.order_to_sign.sell_token);
    assert_eq!(fluent.order_to_sign.buy_token, flat.order_to_sign.buy_token);
}

#[tokio::test]
async fn swap_builder_injects_orderbook_without_arc_and_drives_async_signer() {
    let signer = MockSigner::default();
    let sample = sample_trade_parameters(OrderKind::Sell);

    // `.orderbook(client)` takes the client by value — no `Arc::new` at the call site.
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-swap-lifecycle")
        .orderbook(MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response()))
        .build()
        .expect("build should succeed without an Arc at the call site");

    let posted = trading
        .swap()
        .sell_token(sample.sell_token)
        .buy_token(sample.buy_token)
        .sell_amount(sample.amount)
        .quote(&signer) // the async signer resolves the owner
        .await
        .expect("quote should succeed")
        .submit(&signer) // the async signer signs and posts
        .await
        .expect("submit should succeed");

    assert!(
        !posted.signature.is_empty(),
        "the lifecycle completed end to end and produced a signature"
    );
}

#[tokio::test]
async fn swap_builder_execute_is_one_call_and_transposition_safe() {
    let signer = MockSigner::default();
    let sample = sample_trade_parameters(OrderKind::Sell);

    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-swap-lifecycle")
        .orderbook(MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response()))
        .build()
        .expect("build should succeed");

    let posted = trading
        .swap()
        .sell_token(sample.sell_token) // named — cannot be swapped with the buy token
        .buy_token(sample.buy_token)
        .sell_amount(sample.amount)
        .execute(&signer)
        .await
        .expect("one-call execute should succeed");

    assert!(!posted.signature.is_empty());
}
