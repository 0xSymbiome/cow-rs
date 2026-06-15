//! Order list / trade-history beat.
//!
//! Demonstrates `OrderbookApi::orders` and `OrderbookApi::trades` — the
//! list and history endpoints a consumer uses to show an account's open orders
//! and settled trades.
//!
//! These methods live on the concrete `OrderbookApi`: the `OrderbookClient`
//! trait and the high-level `Trading` facade expose only the single-order read
//! `order`, so code that needs order history reaches for `OrderbookApi`
//! directly, as shown here.
//!
//! Deterministic: the `OrderbookApi` is pointed at a wiremock server (the same
//! pattern as `orderbook_transport`); no live services are contacted.

#![allow(
    clippy::redundant_closure_for_method_calls,
    reason = "example clarity: the explicit `|value| value.to_hex_string()` closure reads better for a learner than a fully-qualified method reference"
)]

use std::error::Error;

use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use cow_sdk::core::{CowEnv, SupportedChainId};
use cow_sdk::orderbook::{ApiContext, ExternalHostPolicy, OrderbookApi, OrdersQuery, TradesQuery};

use cow_sdk_examples_native::support::{COW, ORDER_UID, OWNER, TX_HASH, WETH, sample_open_order};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;

    // GET /api/v1/account/{owner}/orders -> a one-element order list. The fixture
    // reuses `sample_open_order()` so the wire shape is the same normalized
    // `Order` the rest of the suite uses.
    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/account/{}/orders",
            OWNER.to_hex_string()
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([sample_open_order()])))
        .mount(&server)
        .await;

    // GET /api/v2/trades -> a one-element trade list for the same owner.
    Mock::given(method("GET"))
        .and(path("/api/v2/trades"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "blockNumber": 12_345_678u64,
            "logIndex": 3u64,
            "orderUid": ORDER_UID,
            "owner": OWNER,
            "sellToken": WETH,
            "buyToken": COW,
            "sellAmount": "1000000000000000000",
            "sellAmountBeforeFees": "1000000000000000000",
            "buyAmount": "500000000000000000",
            "txHash": TX_HASH,
        }])))
        .mount(&server)
        .await;

    let orderbook = OrderbookApi::builder_from_context(ApiContext::new(
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    ))
    .external_host_policy(ExternalHostPolicy::Test)
    .base_url(server.uri())
    .build()?;

    // LIST an account's orders (default pagination: offset 0, limit 1000).
    let orders = orderbook.orders(&OrdersQuery::new(OWNER)).await?;

    // HISTORY: trades for the same owner (owner XOR order-uid; default limit 10).
    let trades = orderbook.trades(&TradesQuery::by_owner(OWNER)).await?;

    let report = json!({
        "surface": "cow_sdk::orderbook::OrderbookApi::{orders, trades}",
        "mode": "simulated-transport",
        "note": "orders/trades live on OrderbookApi; the Trading facade forwards only order",
        "owner": OWNER,
        "orders": orders.iter().map(|o| json!({
            "uid": o.uid.to_hex_string(),
            "owner": o.owner.to_hex_string(),
            "status": o.status,
            "kind": o.kind,
            "executedSellAmount": o.executed_sell_amount,
            "executedBuyAmount": o.executed_buy_amount,
        })).collect::<Vec<_>>(),
        "orderCount": orders.len(),
        "trades": trades.iter().map(|t| json!({
            "orderUid": t.order_uid.to_hex_string(),
            "blockNumber": t.block_number,
            "sellAmount": t.sell_amount,
            "buyAmount": t.buy_amount,
            "txHash": t.tx_hash.as_ref().map(|h| h.to_hex_string()),
        })).collect::<Vec<_>>(),
        "tradeCount": trades.len(),
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
