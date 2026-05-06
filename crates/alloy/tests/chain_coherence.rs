#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{AsyncProvider, AsyncSigningProvider, SupportedChainId};
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

#[tokio::test]
async fn signer_and_provider_are_bound_to_the_same_chain_id() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x1",
        })))
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
    let handle = client.create_signer("local-key").await.unwrap();

    assert_eq!(client.chain_id(), u64::from(SupportedChainId::Mainnet));
    assert_eq!(handle.chain_id(), client.chain_id());
    assert_eq!(client.get_chain_id().await.unwrap(), handle.chain_id());
}
