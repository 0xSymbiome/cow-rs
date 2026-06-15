//! Contract tests for the opt-in RPC retry seam (`retry`).
//!
//! The default provider issues each request once — a transient rate limit is
//! surfaced to the caller. Opting into a [`RetryConfig`] wraps the JSON-RPC
//! client in a bounded backoff layer that transparently retries a rate-limited
//! read.

use std::time::Duration;

use cow_sdk_alloy_provider::{ProviderErrorClass, RetryConfig, RpcAlloyProvider};
use cow_sdk_core::Provider;
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

#[tokio::test]
async fn default_provider_does_not_retry_a_rate_limited_read() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429))
        .mount(&server)
        .await;

    let provider = RpcAlloyProvider::builder()
        .http(server.uri())
        .unwrap()
        .build()
        .unwrap();

    let error = provider.get_chain_id().await.unwrap_err();

    assert_eq!(error.class(), ProviderErrorClass::Transport);
    // No retry layer is installed by default: exactly one request reaches the
    // endpoint and the rate limit propagates to the caller.
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn retry_recovers_from_a_transient_rate_limit() {
    let server = MockServer::start().await;
    // The first request is rate limited; the mock is exhausted after one match.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(429))
        .up_to_n_times(1)
        .with_priority(1)
        .mount(&server)
        .await;
    // Every subsequent request returns a valid `eth_chainId` result.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x1",
        })))
        .with_priority(2)
        .mount(&server)
        .await;

    let provider = RpcAlloyProvider::builder()
        .http(server.uri())
        .unwrap()
        .retry(RetryConfig::new(3, Duration::from_millis(10)))
        .build()
        .unwrap();

    assert_eq!(provider.get_chain_id().await.unwrap(), 1);
    // The transient 429 was retried transparently: two requests reached the
    // endpoint and the caller saw only the successful result.
    assert_eq!(server.received_requests().await.unwrap().len(), 2);
}
