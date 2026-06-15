//! Typed orderbook transport against a mock HTTP server.
//!
//! Drives the concrete `OrderbookApi` over a wiremock server through its core
//! verbs — `version`, `quote`, `send_order`, and
//! `order_competition_status` — so the typed request and response wire
//! shapes are exercised against real HTTP rather than an in-memory double.

use std::error::Error;

use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use cow_sdk::core::{Amount, CowEnv, SupportedChainId};
use cow_sdk::orderbook::{
    ApiContext, ExternalHostPolicy, OrderCreation, OrderQuoteRequest, OrderQuoteSide, OrderbookApi,
    PriceQuality, SigningScheme as OrderbookSigningScheme,
};

use cow_sdk_examples_native::support::{
    COW, OWNER, WETH, orderbook_version_response, sample_app_data_hash, sample_order_uid,
    sample_quote_response_json, sample_signature,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Stand up a wiremock server and mount the four endpoints this example hits:
    // GET version, POST quote, POST orders, and GET order status.
    let server = MockServer::start().await;
    let order_uid = sample_order_uid();

    Mock::given(method("GET"))
        .and(path("/api/v1/version"))
        .respond_with(orderbook_version_response("v1.2.3"))
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/api/v1/quote"))
        .respond_with(ResponseTemplate::new(200).set_body_json(sample_quote_response_json()))
        .mount(&server)
        .await;

    let order_uid_hex = order_uid.to_hex_string();
    Mock::given(method("POST"))
        .and(path("/api/v1/orders"))
        .respond_with(ResponseTemplate::new(201).set_body_json(&order_uid_hex))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!("/api/v1/orders/{order_uid_hex}/status")))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": "open",
            "value": null
        })))
        .mount(&server)
        .await;

    // Build the OrderbookApi over the mock; the Test host policy allows localhost.
    let orderbook = OrderbookApi::builder_from_context(ApiContext::new(
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    ))
    .external_host_policy(ExternalHostPolicy::Test)
    .base_url(server.uri())
    .build()?;

    // 1. Protocol version.
    let version = orderbook.version().await?;

    // 2. Request a sell-side quote. Pinning the app-data hash binds it to the
    //    request: `OrderbookApi::quote` reconciles the response's echoed hash
    //    against the pin and fails closed on a mismatch, so the order built from
    //    the quote commits to the app-data the caller asked for.
    let quote_request = OrderQuoteRequest::new(
        WETH,
        COW,
        OWNER,
        OrderQuoteSide::sell(
            Amount::parse_units("0.1", 18).expect("example quote amount must remain valid"),
        ),
    )
    .with_app_data_hash(sample_app_data_hash())
    .with_price_quality(PriceQuality::Optimal);
    let quote = orderbook.quote(&quote_request).await?;

    // 3. Turn the quote into a signed order and submit it. The response's
    //    quote id rides along automatically, binding the submission to the
    //    quote the user approved.
    let order = OrderCreation::from_quote(
        &quote,
        OWNER,
        None,
        OrderbookSigningScheme::Eip712,
        sample_signature(),
    );
    let created_order_uid = orderbook.send_order(&order).await?;

    // 4. Read the order's competition status.
    let status = orderbook
        .order_competition_status(&created_order_uid)
        .await?;

    let report = json!({
        "surface": "cow_sdk::orderbook::OrderbookApi",
        "mode": "simulated-transport",
        "version": version,
        "quote": {
            "id": quote.id,
            "sellAmount": quote.quote.sell_amount,
            "buyAmount": quote.quote.buy_amount,
            "verified": quote.verified
        },
        "order": {
            "orderId": created_order_uid.to_hex_string(),
            "signingScheme": "eip712"
        },
        "status": {
            "type": status.kind
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
