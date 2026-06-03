#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{LogProvider, LogQuery, SupportedChainId};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

#[tokio::test]
async fn alloy_client_implements_log_provider_and_returns_typed_error_on_unreachable_rpc() {
    // Compile-time contract: the composed umbrella implements the `LogProvider`
    // capability, so a consumer can fetch event logs from the same client it
    // trades through without constructing a second provider for the same RPC.
    fn requires_log_provider<T: LogProvider>() {}
    requires_log_provider::<AlloyClient>();

    let client = test_client().await;

    // The endpoint is unreachable, so a single bounded `get_logs` must surface a
    // typed error rather than panic. This drives the seam `LogQuery` -> filter
    // conversion, the delegated `eth_getLogs` call on the composed provider, and
    // the Alloy-transport -> `AlloyClientError` mapping end to end.
    let query = LogQuery::new(1, 2);
    let result = client.get_logs(&query).await;
    assert!(
        result.is_err(),
        "get_logs against an unreachable endpoint must return a typed error",
    );
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
