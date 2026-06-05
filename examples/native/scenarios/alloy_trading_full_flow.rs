//! End-to-end Alloy-backed trading boundaries.
//!
//! Drives the async `Trading` boundaries through a real `AlloyClient`
//! (a `SigningProvider`) against a wiremock JSON-RPC server: read the protocol
//! allowance (`get_cow_protocol_allowance`), broadcast an approval and wait for
//! its receipt (`approval_transaction` + `submit_and_wait_for_receipt`), and
//! build a pre-sign transaction (`get_pre_sign_transaction`).

use std::error::Error;

use cow_sdk::alloy::AlloyClient;
use cow_sdk::core::{
    Amount, CowEnv, SigningProvider, SupportedChainId, TransactionHash, TransactionStatus,
};
use cow_sdk::trading::{
    AllowanceParameters, ApprovalParameters, OrderTraderParameters, Trading, WaitOptions,
    approval_transaction, submit_and_wait_for_receipt,
};
use cow_sdk_examples_native::support::{
    COW, OWNER, TEST_KEY, TX_HASH, address, mount_rpc, sample_order_uid,
};
use serde_json::json;
use wiremock::MockServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Wiremock JSON-RPC server; `mount_rpc` records each method it sees so the
    // report can show the exact RPC calls the SDK made.
    let server = MockServer::start().await;
    let methods = mount_rpc(&server).await;
    // build_checked() verifies the configured chain id against the RPC endpoint.
    let client = AlloyClient::builder()
        .http(server.uri())?
        .private_key(TEST_KEY)?
        .chain_id(SupportedChainId::Mainnet)
        .build_checked()
        .await?;
    let signer = client.create_signer("local-key").await?;
    let trading = Trading::builder()
        .chain_id(SupportedChainId::Mainnet)
        .env(CowEnv::Prod)
        .app_code("cow-rs/alloy-trading-example")
        .build()?;

    // 1. Read the protocol allowance for COW held by the owner (an eth_call).
    let allowance = trading
        .get_cow_protocol_allowance(
            &client,
            &AllowanceParameters::new(address(COW), address(OWNER)),
        )
        .await?;
    assert_eq!(allowance, Amount::from(42u32));

    // 2. Build an approval transaction, broadcast it, and wait for the receipt.
    let approval_params = ApprovalParameters::new(address(COW), Amount::new("1000")?);
    let approval_tx =
        approval_transaction(&approval_params, SupportedChainId::Mainnet, CowEnv::Prod)?;
    let approval_receipt = submit_and_wait_for_receipt(
        &signer,
        &client,
        &approval_tx,
        WaitOptions::approve_default(),
    )
    .await?;
    assert_eq!(
        approval_receipt.transaction_hash,
        TransactionHash::new(TX_HASH)?
    );
    assert_eq!(approval_receipt.status, Some(TransactionStatus::Success));

    // 3. Build a pre-sign transaction; gas is estimated through the client.
    let pre_sign = trading
        .get_pre_sign_transaction(&OrderTraderParameters::new(sample_order_uid()), &signer)
        .await?;
    assert_eq!(pre_sign.gas_limit, Some(Amount::from(25_200u32)));

    let methods = methods
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
        .clone();
    let report = json!({
        "surface": "cow-sdk::alloy::AlloyClient with Trading",
        "allowance": allowance,
        "approvalTxHash": approval_receipt.transaction_hash.to_hex_string(),
        "approvalStatus": format!("{:?}", approval_receipt.status),
        "approvalBlockNumber": approval_receipt.block_number,
        "approvalGasUsed": approval_receipt.gas_used,
        "preSignGasLimit": pre_sign.gas_limit,
        "rpcMethods": methods
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
