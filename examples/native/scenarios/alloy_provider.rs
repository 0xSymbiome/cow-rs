//! Read-only Alloy provider leaf in isolation.
//!
//! Uses `RpcAlloyProvider` (the provider leaf, no signer) against a wiremock
//! JSON-RPC server to read contract code (`Provider::get_code`) — the smallest
//! Alloy surface for read-only, analytics, or query-only integrations.

use std::error::Error;

use cow_sdk::alloy_provider::RpcAlloyProvider;
use cow_sdk::core::{Address, Provider};
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const ADDRESS: &str = "0x1111111111111111111111111111111111111111";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Stand up a wiremock JSON-RPC server; every POST returns this canned result.
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x60016002",
        })))
        .mount(&server)
        .await;

    // Build the provider leaf (no signer) pointed at the mock RPC.
    let provider = RpcAlloyProvider::builder()
        .http(server.uri())?
        .build()
        .await?;

    // Read the on-chain bytecode at an address through the `Provider` trait.
    let code = provider.get_code(&Address::new(ADDRESS)?).await?;

    let report = json!({
        "surface": "cow-sdk::alloy_provider::RpcAlloyProvider",
        "address": ADDRESS,
        "code": code.map(|data| data.to_hex_string())
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
