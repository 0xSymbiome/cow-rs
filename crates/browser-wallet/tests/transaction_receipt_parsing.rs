#![cfg(not(target_arch = "wasm32"))]

//! Asserts the rich-receipt parsing path.

use cow_sdk_browser_wallet::{
    BrowserWallet, BrowserWalletError, Eip1193Provider, MockEip1193Transport,
};
use cow_sdk_core::{Provider, TransactionHash, TransactionStatus};
use serde_json::{Value, json};

const HASH_1: &str = "0x0000000000000000000000000000000000000000000000000000000000000001";
const HASH_3: &str = "0x0000000000000000000000000000000000000000000000000000000000000003";
const HASH_5: &str = "0x0000000000000000000000000000000000000000000000000000000000000005";
const HASH_7: &str = "0x0000000000000000000000000000000000000000000000000000000000000007";
const HASH_8: &str = "0x0000000000000000000000000000000000000000000000000000000000000008";
const HASH_9: &str = "0x0000000000000000000000000000000000000000000000000000000000000009";
const HASH_A: &str = "0x000000000000000000000000000000000000000000000000000000000000000a";
const HASH_B: &str = "0x000000000000000000000000000000000000000000000000000000000000000b";
const HASH_C: &str = "0x000000000000000000000000000000000000000000000000000000000000000c";
const HASH_D: &str = "0x000000000000000000000000000000000000000000000000000000000000000d";
const HASH_E: &str = "0x000000000000000000000000000000000000000000000000000000000000000e";
const HASH_F: &str = "0x000000000000000000000000000000000000000000000000000000000000000f";
const BLOCK_HASH_2: &str = "0x0000000000000000000000000000000000000000000000000000000000000002";
const BLOCK_HASH_4: &str = "0x0000000000000000000000000000000000000000000000000000000000000004";
const BLOCK_HASH_6: &str = "0x0000000000000000000000000000000000000000000000000000000000000006";
const FROM_ADDR: &str = "0xf39fd6e51aad88f6f4ce6ab8827279cfffb92266";
const TO_ADDR: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_populates_every_field_from_post_byzantium_response() {
    let provider = provider_with_receipt(
        HASH_1,
        json!({
            "transactionHash": HASH_1,
            "status": "0x1",
            "blockNumber": "0x4d2",
            "blockHash": BLOCK_HASH_2,
            "gasUsed": "0x5208",
            "from": FROM_ADDR,
            "to": TO_ADDR,
        }),
    )
    .await;

    let receipt = provider
        .get_transaction_receipt(&transaction_hash(HASH_1))
        .await
        .unwrap()
        .expect("receipt available");

    assert_eq!(receipt.transaction_hash.to_hex_string(), HASH_1);
    assert_eq!(receipt.status, Some(TransactionStatus::Success));
    assert_eq!(receipt.block_number, Some(1234));
    assert_eq!(receipt.block_hash.unwrap().to_hex_string(), BLOCK_HASH_2);
    assert_eq!(receipt.gas_used.unwrap().to_string(), "21000");
    assert_eq!(receipt.from.unwrap().to_hex_string(), FROM_ADDR);
    assert_eq!(receipt.to.unwrap().to_hex_string(), TO_ADDR);
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_handles_pre_byzantium_absent_status() {
    let provider = provider_with_receipt(
        HASH_3,
        json!({
            "transactionHash": HASH_3,
            "blockNumber": "0x1",
            "blockHash": BLOCK_HASH_4,
        }),
    )
    .await;

    let receipt = provider
        .get_transaction_receipt(&transaction_hash(HASH_3))
        .await
        .unwrap()
        .expect("receipt available");

    assert_eq!(receipt.status, None);
    assert_eq!(receipt.block_number, Some(1));
    assert_eq!(receipt.block_hash.unwrap().to_hex_string(), BLOCK_HASH_4);
    assert_eq!(receipt.gas_used, None);
    assert_eq!(receipt.from, None);
    assert_eq!(receipt.to, None);
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_handles_contract_creation_no_to() {
    let provider = provider_with_receipt(
        HASH_5,
        json!({
            "transactionHash": HASH_5,
            "status": "0x1",
            "blockNumber": "0x10",
            "blockHash": BLOCK_HASH_6,
            "gasUsed": "0x9c40",
            "from": FROM_ADDR,
            "to": Value::Null,
        }),
    )
    .await;

    let receipt = provider
        .get_transaction_receipt(&transaction_hash(HASH_5))
        .await
        .unwrap()
        .expect("receipt available");

    assert!(receipt.to.is_none());
    assert_eq!(receipt.from.unwrap().to_hex_string(), FROM_ADDR);
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_returns_none_when_chain_has_not_observed_tx() {
    let provider = provider_without_receipt().await;

    let receipt = provider
        .get_transaction_receipt(&transaction_hash(HASH_7))
        .await
        .unwrap();

    assert!(receipt.is_none());
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_rejects_response_without_transaction_hash() {
    let provider = provider_with_receipt(HASH_8, json!({ "status": "0x1" })).await;

    let result = provider
        .get_transaction_receipt(&transaction_hash(HASH_8))
        .await;

    assert!(matches!(
        result,
        Err(BrowserWalletError::MalformedResponse { .. })
    ));
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_returns_reverted_for_status_0x0() {
    let provider = provider_with_receipt(
        HASH_9,
        json!({
            "transactionHash": HASH_9,
            "status": "0x0",
            "blockNumber": "0x1",
            "blockHash": BLOCK_HASH_2,
            "gasUsed": "0x5208",
        }),
    )
    .await;

    let receipt = provider
        .get_transaction_receipt(&transaction_hash(HASH_9))
        .await
        .unwrap()
        .expect("receipt available");

    assert_eq!(receipt.status, Some(TransactionStatus::Reverted));
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_rejects_invalid_status_value() {
    assert_malformed_field(
        HASH_A,
        json!({ "transactionHash": HASH_A, "status": "0x42" }),
        "status",
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_rejects_malformed_block_number() {
    assert_malformed_field(
        HASH_B,
        json!({ "transactionHash": HASH_B, "blockNumber": "not-a-number" }),
        "blockNumber",
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_rejects_malformed_block_hash() {
    assert_malformed_field(
        HASH_C,
        json!({ "transactionHash": HASH_C, "blockHash": "0xnot-hex" }),
        "blockHash",
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_rejects_malformed_gas_used() {
    assert_malformed_field(
        HASH_D,
        json!({ "transactionHash": HASH_D, "gasUsed": "0xZZZ" }),
        "gasUsed",
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_rejects_malformed_from() {
    assert_malformed_field(
        HASH_E,
        json!({ "transactionHash": HASH_E, "from": "0x42" }),
        "from",
    )
    .await;
}

#[tokio::test(flavor = "current_thread")]
async fn parse_receipt_rejects_malformed_to() {
    assert_malformed_field(
        HASH_F,
        json!({ "transactionHash": HASH_F, "to": "garbage" }),
        "to",
    )
    .await;
}

async fn assert_malformed_field(hash: &str, receipt: Value, field: &str) {
    let provider = provider_with_receipt(hash, receipt).await;
    let err = provider
        .get_transaction_receipt(&transaction_hash(hash))
        .await
        .unwrap_err();

    let BrowserWalletError::MalformedResponse { message, .. } = err else {
        panic!("expected malformed response error");
    };
    assert!(message.as_inner().contains(field), "{message:?}");
}

async fn provider_with_receipt(hash: &str, receipt: Value) -> Eip1193Provider {
    let transport = MockEip1193Transport::sepolia();
    transport.set_receipt(hash, receipt);
    let wallet = BrowserWallet::from_transport_or_panic(transport);
    wallet.connect().await.unwrap();
    wallet.provider()
}

async fn provider_without_receipt() -> Eip1193Provider {
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport_or_panic(transport);
    wallet.connect().await.unwrap();
    wallet.provider()
}

fn transaction_hash(value: &str) -> TransactionHash {
    TransactionHash::new(value).unwrap()
}
