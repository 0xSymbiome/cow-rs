#![cfg(not(target_arch = "wasm32"))]

//! Cross-adapter invariant for transaction broadcast timing and rich receipts.

use std::sync::{Arc, Mutex};

use cow_sdk_alloy::AlloyClient;
use cow_sdk_browser_wallet::{BrowserWallet, MockEip1193Transport};
use cow_sdk_core::{
    Address, Amount, Provider, Signer, SigningProvider, SupportedChainId, TransactionHash,
    TransactionRequest, TransactionStatus,
};
use serde_json::{Value, json};
use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

#[path = "support/rpc.rs"]
mod support;
use support::{ADDRESS, HASH, mount_rpc};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const WALLET_ADDRESS: &str = "0x4444444444444444444444444444444444444444";

#[tokio::test]
async fn alloy_send_transaction_does_not_poll_for_receipt() {
    let server = MockServer::start().await;
    let methods = mount_rpc(&server).await;
    let client = alloy_client(&server).await;
    let signer = client.create_signer("local-key").await.unwrap();

    let broadcast = signer
        .send_transaction(&sample_transaction())
        .await
        .unwrap();

    assert_eq!(broadcast.transaction_hash.to_hex_string(), HASH);
    let methods = recorded_methods(&methods);
    assert!(
        methods
            .iter()
            .any(|method| method == "eth_sendRawTransaction")
    );
    assert!(
        !methods
            .iter()
            .any(|method| method == "eth_getTransactionReceipt")
    );
}

#[tokio::test]
async fn browser_wallet_send_transaction_does_not_poll_for_receipt() {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport_or_panic(transport.clone());
    wallet.connect().await.unwrap();
    let signer = wallet.signer();

    let broadcast = signer
        .send_transaction(&sample_transaction())
        .await
        .unwrap();

    assert_eq!(
        broadcast.transaction_hash.to_hex_string(),
        format!("0x{}", "33".repeat(32))
    );
    let methods = transport
        .request_log()
        .into_iter()
        .map(|record| record.method)
        .collect::<Vec<String>>();
    assert!(methods.iter().any(|method| method == "eth_sendTransaction"));
    assert!(
        !methods
            .iter()
            .any(|method| method == "eth_getTransactionReceipt")
    );
}

#[tokio::test]
async fn alloy_get_transaction_receipt_populates_status_and_block() {
    let (server, client) = alloy_client_with_result(receipt_response()).await;

    let receipt = client
        .get_transaction_receipt(&TransactionHash::new(HASH).unwrap())
        .await
        .unwrap()
        .expect("receipt available");

    assert_rich_receipt(&receipt);
    assert_eq!(server.received_requests().await.unwrap().len(), 1);
}

#[tokio::test]
async fn browser_wallet_get_transaction_receipt_populates_status_and_block() {
    let transport = MockEip1193Transport::sepolia();
    transport.set_receipt(HASH, receipt_response());
    let wallet = BrowserWallet::from_transport_or_panic(transport);
    wallet.connect().await.unwrap();

    let receipt = wallet
        .provider()
        .get_transaction_receipt(&TransactionHash::new(HASH).unwrap())
        .await
        .unwrap()
        .expect("receipt available");

    assert_rich_receipt(&receipt);
}

async fn alloy_client_with_result(result: Value) -> (MockServer, AlloyClient) {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": result,
        })))
        .mount(&server)
        .await;
    let client = alloy_client(&server).await;
    (server, client)
}

async fn alloy_client(server: &MockServer) -> AlloyClient {
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

fn receipt_response() -> Value {
    json!({
        "transactionHash": HASH,
        "transactionIndex": "0x0",
        "blockHash": HASH,
        "blockNumber": "0x4d2",
        "from": WALLET_ADDRESS,
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

fn assert_rich_receipt(receipt: &cow_sdk_core::TransactionReceipt) {
    assert_eq!(receipt.transaction_hash.to_hex_string(), HASH);
    assert_eq!(receipt.status, Some(TransactionStatus::Success));
    assert_eq!(receipt.block_number, Some(1234));
    assert_eq!(receipt.block_hash.as_ref().unwrap().to_hex_string(), HASH);
    assert_eq!(receipt.gas_used, Some(Amount::from(21_000u64)));
    assert_eq!(
        receipt.from.as_ref().unwrap().to_hex_string(),
        WALLET_ADDRESS
    );
    assert_eq!(receipt.to.as_ref().unwrap().to_hex_string(), ADDRESS);
}

fn recorded_methods(methods: &Arc<Mutex<Vec<String>>>) -> Vec<String> {
    methods
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone()
}

fn sample_transaction() -> TransactionRequest {
    TransactionRequest::new(
        Some(Address::new(ADDRESS).unwrap()),
        None,
        Some(Amount::ZERO),
        Some(Amount::from(21_000u32)),
    )
}
