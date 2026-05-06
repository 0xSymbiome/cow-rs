use cow_sdk_alloy_provider::RpcAlloyProvider;
use cow_sdk_core::{Address, AsyncProvider, ContractCall};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const ERC20_ALLOWANCE_ABI: &str = r#"[{"type":"function","name":"allowance","inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;
const EIP1271_ABI: &str = r#"[{"type":"function","name":"isValidSignature","inputs":[{"name":"hash","type":"bytes32"},{"name":"signature","type":"bytes"}],"outputs":[{"name":"","type":"bytes4"}],"stateMutability":"view"}]"#;

#[tokio::test]
async fn read_contract_uint256_allowance_matches_browser_wallet_shape() {
    let response = format!("0x{:0>64x}", 10_000_000_000_000_000_000_u128);
    let provider = provider_with_eth_call(&response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x33; 20]),
        "allowance".to_owned(),
        ERC20_ALLOWANCE_ABI.to_owned(),
        serde_json::to_string(&[
            Address::from_bytes([0x11; 20]).as_str(),
            Address::from_bytes([0x22; 20]).as_str(),
        ])
        .unwrap(),
    );

    let result = provider.read_contract(&request).await.unwrap();

    assert_eq!(result, r#""10000000000000000000""#);
}

#[tokio::test]
async fn read_contract_bytes4_eip1271_magic_value_matches_browser_wallet_shape() {
    let provider = provider_with_eth_call(
        "0x1626ba7e00000000000000000000000000000000000000000000000000000000",
    )
    .await;
    let request = ContractCall::new(
        Address::from_bytes([0x44; 20]),
        "isValidSignature".to_owned(),
        EIP1271_ABI.to_owned(),
        r#"["0x0000000000000000000000000000000000000000000000000000000000000001","0xdeadbeef"]"#
            .to_owned(),
    );

    let result = provider.read_contract(&request).await.unwrap();

    assert_eq!(result, r#""0x1626ba7e""#);
}

async fn provider_with_eth_call(result: &str) -> RpcAlloyProvider {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": Value::String(result.to_owned()),
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
