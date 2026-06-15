#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::{AlloyClient, AlloyClientBuilderError, AlloyClientError};
use cow_sdk_core::SupportedChainId;
use serde_json::json;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{body_partial_json, method, path},
};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

#[tokio::test]
async fn build_checked_rejects_remote_chain_mismatch() {
    let server = chain_id_server(11_155_111).await;

    let result = AlloyClient::builder()
        .http(server.uri())
        .expect("transport")
        .private_key(TEST_KEY)
        .expect("key")
        .chain_id(SupportedChainId::Mainnet)
        .build_checked()
        .await;

    match result {
        Err(AlloyClientBuilderError::ChainMismatch { configured, remote }) => {
            assert_eq!(configured, 1);
            assert_eq!(remote, 11_155_111);
        }
        other => panic!("expected ChainMismatch, got {other:?}"),
    }
}

#[tokio::test]
async fn build_does_not_dispatch_any_http_request() {
    let server = MockServer::start().await;

    let _client = AlloyClient::builder()
        .http(server.uri())
        .expect("transport")
        .private_key(TEST_KEY)
        .expect("key")
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
        .expect("default build path is free of network I/O");

    let received = server.received_requests().await.expect("requests captured");
    assert!(
        received.is_empty(),
        "default build() must not dispatch any HTTP request, got {received:?}"
    );
}

#[tokio::test]
async fn verify_chain_id_returns_validation_error_on_mismatch() {
    let server = chain_id_server(11_155_111).await;
    let client = AlloyClient::builder()
        .http(server.uri())
        .expect("transport")
        .private_key(TEST_KEY)
        .expect("key")
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
        .expect("default build is free of network I/O");

    let error = client
        .verify_chain_id()
        .await
        .expect_err("mismatch must surface as validation error");

    assert!(
        matches!(error, AlloyClientError::Validation(_)),
        "expected AlloyClientError::Validation, got {error:?}"
    );
}

async fn chain_id_server(chain_id: u64) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/"))
        .and(body_partial_json(json!({"method": "eth_chainId"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": format!("0x{chain_id:x}"),
        })))
        .mount(&server)
        .await;
    server
}
