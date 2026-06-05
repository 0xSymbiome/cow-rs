//! Composed native Alloy client, first touch.
//!
//! Builds the umbrella `AlloyClient` (`builder().build_checked()`) against a
//! wiremock JSON-RPC server, derives a `Signer` (`create_signer`), and signs a
//! message — the shortest path into the native Alloy adapter without a live RPC.

use std::error::Error;

use cow_sdk::alloy::AlloyClient;
use cow_sdk::core::{Provider, Signer, SigningProvider};
use cow_sdk::prelude::SupportedChainId;
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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
    let signer = client.create_signer("local-key").await?;
    let signature = signer.sign_message(b"hello cow").await?;

    let report = json!({
        "surface": "cow-sdk::alloy::AlloyClient",
        "chainId": client.get_chain_id().await?,
        "signer": signer.get_address().await?.to_hex_string(),
        "messageSignatureBytes": (signature.len() - 2) / 2
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
