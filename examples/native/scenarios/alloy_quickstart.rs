//! Composed native Alloy client, first touch.
//!
//! Builds the umbrella `AlloyClient` (`builder().build_checked()`) against a
//! wiremock JSON-RPC server, derives a `Signer` (`create_signer`), and signs a
//! message — the shortest path into the native Alloy adapter without a live RPC.

use std::error::Error;

use cow_sdk::alloy::AlloyClient;
use cow_sdk::core::{Provider, Signer, SigningProvider, SupportedChainId};
use cow_sdk_examples_native::support::TEST_KEY;
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Mock JSON-RPC server: eth_chainId returns 0x1 (mainnet), so build_checked passes.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x1",
        })))
        .mount(&server)
        .await;

    // build_checked() verifies the configured chain id against the RPC endpoint.
    let client = AlloyClient::builder()
        .http(server.uri())?
        .private_key(TEST_KEY)?
        .chain_id(SupportedChainId::Mainnet)
        .build_checked()
        .await?;
    // Derive a Signer from the client and sign a message with it.
    let signer = client.create_signer("local-key").await?;
    let signature = signer.sign_message(b"hello cow").await?;

    let report = json!({
        "surface": "cow_sdk::alloy::AlloyClient",
        "chainId": client.get_chain_id().await?,
        "signer": signer.address().await?.to_hex_string(),
        "messageSignatureBytes": (signature.len() - 2) / 2
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
