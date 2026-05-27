use cow_sdk_alloy_provider::{ProviderError, RpcAlloyProvider};
use cow_sdk_core::{Address, ContractCall, Provider};
use serde_json::json;
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

#[tokio::test]
async fn read_contract_no_panic_on_malformed_uint256() {
    let provider = provider().await;
    let request = call(
        r#"[{"type":"function","name":"f","inputs":[{"name":"a","type":"uint256"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#,
        r#"["not-a-number"]"#,
    );

    assert_validation(&provider.read_contract(&request).await);
}

#[tokio::test]
async fn read_contract_no_panic_on_malformed_address() {
    let provider = provider().await;
    let request = call(
        r#"[{"type":"function","name":"f","inputs":[{"name":"a","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#,
        r#"["0x1234"]"#,
    );

    assert_validation(&provider.read_contract(&request).await);
}

#[tokio::test]
async fn read_contract_no_panic_on_malformed_abi_fragment() {
    let provider = provider().await;
    let request = call("not-json", "[]");

    assert_validation(&provider.read_contract(&request).await);
}

#[tokio::test]
async fn read_contract_no_panic_on_malformed_args_json() {
    let provider = provider().await;
    let request = call(
        r#"[{"type":"function","name":"f","inputs":[{"name":"a","type":"uint256"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#,
        "[",
    );

    assert_validation(&provider.read_contract(&request).await);
}

fn call(abi_json: &str, args_json: &str) -> ContractCall {
    ContractCall::new(
        Address::from_bytes([0x11; 20]),
        "f".to_owned(),
        abi_json.to_owned(),
        args_json.to_owned(),
    )
}

fn assert_validation(result: &Result<String, ProviderError>) {
    assert!(matches!(result, Err(ProviderError::Validation(_))));
}

async fn provider() -> RpcAlloyProvider {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": "0x"
        })))
        .mount(&server)
        .await;
    RpcAlloyProvider::builder()
        .http(server.uri())
        .unwrap()
        .build()
        .await
        .unwrap()
}
