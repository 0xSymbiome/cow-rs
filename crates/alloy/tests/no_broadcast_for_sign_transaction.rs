#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::{AlloyClient, AlloyClientError};
use cow_sdk_core::{Signer, SigningProvider, SupportedChainId, TransactionRequest};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

#[tokio::test]
async fn sign_transaction_returns_unsupported_without_http_request() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(500))
        .expect(0)
        .mount(&server)
        .await;
    let client = AlloyClient::builder()
        .http(server.uri())
        .unwrap()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
        .unwrap();
    let signer = client.create_signer("local-key").await.unwrap();

    let error = signer
        .sign_transaction(&TransactionRequest::default())
        .await
        .unwrap_err();

    assert!(matches!(
        error,
        AlloyClientError::UnsupportedTransactionRequest {
            method: "sign_transaction",
            ..
        }
    ));
    assert!(server.received_requests().await.unwrap().is_empty());
}
