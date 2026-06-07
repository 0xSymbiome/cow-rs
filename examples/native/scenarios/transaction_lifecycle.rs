//! Demonstrates two transaction lifecycle shapes:
//! (A) submit and wait for one mined receipt through the trading helper, and
//! (B) broadcast once and keep receipt observation separate.

use std::error::Error;

use cow_sdk::alloy::AlloyClient;
use cow_sdk::core::{
    Signer, SigningProvider, TransactionBroadcast, TransactionRequest, TransactionStatus,
};
use cow_sdk::prelude::{Address, Amount, SupportedChainId};
use cow_sdk::trading::{WaitOptions, submit_and_wait_for_receipt};
use cow_sdk_examples_native::support::{TEST_KEY, mount_rpc};
use serde_json::json;
use wiremock::MockServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Wiremock JSON-RPC server; `mount_rpc` records each method so the report can
    // count how many receipt lookups each shape triggers.
    let server = MockServer::start().await;
    let methods = mount_rpc(&server).await;
    let client = AlloyClient::builder()
        .http(server.uri())?
        .private_key(TEST_KEY)?
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await?;
    let signer = client.create_signer("local-key").await?;

    // A simple self-transfer to broadcast in both shapes below.
    let tx = self_transfer(&signer.address().await?);

    // Shape A: one helper call broadcasts once and returns the mined receipt.
    let helper_receipt =
        submit_and_wait_for_receipt(&signer, &client, &tx, WaitOptions::approve_default()).await?;
    assert_eq!(helper_receipt.status, Some(TransactionStatus::Success));

    // Shape B: one manual broadcast, with receipt observation left separate.
    let method_start = {
        methods
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    };
    let broadcast: TransactionBroadcast = signer.send_transaction(&tx).await?;

    let methods = methods
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let shape_b_methods = &methods[method_start..];
    let report = json!({
        "surface": "transaction lifecycle",
        "shapeA": {
            "receiptHash": helper_receipt.transaction_hash.to_hex_string(),
            "status": format!("{:?}", helper_receipt.status),
            "blockNumber": helper_receipt.block_number,
            "gasUsed": helper_receipt.gas_used,
        },
        "shapeB": {
            "broadcastHash": broadcast.transaction_hash.to_hex_string(),
            "receiptRequestsDuringBroadcast": shape_b_methods
                .iter()
                .filter(|method| method.as_str() == "eth_getTransactionReceipt")
                .count()
        },
        "totalBroadcasts": methods
            .iter()
            .filter(|method| method.as_str() == "eth_sendRawTransaction")
            .count()
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn self_transfer(address: &Address) -> TransactionRequest {
    TransactionRequest::new(
        Some(*address),
        None,
        Some(Amount::ZERO),
        Some(Amount::from(21_000u32)),
    )
}
