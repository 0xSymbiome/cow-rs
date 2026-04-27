//! Paginated order fetch against a local mock orderbook.
//!
//! This example shows the public `cow-sdk-orderbook` path for iterating
//! through an owner's order history without requiring live API credentials.
//! A `wiremock::MockServer` stands in for the real orderbook HTTP endpoint,
//! and `OrderBookApi` points at the mock through the typestate builder,
//! with a native `ReqwestTransport` dispatched against the mock URL via
//! the builder's `.base_url(...)` step. The example mirrors the behavior
//! a real consumer would see: `GetOrdersRequest` carries the owner plus a
//! mutable `offset` / `limit` pair, and each call returns the next page
//! of decoded `Order` values.
//!
//! Run with:
//!
//! ```text
//! cargo run -p cow-sdk-orderbook --example paginated_orders_fetch
//! ```
//!
//! Expected output:
//!
//! - one line per page showing the offset, limit, and returned order count
//! - a final summary with the total orders aggregated across pages

use cow_sdk_orderbook::{
    ApiContext, CowEnv, ExternalHostPolicy, GetOrdersRequest, OrderBookApi, OrderUid,
    SupportedChainId, types::Address,
};
use serde_json::json;
use wiremock::matchers::{method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

const OWNER: &str = "0xc8c753ee51e8fc80e199ab297fb575634a1ac1d3";
const PAGE_SIZE: u32 = 2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = MockServer::start().await;
    let owner = Address::new(OWNER)?;

    // Page 0: two orders.
    mount_page(
        &server,
        &owner,
        0,
        PAGE_SIZE,
        vec![
            order_fixture("0x1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111119999"),
            order_fixture("0x2222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222222229999"),
        ],
    )
    .await;

    // Page 1: one order.
    mount_page(
        &server,
        &owner,
        PAGE_SIZE,
        PAGE_SIZE,
        vec![order_fixture(
            "0x3333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333339999",
        )],
    )
    .await;

    // Page 2: empty, ending the iteration.
    mount_page(&server, &owner, PAGE_SIZE * 2, PAGE_SIZE, vec![]).await;

    let api = OrderBookApi::builder_from_context(ApiContext::new(
        SupportedChainId::GnosisChain,
        CowEnv::Prod,
    ))
    .with_external_host_policy(ExternalHostPolicy::Test)
    .base_url(server.uri())
    .build()?;

    let mut total_orders = 0_usize;
    let mut offset = 0_u32;

    loop {
        let mut request = GetOrdersRequest::new(owner.clone());
        request.offset = offset;
        request.limit = PAGE_SIZE;

        let page = api.get_orders(&request).await?;
        println!(
            "offset={offset:>3} limit={limit:>3} returned={count}",
            limit = PAGE_SIZE,
            count = page.len(),
        );

        if page.is_empty() {
            break;
        }

        total_orders += page.len();
        offset = offset.saturating_add(PAGE_SIZE);
    }

    println!("total_orders={total_orders}");

    Ok(())
}

async fn mount_page(
    server: &MockServer,
    owner: &Address,
    offset: u32,
    limit: u32,
    orders: Vec<serde_json::Value>,
) {
    Mock::given(method("GET"))
        .and(path(format!("/api/v1/account/{}/orders", owner.as_str())))
        .and(query_param("offset", offset.to_string()))
        .and(query_param("limit", limit.to_string()))
        .respond_with(ResponseTemplate::new(200).set_body_json(orders))
        .mount(server)
        .await;
}

fn order_fixture(uid: &str) -> serde_json::Value {
    // The fields below match the public `Order` DTO shape decoded by
    // `cow-sdk-orderbook` so the example exercises the full transform path.
    let uid = OrderUid::new(uid).expect("example uid literal must be valid");
    json!({
        "creationDate": "2025-01-21T12:55:14Z",
        "owner": OWNER,
        "uid": uid.as_str(),
        "availableBalance": "0",
        "executedBuyAmount": "0",
        "executedSellAmount": "0",
        "executedSellAmountBeforeFees": "0",
        "executedFeeAmount": "0",
        "invalidated": false,
        "status": "open",
        "class": "limit",
        "settlementContract": "0x9008d19f58aabd9ed0d60971565aa8510560ab41",
        "fullFeeAmount": "0",
        "isLiquidityOrder": false,
        "executedFee": "0",
        "executedFeeToken": "0x0000000000000000000000000000000000000000",
        "sellToken": "0xfff9976782d46cc05630d1f6ebab18b2324d6b14",
        "buyToken": "0x0625afb445c3b6b7b929342a04a22599fd5dbb59",
        "receiver": OWNER,
        "sellAmount": "1000000000000000000",
        "buyAmount": "1000000000000000000",
        "validTo": 1_700_000_000_u32,
        "appData": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "feeAmount": "0",
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20",
        "signingScheme": "eip712",
        "signature": "0x4d306ce7c770d22005bcfc00223f8d9aaa04e8a20099cc986cb9ccf60c7e876b777ceafb1e03f359ebc6d3dc84245d111a3df584212b5679cb5f9e6717b69b031b"
    })
}
