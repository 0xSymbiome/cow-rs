#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_alloy_provider::RpcAlloyProvider;
use cow_sdk_core::{Address, ContractCall, Provider, SupportedChainId};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const ERC20_ALLOWANCE_ABI: &str = r#"[{"type":"function","name":"allowance","inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;
const EIP1271_ABI: &str = r#"[{"type":"function","name":"isValidSignature","inputs":[{"name":"hash","type":"bytes32"},{"name":"signature","type":"bytes"}],"outputs":[{"name":"","type":"bytes4"}],"stateMutability":"view"}]"#;
const TUPLE_ABI: &str = r#"[{"type":"function","name":"quote","inputs":[],"outputs":[{"name":"","type":"tuple","components":[{"name":"amount","type":"uint256"},{"name":"ok","type":"bool"}]}],"stateMutability":"view"}]"#;

#[tokio::test]
async fn provider_leaf_and_umbrella_read_contract_outputs_match_byte_for_byte() {
    let fixtures = [
        Fixture {
            name: "uint256-allowance",
            abi_json: ERC20_ALLOWANCE_ABI,
            method: "allowance",
            args_json: r#"["0x1111111111111111111111111111111111111111","0x2222222222222222222222222222222222222222"]"#,
            return_hex: "0x0000000000000000000000000000000000000000000000008ac7230489e80000",
        },
        Fixture {
            name: "bytes4-EIP-1271-magic-value",
            abi_json: EIP1271_ABI,
            method: "isValidSignature",
            args_json: r#"["0x0000000000000000000000000000000000000000000000000000000000000001","0xdeadbeef"]"#,
            return_hex: "0x1626ba7e00000000000000000000000000000000000000000000000000000000",
        },
        Fixture {
            name: "tuple-ordered-args",
            abi_json: TUPLE_ABI,
            method: "quote",
            args_json: "[]",
            return_hex: "0x00000000000000000000000000000000000000000000000000000000000000070000000000000000000000000000000000000000000000000000000000000001",
        },
    ];

    for fixture in fixtures {
        assert_fixture_matches(fixture).await;
    }
}

async fn assert_fixture_matches(fixture: Fixture<'_>) {
    let server = eth_call_server(fixture.return_hex).await;
    let request = ContractCall::new(
        Address::from_bytes([0x33; 20]),
        fixture.method.to_owned(),
        fixture.abi_json.to_owned(),
        fixture.args_json.to_owned(),
    );
    let provider = RpcAlloyProvider::builder()
        .http(server.uri())
        .expect("provider transport")
        .build()
        .expect("provider build");
    let client = AlloyClient::builder()
        .http(server.uri())
        .expect("client transport")
        .private_key(TEST_KEY)
        .expect("client key")
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
        .expect("client build");

    let provider_json = provider
        .read_contract(&request)
        .await
        .unwrap_or_else(|error| panic!("provider fixture `{}` failed: {error}", fixture.name));
    let umbrella_json = client
        .read_contract(&request)
        .await
        .unwrap_or_else(|error| panic!("umbrella fixture `{}` failed: {error}", fixture.name));

    assert_eq!(
        provider_json, umbrella_json,
        "fixture `{}` drifted between provider leaf and umbrella",
        fixture.name
    );
}

async fn eth_call_server(result: &str) -> MockServer {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": Value::String(result.to_owned()),
        })))
        .mount(&server)
        .await;
    server
}

#[derive(Clone, Copy)]
struct Fixture<'a> {
    name: &'a str,
    abi_json: &'a str,
    method: &'a str,
    args_json: &'a str,
    return_hex: &'a str,
}
