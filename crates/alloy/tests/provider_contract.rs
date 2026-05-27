#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{
    Address, Amount, Provider, ContractCall, HexData, SupportedChainId, TransactionHash,
    TransactionRequest, TransactionStatus,
};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const HASH: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";
const ADDRESS: &str = "0x1111111111111111111111111111111111111111";

#[tokio::test]
async fn get_chain_id_delegates_to_inner_provider() {
    let (server, client) = client_with_result(json!("0x1")).await;

    assert_eq!(client.get_chain_id().await.unwrap(), 1);
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn get_code_delegates_to_inner_provider() {
    let (_server, client) = client_with_result(json!("0x60016002")).await;

    assert_eq!(
        client
            .get_code(&Address::new(ADDRESS).unwrap())
            .await
            .unwrap(),
        Some(HexData::new("0x60016002").unwrap())
    );
}

#[tokio::test]
async fn get_transaction_receipt_delegates_to_inner_provider() {
    let (_server, client) = client_with_result(receipt_response()).await;

    let receipt = client
        .get_transaction_receipt(&TransactionHash::new(HASH).unwrap())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(receipt.transaction_hash.to_hex_string(), HASH);
}

#[tokio::test]
async fn get_transaction_receipt_populates_rich_fields_from_alloy_receipt() {
    let (_server, client) = client_with_result(full_receipt_response()).await;

    let receipt = client
        .get_transaction_receipt(&TransactionHash::new(HASH).unwrap())
        .await
        .unwrap()
        .unwrap();

    assert_eq!(receipt.status, Some(TransactionStatus::Success));
    assert_eq!(receipt.block_number, Some(1234));
    assert_eq!(receipt.gas_used, Some(Amount::from(21_000u64)));
    assert_eq!(receipt.block_hash.unwrap().to_hex_string(), HASH);
    assert_eq!(receipt.from.unwrap().to_hex_string(), ADDRESS);
    assert_eq!(receipt.to.unwrap().to_hex_string(), ADDRESS);
}

#[tokio::test]
async fn get_storage_at_delegates_to_inner_provider() {
    let word = format!("0x{:0>64}", "2a");
    let (_server, client) = client_with_result(json!(word)).await;

    assert_eq!(
        client
            .get_storage_at(&Address::new(ADDRESS).unwrap(), "0x0")
            .await
            .unwrap(),
        HexData::new(format!("0x{:0>64}", "2a")).unwrap()
    );
}

#[tokio::test]
async fn call_delegates_to_inner_provider() {
    let (_server, client) = client_with_result(json!("0x1234")).await;
    let tx = TransactionRequest::new(
        Some(Address::new(ADDRESS).unwrap()),
        Some(HexData::new("0xabcdef").unwrap()),
        Some(Amount::from(1u32)),
        Some(Amount::from(21_000u32)),
    );

    assert_eq!(client.call(&tx).await.unwrap().to_hex_string(), "0x1234");
}

#[tokio::test]
async fn read_contract_delegates_through_inner_provider() {
    let encoded = format!("0x{:0>64}", "2a");
    let (_server, client) = client_with_result(json!(encoded)).await;
    let request = ContractCall::new(
        Address::new(ADDRESS).unwrap(),
        "balanceOf".to_owned(),
        r#"[{"type":"function","name":"balanceOf","inputs":[{"name":"owner","type":"address"}],"outputs":[{"name":"","type":"uint256"}],"stateMutability":"view"}]"#.to_owned(),
        format!(r#"["{ADDRESS}"]"#),
    );

    assert_eq!(client.read_contract(&request).await.unwrap(), r#""42""#);
}

#[tokio::test]
async fn get_block_delegates_to_inner_provider() {
    let (_server, client) = client_with_result(block_response("0x2a")).await;

    let block = client.get_block("latest").await.unwrap();

    assert_eq!(block.number, 42);
    assert_eq!(block.hash.unwrap().to_hex_string(), HASH);
}

#[tokio::test]
async fn get_contract_returns_handle_without_rpc() {
    let server = MockServer::start().await;
    let client = client_for(&server).await;
    let address = Address::new(ADDRESS).unwrap();

    let handle = client.get_contract(&address, "[]").await.unwrap();

    assert_eq!(handle.address, address);
    assert_eq!(handle.abi_json, "[]");
    assert!(server.received_requests().await.unwrap().is_empty());
}

async fn client_with_result(result: Value) -> (MockServer, AlloyClient) {
    client_with_response(json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": result,
    }))
    .await
}

async fn client_with_response(response: Value) -> (MockServer, AlloyClient) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .mount(&server)
        .await;
    let client = client_for(&server).await;
    (server, client)
}

async fn client_for(server: &MockServer) -> AlloyClient {
    AlloyClient::builder()
        .http(server.uri())
        .unwrap()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
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
        "gasUsed": "0x5208",
        "effectiveGasPrice": "0x1",
        "cumulativeGasUsed": "0x5208",
        "logsBloom": format!("0x{}", "00".repeat(256)),
        "status": "0x1",
        "logs": [],
        "type": "0x2"
    })
}
