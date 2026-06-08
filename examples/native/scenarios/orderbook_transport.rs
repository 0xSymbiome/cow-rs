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
    orderbook_version_response, sample_buy_token, sample_order_uid, sample_owner,
    sample_quote_response_json, sample_sell_token, sample_signature,
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

    // 2. Request a sell-side quote.
    let quote_request = OrderQuoteRequest::new(
        sample_sell_token(),
        sample_buy_token(),
        sample_owner(),
        OrderQuoteSide::sell(
            Amount::parse_units("0.1", 18).expect("example quote amount must remain valid"),
        ),
    )
    .with_price_quality(PriceQuality::Optimal);
    let quote = orderbook.quote(&quote_request).await?;

    // 3. Turn the quote into a signed order and submit it.
    let order = OrderCreation::from_quote(
        &quote.quote,
        sample_owner(),
        None,
        OrderbookSigningScheme::Eip712,
        sample_signature(),
    )
    .with_quote_id(quote.id.expect("example quote id remains present"));
    let created_order_uid = orderbook.send_order(&order).await?;

    // 4. Read the order's competition status.
    let status = orderbook
        .order_competition_status(&created_order_uid)
        .await?;

    let report = json!({
        "surface": "cow-sdk::orderbook",
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
            "type": format!("{:?}", status.kind)
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
