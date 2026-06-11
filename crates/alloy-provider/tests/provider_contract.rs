use std::time::Duration;

use cow_sdk_alloy_provider::{ProviderError, RpcAlloyProvider};
use cow_sdk_core::{
    Address, Amount, ContractCall, HexData, Provider, TransactionHash, TransactionRequest,
    TransactionStatus, TransportErrorClass,
};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const HASH: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";
const ADDRESS: &str = "0x1111111111111111111111111111111111111111";

#[tokio::test]
async fn get_chain_id_returns_decimal_chain_id() {
    let (server, provider) = provider_with_result(json!("0x1")).await;

    assert_eq!(provider.get_chain_id().await.unwrap(), 1);
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn get_code_returns_none_for_empty_code() {
    let (_server, provider) = provider_with_result(json!("0x")).await;

    assert_eq!(
        provider
            .get_code(&Address::new(ADDRESS).unwrap())
            .await
            .unwrap(),
        None
    );
}

#[tokio::test]
async fn get_code_returns_present_bytecode() {
    let (_server, provider) = provider_with_result(json!("0x60016002")).await;

    assert_eq!(
        provider
            .get_code(&Address::new(ADDRESS).unwrap())
            .await
            .unwrap(),
        Some(HexData::new("0x60016002").unwrap())
    );
}

#[tokio::test]
async fn get_storage_at_returns_32_byte_word() {
    let word = format!("0x{:0>64}", "2a");
    let (_server, provider) = provider_with_result(json!(word)).await;

    assert_eq!(
        provider
            .get_storage_at(&Address::new(ADDRESS).unwrap(), "0x0")
            .await
            .unwrap(),
        HexData::new(format!("0x{:0>64}", "2a")).unwrap()
    );
}

#[tokio::test]
async fn get_block_accepts_latest_tag() {
    let (_server, provider) = provider_with_result(block_response("0x2a")).await;

    let block = provider.get_block("latest").await.unwrap();
    assert_eq!(block.number, 42);
    assert_eq!(block.hash.unwrap().to_hex_string(), HASH);
}

#[tokio::test]
async fn get_block_accepts_decimal_tag() {
    let (_server, provider) = provider_with_result(block_response("0x2a")).await;

    let block = provider.get_block("42").await.unwrap();
    assert_eq!(block.number, 42);
}

#[tokio::test]
async fn get_transaction_receipt_returns_none_for_null() {
    let (_server, provider) = provider_with_result(Value::Null).await;

    assert!(
        provider
            .get_transaction_receipt(&TransactionHash::new(HASH).unwrap())
            .await
            .unwrap()
            .is_none()
    );
}

#[tokio::test]
async fn get_transaction_receipt_returns_rich_receipt() {
    let (_server, provider) = provider_with_result(receipt_response()).await;

    let receipt = provider
        .get_transaction_receipt(&TransactionHash::new(HASH).unwrap())
        .await
        .unwrap()
        .unwrap();
    assert_eq!(receipt.transaction_hash.to_hex_string(), HASH);
    assert_eq!(receipt.status, Some(TransactionStatus::Success));
    assert_eq!(receipt.block_number, Some(42));
    assert_eq!(receipt.block_hash.unwrap().to_hex_string(), HASH);
    assert_eq!(receipt.gas_used, Some(Amount::from(21_000u64)));
    assert_eq!(receipt.from.unwrap().to_hex_string(), ADDRESS);
    assert_eq!(receipt.to.unwrap().to_hex_string(), ADDRESS);
}

#[tokio::test]
async fn get_transaction_receipt_populates_status_block_gas_from_to() {
    let (_server, provider) = provider_with_result(full_receipt_response()).await;

    let receipt = provider
        .get_transaction_receipt(&TransactionHash::new(HASH).unwrap())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(receipt.status, Some(TransactionStatus::Success));
    assert_eq!(receipt.block_number, Some(1234));
    assert_eq!(receipt.block_hash.unwrap().to_hex_string(), HASH);
    assert_eq!(receipt.gas_used, Some(Amount::from(30_000u64)));
    assert_eq!(receipt.from.unwrap().to_hex_string(), ADDRESS);
    assert_eq!(receipt.to.unwrap().to_hex_string(), ADDRESS);
}

#[tokio::test]
async fn call_returns_hex_data() {
    let (_server, provider) = provider_with_result(json!("0x1234")).await;
    let tx = TransactionRequest::new(
        Some(Address::new(ADDRESS).unwrap()),
        Some(HexData::new("0xabcdef").unwrap()),
        Some(Amount::from(1u32)),
        Some(Amount::from(21_000u32)),
    );

    assert_eq!(provider.call(&tx).await.unwrap().to_hex_string(), "0x1234");
}

#[tokio::test]
async fn get_contract_returns_value_handle_without_rpc() {
    let server = MockServer::start().await;
    let provider = provider_for(&server);
    let address = Address::new(ADDRESS).unwrap();

    let handle = provider.get_contract(&address, "[]").await.unwrap();

    assert_eq!(handle.address, address);
    assert_eq!(handle.abi_json, "[]");
    assert!(server.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn read_contract_balance_of_happy_path() {
    let encoded = format!("0x{:0>64}", "2a");
    let (_server, provider) = provider_with_result(json!(encoded)).await;
    let request = ContractCall::new(
        Address::new(ADDRESS).unwrap(),
        "balanceOf".to_owned(),
        r#"[{"type":"function","name":"balanceOf","inputs":[{"name":"owner","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#.to_owned(),
        format!(r#"["{ADDRESS}"]"#),
    );

    assert_eq!(provider.read_contract(&request).await.unwrap(), r#""42""#);
}

#[tokio::test]
async fn read_contract_domain_separator_happy_path() {
    let encoded = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let (_server, provider) = provider_with_result(json!(encoded)).await;
    let request = ContractCall::new(
        Address::new(ADDRESS).unwrap(),
        "domainSeparator".to_owned(),
        r#"[{"type":"function","name":"domainSeparator","inputs":[],"outputs":[{"name":"","type":"bytes32"}],"stateMutability":"view"}]"#.to_owned(),
        "[]".to_owned(),
    );

    assert_eq!(
        provider.read_contract(&request).await.unwrap(),
        r#""0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa""#
    );
}

#[tokio::test]
async fn network_failure_maps_to_transport_error_class() {
    let provider = RpcAlloyProvider::builder()
        .http("http://127.0.0.1:9")
        .unwrap()
        .timeout(Duration::from_millis(200))
        .build()
        .unwrap();

    let error = provider.get_chain_id().await.unwrap_err();
    assert_eq!(
        error.class(),
        cow_sdk_alloy_provider::ProviderErrorClass::Transport
    );
}

#[tokio::test]
async fn jsonrpc_error_maps_to_remote_error_class() {
    let (server, provider) = provider_with_response(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": { "code": -32000, "message": "execution reverted" }
    }))
    .await;

    let error = provider.get_chain_id().await.unwrap_err();

    assert!(matches!(
        error,
        ProviderError::Remote {
            code: -32000,
            message
        } if message == "execution reverted"
    ));
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn null_response_maps_to_decode_transport_class() {
    let (_server, provider) = provider_with_result(Value::Null).await;

    let error = provider.get_chain_id().await.unwrap_err();

    assert!(matches!(
        error,
        ProviderError::Transport {
            class: TransportErrorClass::Decode,
            ..
        }
    ));
}

async fn provider_with_result(result: Value) -> (MockServer, RpcAlloyProvider) {
    provider_with_response(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": result,
    }))
    .await
}

async fn provider_with_response(response: Value) -> (MockServer, RpcAlloyProvider) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;
    let provider = provider_for(&server);
    (server, provider)
}

fn provider_for(server: &MockServer) -> RpcAlloyProvider {
    RpcAlloyProvider::builder()
        .http(server.uri())
        .unwrap()
        .build()
        .unwrap()
}

fn block_response(number: &str) -> Value {
    json!({
        "hash": HASH,
        "parentHash": HASH,
        "sha3Uncles": "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
        "miner": ADDRESS,
        "stateRoot": HASH,
        "transactionsRoot": HASH,
        "receiptsRoot": HASH,
        "logsBloom": format!("0x{}", "00".repeat(256)),
        "difficulty": "0x0",
        "number": number,
        "gasLimit": "0x1c9c380",
        "gasUsed": "0x5208",
        "timestamp": "0x5",
        "extraData": "0x",
        "mixHash": HASH,
        "nonce": "0x0000000000000000",
        "baseFeePerGas": "0x1",
        "transactions": [],
        "uncles": [],
        "totalDifficulty": "0x0",
        "size": "0x1",
    })
}

fn receipt_response() -> Value {
    json!({
        "transactionHash": HASH,
        "transactionIndex": "0x0",
        "blockHash": HASH,
        "blockNumber": "0x2a",
        "from": ADDRESS,
        "to": ADDRESS,
        "contractAddress": null,
        "gasUsed": "0x5208",
        "effectiveGasPrice": "0x1",
        "cumulativeGasUsed": "0x5208",
        "logsBloom": format!("0x{}", "00".repeat(256)),
        "status": "0x1",
        "logs": [],
        "type": "0x2"
    })
}

fn full_receipt_response() -> Value {
    json!({
        "transactionHash": HASH,
        "transactionIndex": "0x0",
        "blockHash": HASH,
        "blockNumber": "0x4d2",
        "from": ADDRESS,
        "to": ADDRESS,
        "contractAddress": null,
        "gasUsed": "0x7530",
        "effectiveGasPrice": "0x1",
        "cumulativeGasUsed": "0x7530",
        "logsBloom": format!("0x{}", "00".repeat(256)),
        "status": "0x1",
        "logs": [],
        "type": "0x2"
    })
}
