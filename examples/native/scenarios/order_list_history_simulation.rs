//! Order list / trade-history beat.
//!
//! Demonstrates `OrderbookApi::get_orders` and `OrderbookApi::get_trades` — the
//! list and history endpoints a consumer uses to show an account's open orders
//! and settled trades.
//!
//! These methods live on the concrete `OrderbookApi`: the `OrderbookClient`
//! trait and the high-level `Trading` facade expose only the single-order read
//! `get_order`, so code that needs order history reaches for `OrderbookApi`
//! directly, as shown here.
//!
//! Deterministic: the `OrderbookApi` is pointed at a wiremock server (the same
//! pattern as `orderbook_transport_roundtrip`); no live services are contacted.

use std::error::Error;

use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use cow_sdk::orderbook::{
    ApiContext, ExternalHostPolicy, GetOrdersRequest, GetTradesRequest, OrderbookApi,
};
use cow_sdk::prelude::{CowEnv, SupportedChainId};

use cow_sdk_examples_native::support::{ORDER_UID, OWNER, TX_HASH, sample_open_order, sample_owner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let server = MockServer::start().await;
    let owner = sample_owner();
    let owner_hex = owner.to_hex_string();

    // GET /api/v1/account/{owner}/orders -> a one-element order list. The fixture
    // reuses `sample_open_order()` so the wire shape is the same normalized
    // `Order` the rest of the suite uses.
    let order_json = serde_json::to_value(sample_open_order())?;
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/account/{owner_hex}/orders")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([order_json])))
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
            "sellToken": "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14",
            "buyToken": "0x0625aFB445C3B6B7B929342a04A22599fd5dBB59",
            "sellAmount": "1000000000000000000",
            "sellAmountBeforeFees": "1000000000000000000",
            "buyAmount": "500000000000000000",
            "txHash": TX_HASH,
        }])))
        .mount(&server)
        .await;

    let api = OrderbookApi::builder_from_context(ApiContext::new(
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    ))
    .with_external_host_policy(ExternalHostPolicy::Test)
    .base_url(server.uri())
    .build()?;

    // LIST an account's orders (default pagination: offset 0, limit 1000).
    let orders = api.get_orders(&GetOrdersRequest::new(owner)).await?;

    // HISTORY: trades for the same owner (owner XOR order-uid; default limit 10).
    let trades = api.get_trades(&GetTradesRequest::by_owner(owner)).await?;

    let report = json!({
        "surface": "cow-sdk::orderbook list/history (OrderbookApi)",
        "mode": "simulated-transport",
        "note": "get_orders/get_trades live on OrderbookApi; the Trading facade forwards only get_order",
        "owner": owner_hex,
        "orders": orders.iter().map(|o| json!({
            "uid": o.uid.to_hex_string(),
            "owner": o.owner.to_hex_string(),
            "status": format!("{:?}", o.status),
            "kind": format!("{:?}", o.kind),
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
