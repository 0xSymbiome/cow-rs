use std::error::Error;

use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use cow_sdk::orderbook::{
    ApiContext, ExternalHostPolicy, OrderCreation, OrderQuoteRequest, PriceQuality, QuoteSide,
    SigningScheme as OrderbookSigningScheme,
};
use cow_sdk::prelude::{Amount, CowEnv, OrderBookApi, SupportedChainId};

use cow_sdk_examples_native::support::{
    orderbook_version_response, sample_buy_token, sample_order_uid, sample_owner,
    sample_quote_response_json, sample_sell_token, sample_signature,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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

    Mock::given(method("POST"))
        .and(path("/api/v1/orders"))
        .respond_with(ResponseTemplate::new(201).set_body_json(order_uid.as_str()))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path(format!(
            "/api/v1/orders/{}/status",
            order_uid.as_str()
        )))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "type": "open",
            "value": null
        })))
        .mount(&server)
        .await;

    let api = OrderBookApi::builder_from_context(ApiContext::new(
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    ))
    .with_external_host_policy(ExternalHostPolicy::Test)
    .base_url(server.uri())
    .build()?;

    let version = api.get_version().await?;
    let quote_request = OrderQuoteRequest::new(
        sample_sell_token(),
        sample_buy_token(),
        sample_owner(),
        QuoteSide::sell(
            Amount::new("100000000000000000").expect("example quote amount must remain valid"),
        ),
    )
    .with_price_quality(PriceQuality::Optimal);
    let quote = api.get_quote(&quote_request).await?;
    let order = OrderCreation::from_quote(
        &quote.quote,
        sample_owner(),
        None,
        OrderbookSigningScheme::Eip712,
        sample_signature(),
    )
    .with_quote_id(quote.id.expect("example quote id remains present"));
    let created_order_uid = api.send_order(&order).await?;
    let status = api.get_order_competition_status(&created_order_uid).await?;

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
            "orderId": created_order_uid.as_str(),
            "signingScheme": "eip712"
        },
        "status": {
            "type": format!("{:?}", status.kind)
        }
    });

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
