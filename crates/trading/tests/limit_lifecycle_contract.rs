//! Contract for the fluent limit-order lifecycle (`Trading::limit`).
//!
//! Proves the typed builder is an additive façade over the flat surface: it posts
//! the same order as `post_limit_order`, the named sell/buy token and amount setters
//! cannot be transposed, and the signer-less `post_presign` path drives the
//! smart-account placement through the builder.

mod common;

use cow_sdk_core::{OrderKind, SupportedChainId};
use cow_sdk_trading::{LimitTradeParams, Trading, post_limit_order};

use crate::common::{
    MockOrderbook, MockSigner, sample_limit_parameters, sample_trader_parameters,
    sell_quote_response,
};

#[tokio::test]
async fn limit_builder_post_matches_flat_post_limit_order() {
    let trader = sample_trader_parameters();
    let signer = MockSigner::default();

    // Identical minimal inputs on both sides so the comparison is exact. The owner is
    // left to the signer on both paths, so neither trips the owner-recovery gate.
    let sample = sample_limit_parameters(OrderKind::Sell);
    let sell_token = sample.sell_token;
    let buy_token = sample.buy_token;
    let sell_amount = sample.sell_amount;
    let buy_amount = sample.buy_amount;

    // Flat reference path.
    let flat_orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let flat_params = LimitTradeParams::new(
        OrderKind::Sell,
        sell_token,
        buy_token,
        sell_amount,
        buy_amount,
    );
    let flat = post_limit_order(&flat_params, &trader, &signer, None, &flat_orderbook)
        .await
        .expect("flat post_limit_order should succeed");
    let flat_sent = flat_orderbook
        .state()
        .sent_orders
        .last()
        .cloned()
        .expect("flat order must be recorded");

    // Fluent builder path against the SDK facade.
    let fluent_orderbook = MockOrderbook::new(trader.chain_id, sell_quote_response());
    let trading = Trading::builder()
        .chain_id(trader.chain_id)
        .app_code(trader.app_code.clone())
        .orderbook(fluent_orderbook.clone())
        .build()
        .expect("sdk construction should succeed");

    let fluent = trading
        .limit()
        .sell_token(sell_token) // named — cannot be swapped with the buy token
        .buy_token(buy_token)
        .sell_amount(sell_amount) // named — cannot be swapped with the buy amount
        .buy_amount(buy_amount)
        .post(&signer)
        .await
        .expect("fluent limit post should succeed");
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
    assert_eq!(
        fluent.order_to_sign.sell_token,
        flat.order_to_sign.sell_token
    );
    assert_eq!(fluent.order_to_sign.buy_token, flat.order_to_sign.buy_token);
}

#[tokio::test]
async fn limit_builder_post_presign_posts_without_a_signer_through_the_builder() {
    // Pre-sign placement needs an explicit owner because no signer participates.
    let sample = sample_limit_parameters(OrderKind::Sell);
    let owner = sample
        .owner
        .expect("the limit sample carries an explicit owner");

    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response());
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Sepolia)
        .app_code("cow-rs-limit-lifecycle")
        .orderbook(orderbook.clone())
        .build()
        .expect("build should succeed");

    let posted = trading
        .limit()
        .sell_token(sample.sell_token)
        .buy_token(sample.buy_token)
        .sell_amount(sample.sell_amount)
        .buy_amount(sample.buy_amount)
        .owner(owner)
        .post_presign()
        .await
        .expect("builder pre-sign post should succeed without a signer");

    assert!(
        !orderbook.state().sent_orders.is_empty(),
        "the pre-sign order was posted through the builder"
    );
    let _ = posted;
}
