#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{Signer, SigningProvider, SupportedChainId};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const EXPECTED_ADDRESS: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";

#[tokio::test]
async fn create_signer_returns_owned_handle() {
    fn requires_signing_provider<T: SigningProvider>() {}
    requires_signing_provider::<AlloyClient>();

    let client = test_client().await;
    let handle = client
        .create_signer("local-key")
        .await
        .expect("signer handle creation should succeed");

    assert_eq!(
        handle.get_address().await.unwrap().to_hex_string(),
        EXPECTED_ADDRESS
    );
    assert_eq!(handle.chain_id(), u64::from(SupportedChainId::Mainnet));
}

async fn test_client() -> AlloyClient {
    AlloyClient::builder()
        .http("http://127.0.0.1:9")
        .unwrap()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
        .unwrap()
}
