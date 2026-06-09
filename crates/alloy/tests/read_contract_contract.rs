//! Behaviour tests for the umbrella Alloy adapter's `read_contract` surface.
//!
//! `AlloyClient::read_contract` is a thin delegation to the provider leaf's
//! `__seam::execute_read_contract` (see `crates/alloy/src/client.rs`) — the
//! umbrella ships no independent decoder. The full ABI-type matrix (every
//! scalar, compound, argument shape, and rejection path) is therefore
//! exercised once, on the leaf, in
//! `crates/alloy-provider/tests/read_contract_parity.rs`, and the umbrella's
//! byte-for-byte agreement with the leaf output is pinned by the workspace
//! `tests/alloy_read_contract_parity_invariant.rs` regression test.
//!
//! What is umbrella-specific — and therefore pinned here — is the end-to-end
//! delegation producing a decoded value at the public `AlloyClient` boundary,
//! and the `ProviderError -> AlloyClientError::Validation` lift that preserves
//! the rejection discriminant through `read_contract`.

#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::{AlloyClient, AlloyClientError};
use cow_sdk_core::{Address, ContractCall, Provider, SupportedChainId};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

const ERC20_ALLOWANCE_ABI: &str = r#"[{"type":"function","name":"allowance","inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;

async fn client_with_eth_call(result: &str) -> AlloyClient {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": Value::String(result.to_owned()),
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
    // Keep the wiremock server alive for the duration of the test by leaking
    // it into a static slot; the process tears it down on test exit.
    let _server: &'static MockServer = Box::leak(Box::new(server));
    client
}

/// The umbrella decodes a real `eth_call` result end-to-end through the seam
/// delegation and surfaces the leaf's canonical JSON at the public boundary.
/// (Per-type decoding is the leaf's responsibility, covered exhaustively in
/// `crates/alloy-provider/tests/read_contract_parity.rs`.)
#[tokio::test]
async fn read_contract_returns_uint256_for_allowance_call() {
    let response = format!("0x{:0>64x}", 10_000_000_000_000_000_000_u128);
    let client = client_with_eth_call(&response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x33; 20]),
        "allowance".to_owned(),
        ERC20_ALLOWANCE_ABI.to_owned(),
        serde_json::to_string(&[
            Address::from_bytes([0x11; 20]).to_hex_string(),
            Address::from_bytes([0x22; 20]).to_hex_string(),
        ])
        .unwrap(),
    );

    let result = client.read_contract(&request).await.unwrap();

    assert_eq!(result, r#""10000000000000000000""#);
}

// -------------------------------------------------------------------------
// Umbrella-side variant pins. The leaf provider's `execute_read_contract`
// emits `ProviderError::Validation` on every rejection path; the umbrella's
// `From<ProviderError>` impl preserves the discriminant so
// `AlloyClient::read_contract` surfaces `AlloyClientError::Validation`
// end-to-end. These pins lock that contract at the umbrella layer.
// -------------------------------------------------------------------------

#[tokio::test]
async fn read_contract_invalid_abi_type_surfaces_validation_variant() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x40; 20]),
        "f".to_owned(),
        r#"[{"type":"function","name":"f","inputs":[{"name":"x","type":"notatype"}],"outputs":[],"stateMutability":"view"}]"#
            .to_owned(),
        r#"["0"]"#.to_owned(),
    );

    let err = client
        .read_contract(&request)
        .await
        .expect_err("malformed ABI type must reject the call");

    assert!(
        matches!(err, AlloyClientError::Validation(_)),
        "expected AlloyClientError::Validation, got {err:?}",
    );
}

#[tokio::test]
async fn read_contract_overload_resolution_surfaces_validation_variant() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x41; 20]),
        "f".to_owned(),
        r#"[{"type":"function","name":"f","inputs":[{"type":"uint256","name":"a"}],"outputs":[]},{"type":"function","name":"f","inputs":[{"type":"address","name":"a"}],"outputs":[]}]"#
            .to_owned(),
        r#"["0"]"#.to_owned(),
    );

    let err = client
        .read_contract(&request)
        .await
        .expect_err("overloaded function must reject the call");

    assert!(
        matches!(err, AlloyClientError::Validation(_)),
        "expected AlloyClientError::Validation, got {err:?}",
    );
}
