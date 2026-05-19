#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{AsyncSigner, AsyncSigningProvider, SupportedChainId};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const EXPECTED_ADDRESS: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";
const EXPECTED_MESSAGE_SIGNATURE: &str = "0x267c1300572586cc72a2780636139a843ce20866dcc515c62c02909f0bbf3ce71468a683b857347aced6470cd911828201eb0fe21e2ba3bcf14f903916407d101b";

#[tokio::test]
async fn signer_handle_remains_usable_after_parent_client_drop() {
    let client = AlloyClient::builder()
        .http("http://127.0.0.1:9")
        .unwrap()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
        .unwrap();
    let handle = client.create_signer("local-key").await.unwrap();
    drop(client);

    assert_eq!(
        handle.get_address().await.unwrap().to_hex_string(),
        EXPECTED_ADDRESS
    );
    assert_eq!(
        handle.sign_message(b"hello cow").await.unwrap(),
        EXPECTED_MESSAGE_SIGNATURE
    );
}
