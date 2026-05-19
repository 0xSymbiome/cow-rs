//! Behaviour tests for the umbrella Alloy adapter's `read_contract` path.
//!
//! These tests drive `AlloyClient::read_contract` through wiremock-served
//! `eth_call` responses, exercising the recursive `DynSolType` walk on every
//! documented scalar (`uint256`, `int256`, `bool`, `string`, `bytes`,
//! `bytes32`, `address`), every compound shape (dynamic arrays, fixed arrays,
//! multi-output tuples), and the documented error paths (invalid ABI type,
//! wrong argument count, type mismatch on input).
//!
//! `crates/alloy-provider/tests/read_contract_parity.rs` exercises the
//! identical surface on the provider adapter. The umbrella and provider
//! adapters intentionally ship duplicated `read_contract.rs` implementations
//! per the architecture seam — these matching fixtures keep the two
//! implementations in lockstep.

#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{Address, AsyncProvider, ContractCall, SupportedChainId};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

const ERC20_ALLOWANCE_ABI: &str = r#"[{"type":"function","name":"allowance","inputs":[{"name":"owner","type":"address"},{"name":"spender","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;
const INT256_ABI: &str = r#"[{"type":"function","name":"signedAmount","inputs":[{"name":"value","type":"int256"}],"outputs":[{"name":"","type":"int256"}],"stateMutability":"view"}]"#;
const BOOL_ABI: &str = r#"[{"type":"function","name":"isPaused","inputs":[],"outputs":[{"name":"","type":"bool"}],"stateMutability":"view"}]"#;
const STRING_ABI: &str = r#"[{"type":"function","name":"name","inputs":[],"outputs":[{"name":"","type":"string"}],"stateMutability":"view"}]"#;
const BYTES_ABI: &str = r#"[{"type":"function","name":"getCode","inputs":[],"outputs":[{"name":"","type":"bytes"}],"stateMutability":"view"}]"#;
const BYTES32_ABI: &str = r#"[{"type":"function","name":"slot","inputs":[],"outputs":[{"name":"","type":"bytes32"}],"stateMutability":"view"}]"#;
const ADDRESS_ABI: &str = r#"[{"type":"function","name":"owner","inputs":[],"outputs":[{"name":"","type":"address"}],"stateMutability":"view"}]"#;
const DYNAMIC_ARRAY_ABI: &str = r#"[{"type":"function","name":"holders","inputs":[],"outputs":[{"name":"","type":"uint256[]"}],"stateMutability":"view"}]"#;
const FIXED_ARRAY_ABI: &str = r#"[{"type":"function","name":"triple","inputs":[],"outputs":[{"name":"","type":"uint256[3]"}],"stateMutability":"view"}]"#;
const MULTI_OUTPUT_ABI: &str = r#"[{"type":"function","name":"snapshot","inputs":[],"outputs":[{"name":"","type":"uint256"},{"name":"","type":"address"}],"stateMutability":"view"}]"#;

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

#[tokio::test]
async fn read_contract_returns_signed_int_for_negative_value() {
    let response = "0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffd6";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x55; 20]),
        "signedAmount".to_owned(),
        INT256_ABI.to_owned(),
        r#"["-1"]"#.to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#""-42""#);
}

#[tokio::test]
async fn read_contract_returns_signed_int_for_hex_argument() {
    let response = "0x000000000000000000000000000000000000000000000000000000000000002a";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x55; 20]),
        "signedAmount".to_owned(),
        INT256_ABI.to_owned(),
        r#"["0x2a"]"#.to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#""42""#);
}

#[tokio::test]
async fn read_contract_returns_bool_for_true_response() {
    let response = "0x0000000000000000000000000000000000000000000000000000000000000001";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x66; 20]),
        "isPaused".to_owned(),
        BOOL_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, "true");
}

#[tokio::test]
async fn read_contract_returns_string_for_utf8_encoded_response() {
    let response = "0x0000000000000000000000000000000000000000000000000000000000000020\
                    0000000000000000000000000000000000000000000000000000000000000003\
                    436f570000000000000000000000000000000000000000000000000000000000";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x77; 20]),
        "name".to_owned(),
        STRING_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#""CoW""#);
}

#[tokio::test]
async fn read_contract_returns_bytes_as_hex_string() {
    let response = "0x0000000000000000000000000000000000000000000000000000000000000020\
                    0000000000000000000000000000000000000000000000000000000000000004\
                    deadbeef00000000000000000000000000000000000000000000000000000000";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x88; 20]),
        "getCode".to_owned(),
        BYTES_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#""0xdeadbeef""#);
}

#[tokio::test]
async fn read_contract_returns_bytes32_as_hex_string() {
    let response = "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x99; 20]),
        "slot".to_owned(),
        BYTES32_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(
        result,
        r#""0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f""#,
    );
}

#[tokio::test]
async fn read_contract_returns_address_as_lowercase_hex() {
    let response = "0x000000000000000000000000aaaabbbbccccddddeeeeffff0000111122223333";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0xaa; 20]),
        "owner".to_owned(),
        ADDRESS_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert!(
        result
            .to_ascii_lowercase()
            .contains("0xaaaabbbbccccddddeeeeffff0000111122223333"),
        "address output should contain the canonical lowercase hex; got {result}",
    );
}

#[tokio::test]
async fn read_contract_returns_dynamic_array_as_json_array() {
    let response = "0x0000000000000000000000000000000000000000000000000000000000000020\
                    0000000000000000000000000000000000000000000000000000000000000002\
                    0000000000000000000000000000000000000000000000000000000000000001\
                    0000000000000000000000000000000000000000000000000000000000000002";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0xbb; 20]),
        "holders".to_owned(),
        DYNAMIC_ARRAY_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#"["1","2"]"#);
}

#[tokio::test]
async fn read_contract_returns_fixed_array_as_json_array() {
    let response = "0x000000000000000000000000000000000000000000000000000000000000000a\
                    0000000000000000000000000000000000000000000000000000000000000014\
                    000000000000000000000000000000000000000000000000000000000000001e";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0xcc; 20]),
        "triple".to_owned(),
        FIXED_ARRAY_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#"["10","20","30"]"#);
}

#[tokio::test]
async fn read_contract_returns_multi_output_function_as_json_array() {
    let response = "0x000000000000000000000000000000000000000000000000000000000000002a\
                    0000000000000000000000001111111111111111111111111111111111111111";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0xdd; 20]),
        "snapshot".to_owned(),
        MULTI_OUTPUT_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert!(
        result.starts_with('[') && result.ends_with(']'),
        "multi-output result must be a JSON array; got {result}",
    );
    assert!(result.contains(r#""42""#));
    assert!(
        result
            .to_ascii_lowercase()
            .contains("0x1111111111111111111111111111111111111111"),
    );
}

#[tokio::test]
async fn read_contract_rejects_invalid_abi_type_string() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0xee; 20]),
        "f".to_owned(),
        r#"[{"type":"function","name":"f","inputs":[{"name":"x","type":"notatype"}],"outputs":[],"stateMutability":"view"}]"#
            .to_owned(),
        r#"["0"]"#.to_owned(),
    );

    let result = client.read_contract(&request).await;
    assert!(
        result.is_err(),
        "malformed ABI type must reject the call; got {result:?}",
    );
}

#[tokio::test]
async fn read_contract_rejects_wrong_argument_count() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0xff; 20]),
        "allowance".to_owned(),
        ERC20_ALLOWANCE_ABI.to_owned(),
        r#"["0x1111111111111111111111111111111111111111"]"#.to_owned(),
    );

    let result = client.read_contract(&request).await;
    assert!(
        result.is_err(),
        "wrong argument count must reject the call; got {result:?}",
    );
}

#[tokio::test]
async fn read_contract_rejects_bool_for_address_input() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x10; 20]),
        "allowance".to_owned(),
        ERC20_ALLOWANCE_ABI.to_owned(),
        r#"[true, "0x1111111111111111111111111111111111111111"]"#.to_owned(),
    );

    let result = client.read_contract(&request).await;
    assert!(
        result.is_err(),
        "type mismatch on address input must reject the call; got {result:?}",
    );
}

// -------------------------------------------------------------------------
// Argument-shape variations and small-bit-width integers
// -------------------------------------------------------------------------

const UINT8_ABI: &str = r#"[{"type":"function","name":"decimals","inputs":[],"outputs":[{"name":"","type":"uint8"}],"stateMutability":"view"}]"#;
const TUPLE_INPUT_ABI: &str = r#"[{"type":"function","name":"swap","inputs":[{"name":"in","type":"uint256"},{"name":"min_out","type":"uint256"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;
const SINGLE_INPUT_ABI: &str = r#"[{"type":"function","name":"balanceOf","inputs":[{"name":"owner","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#;
const STRING_INPUT_ABI: &str = r#"[{"type":"function","name":"hashName","inputs":[{"name":"name","type":"string"}],"outputs":[{"name":"","type":"bytes32"}],"stateMutability":"view"}]"#;
const BOOL_INPUT_ABI: &str = r#"[{"type":"function","name":"setFlag","inputs":[{"name":"flag","type":"bool"}],"outputs":[],"stateMutability":"view"}]"#;

#[tokio::test]
async fn read_contract_returns_uint8_for_decimals_call() {
    let response = "0x0000000000000000000000000000000000000000000000000000000000000012";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x21; 20]),
        "decimals".to_owned(),
        UINT8_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#""18""#);
}

#[tokio::test]
async fn read_contract_accepts_json_object_with_named_params() {
    let response = "0x000000000000000000000000000000000000000000000000000000000000007b";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x22; 20]),
        "swap".to_owned(),
        TUPLE_INPUT_ABI.to_owned(),
        r#"{"in":"100","min_out":"50"}"#.to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#""123""#);
}

#[tokio::test]
async fn read_contract_rejects_json_object_missing_a_named_param() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x23; 20]),
        "swap".to_owned(),
        TUPLE_INPUT_ABI.to_owned(),
        r#"{"in":"100"}"#.to_owned(),
    );

    let result = client.read_contract(&request).await;
    assert!(
        result.is_err(),
        "missing named param must reject the call; got {result:?}",
    );
}

#[tokio::test]
async fn read_contract_accepts_scalar_for_single_unnamed_input() {
    let response = "0x0000000000000000000000000000000000000000000000000000000000000064";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x24; 20]),
        "balanceOf".to_owned(),
        SINGLE_INPUT_ABI.to_owned(),
        r#""0x1111111111111111111111111111111111111111""#.to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#""100""#);
}

#[tokio::test]
async fn read_contract_accepts_string_argument_with_utf8_value() {
    let response = "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x25; 20]),
        "hashName".to_owned(),
        STRING_INPUT_ABI.to_owned(),
        r#"["CoW Protocol"]"#.to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert!(result.starts_with('"') && result.ends_with('"'));
}

#[tokio::test]
async fn read_contract_accepts_bool_argument() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x26; 20]),
        "setFlag".to_owned(),
        BOOL_INPUT_ABI.to_owned(),
        r"[true]".to_owned(),
    );

    let result = client.read_contract(&request).await;
    assert!(
        result.is_ok(),
        "bool argument with no-output ABI must succeed; got {result:?}",
    );
}

#[tokio::test]
async fn read_contract_rejects_object_for_address_input() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x27; 20]),
        "balanceOf".to_owned(),
        SINGLE_INPUT_ABI.to_owned(),
        r"[{}]".to_owned(),
    );

    let result = client.read_contract(&request).await;
    assert!(
        result.is_err(),
        "object argument for address input must reject; got {result:?}",
    );
}

// -------------------------------------------------------------------------
// Additional edge cases: empty values, byte arrays, bit widths, rejections
// -------------------------------------------------------------------------

const BYTES_INPUT_ABI: &str = r#"[{"type":"function","name":"submit","inputs":[{"name":"payload","type":"bytes"}],"outputs":[{"name":"","type":"bytes32"}],"stateMutability":"view"}]"#;
const BYTES32_INPUT_ABI: &str = r#"[{"type":"function","name":"recover","inputs":[{"name":"hash","type":"bytes32"}],"outputs":[{"name":"","type":"address"}],"stateMutability":"view"}]"#;
const UINT64_ABI: &str = r#"[{"type":"function","name":"timestamp","inputs":[],"outputs":[{"name":"","type":"uint64"}],"stateMutability":"view"}]"#;
const ADDRESS_ARRAY_ABI: &str = r#"[{"type":"function","name":"signers","inputs":[],"outputs":[{"name":"","type":"address[]"}],"stateMutability":"view"}]"#;

#[tokio::test]
async fn read_contract_returns_empty_dynamic_array() {
    let response = "0x0000000000000000000000000000000000000000000000000000000000000020\
                    0000000000000000000000000000000000000000000000000000000000000000";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x30; 20]),
        "holders".to_owned(),
        DYNAMIC_ARRAY_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, "[]");
}

#[tokio::test]
async fn read_contract_returns_empty_bytes() {
    let response = "0x0000000000000000000000000000000000000000000000000000000000000020\
                    0000000000000000000000000000000000000000000000000000000000000000";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x31; 20]),
        "getCode".to_owned(),
        BYTES_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#""0x""#);
}

#[tokio::test]
async fn read_contract_returns_address_array_as_json_array() {
    let response = "0x0000000000000000000000000000000000000000000000000000000000000020\
                    0000000000000000000000000000000000000000000000000000000000000002\
                    0000000000000000000000001111111111111111111111111111111111111111\
                    0000000000000000000000002222222222222222222222222222222222222222";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x32; 20]),
        "signers".to_owned(),
        ADDRESS_ARRAY_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    let lower = result.to_ascii_lowercase();
    assert!(lower.contains("0x1111111111111111111111111111111111111111"));
    assert!(lower.contains("0x2222222222222222222222222222222222222222"));
}

#[tokio::test]
async fn read_contract_returns_uint64_value() {
    let response = "0x000000000000000000000000000000000000000000000000000000006a4c6c2a";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x33; 20]),
        "timestamp".to_owned(),
        UINT64_ABI.to_owned(),
        "[]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert_eq!(result, r#""1783393322""#);
}

#[tokio::test]
async fn read_contract_accepts_bytes_input_as_hex_string() {
    let response = "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x34; 20]),
        "submit".to_owned(),
        BYTES_INPUT_ABI.to_owned(),
        r#"["0xdeadbeefcafebabe"]"#.to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert!(result.starts_with('"') && result.ends_with('"'));
}

#[tokio::test]
async fn read_contract_accepts_bytes_input_as_byte_array_form() {
    let response = "0x000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x35; 20]),
        "submit".to_owned(),
        BYTES_INPUT_ABI.to_owned(),
        r"[[222, 173, 190, 239]]".to_owned(),
    );

    let result = client.read_contract(&request).await.unwrap();
    assert!(result.starts_with('"') && result.ends_with('"'));
}

#[tokio::test]
async fn read_contract_rejects_bytes32_length_mismatch() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x36; 20]),
        "recover".to_owned(),
        BYTES32_INPUT_ABI.to_owned(),
        r#"["0xdeadbeef"]"#.to_owned(),
    );

    let result = client.read_contract(&request).await;
    assert!(
        result.is_err(),
        "bytes32 length mismatch must reject the call; got {result:?}",
    );
}

#[tokio::test]
async fn read_contract_rejects_null_for_int256_argument() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x37; 20]),
        "signedAmount".to_owned(),
        INT256_ABI.to_owned(),
        r"[null]".to_owned(),
    );

    let result = client.read_contract(&request).await;
    assert!(
        result.is_err(),
        "null argument for int256 input must reject; got {result:?}",
    );
}

#[tokio::test]
async fn read_contract_rejects_array_for_bool_argument() {
    let response = "0x";
    let client = client_with_eth_call(response).await;
    let request = ContractCall::new(
        Address::from_bytes([0x38; 20]),
        "setFlag".to_owned(),
        BOOL_INPUT_ABI.to_owned(),
        r"[[1, 2, 3]]".to_owned(),
    );

    let result = client.read_contract(&request).await;
    assert!(
        result.is_err(),
        "array argument for bool input must reject; got {result:?}",
    );
}
