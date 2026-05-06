#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::{AlloyClient, AlloyClientError};
use cow_sdk_core::{AsyncProvider, Cancellable, CancellationToken, SupportedChainId};
use wiremock::MockServer;

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

#[tokio::test]
async fn cancel_with_propagates_cancelled_through_question_mark() {
    let server = MockServer::start().await;
    let client = AlloyClient::builder()
        .http(server.uri())
        .unwrap()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
        .unwrap();
    let token = CancellationToken::new();
    token.cancel();

    let error = client.get_chain_id().cancel_with(&token).await.unwrap_err();

    assert!(matches!(error, AlloyClientError::Cancelled));
    assert!(server.received_requests().await.unwrap().is_empty());
}
