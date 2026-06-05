//! Cooperative cancellation of an in-flight request.
//!
//! Wraps a `Trading::get_quote_only` call with `Cancellable::cancel_with(&token)`
//! and fires the `CancellationToken` mid-flight, so the call resolves to
//! `TradingError::Cancelled` instead of completing. Uses a wiremock server with
//! a delayed response — aborting an in-flight request cannot be shown against an
//! instant in-memory double.

use std::{error::Error, sync::Arc, time::Duration};

use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path},
};

use cow_sdk::{
    core::{Cancellable, CancellationToken},
    orderbook::{ApiContext, ExternalHostPolicy},
    prelude::{CowEnv, OrderbookApi, SupportedChainId, TraderParameters, TradingError},
    trading::{TradingBuilder, TradingOptions},
};

use cow_sdk_examples_native::support::{sample_quote_response_json, sample_trade_parameters};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Quote endpoint that stalls for 30s, so the request is genuinely in-flight
    // when we cancel it.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/v1/quote"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(sample_quote_response_json())
                .set_delay(Duration::from_secs(30)),
        )
        .mount(&server)
        .await;

    let orderbook = OrderbookApi::builder_from_context(ApiContext::new(
        SupportedChainId::Sepolia,
        CowEnv::Prod,
    ))
    .with_external_host_policy(ExternalHostPolicy::Test)
    .base_url(server.uri())
    .build()?;

    let trading = TradingBuilder::ready(
        TraderParameters::new(SupportedChainId::Sepolia, "cow-rs-cancellation-example")
            .expect("app code should validate"),
        TradingOptions::new().with_orderbook_client(Arc::new(orderbook)),
    )?;

    // One token, cloned: clones share the same cancellation state. A background
    // task fires it after 100ms, long before the 30s response would arrive.
    let token = CancellationToken::new();
    let token_for_quote = token.clone();
    let token_for_timer = token.clone();
    let cancel_after_delay = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(100)).await;
        token_for_timer.cancel();
    });

    // `cancel_with` races the call against the token; here the token wins.
    let quote_result = trading
        .get_quote_only(sample_trade_parameters(), None)
        .cancel_with(&token_for_quote)
        .await;
    cancel_after_delay.await?;

    // The aborted call surfaces as a typed `TradingError::Cancelled`.
    let error = quote_result.expect_err("the delayed quote must be cancelled");
    assert!(
        matches!(error, TradingError::Cancelled),
        "expected TradingError::Cancelled, got {error:?}",
    );

    let report = json!({
        "surface": "cow-sdk::Trading::get_quote_only",
        "mode": "simulated-transport",
        "cancellation": {
            "delayMs": 100,
            "error": "TradingError::Cancelled"
        }
    });
    println!("{}", serde_json::to_string_pretty(&report)?);

    Ok(())
}
