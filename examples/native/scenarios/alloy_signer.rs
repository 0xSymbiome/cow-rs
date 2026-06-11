//! Alloy signer leaf in isolation: EIP-712 order signing.
//!
//! Uses `LocalAlloySigner` (the signer leaf, no provider) to sign a real
//! `CoW` order's EIP-712 typed-data payload (`order_typed_data_payload`,
//! `ORDER_PRIMARY_TYPE`) entirely in memory — no RPC — and checks the recovery
//! byte normalizes to the legacy 27/28 range.

use std::error::Error;

use cow_sdk::alloy_signer::LocalAlloySigner;
use cow_sdk::core::{
    Address, Amount, AppDataHash, BuyTokenDestination, HexData, OrderData, OrderKind,
    SellTokenSource, Signer, SupportedChainId, address,
};
use cow_sdk::signing::{ORDER_PRIMARY_TYPE, order_typed_data_payload};
use serde_json::json;

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

const SELL_TOKEN: Address = address!("0xd057b63f5e69cf1b929b356b579cba08d7688048");
const BUY_TOKEN: Address = address!("0x7b878668cd1a3adf89764d3a331e0a7bb832192d");
const RECEIVER: Address = address!("0xa6ddbd0de6b310819b49f680f65871bee85f517e");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Build the signer leaf from a private key — entirely in memory, no provider.
    let signer = LocalAlloySigner::builder()
        .private_key(TEST_KEY)?
        .chain_id(SupportedChainId::Mainnet)
        .build()?;

    // Build the EIP-712 typed-data payload for a CoW order and confirm its
    // primary type matches the protocol constant.
    let payload = order_typed_data_payload(SupportedChainId::Mainnet, &sample_order()?, None)?;
    assert_eq!(payload.primary_type, ORDER_PRIMARY_TYPE);

    // Sign it; the SDK normalizes the recovery byte to the legacy 27/28 range.
    let signature = signer.sign_typed_data_payload(&payload).await?;
    assert_recovery_byte_is_legacy(&signature)?;

    let report = json!({
        "surface": "cow_sdk::alloy_signer::LocalAlloySigner",
        "signer": signer.address().await?.to_hex_string(),
        "primaryType": payload.primary_type,
        "signatureBytes": (signature.len() - 2) / 2
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn sample_order() -> Result<OrderData, Box<dyn Error>> {
    Ok(OrderData {
        sell_token: SELL_TOKEN,
        buy_token: BUY_TOKEN,
        receiver: RECEIVER,
        sell_amount: Amount::new("500000000000000")?,
        buy_amount: Amount::new("23000020000")?,
        valid_to: 5_000_222,
        app_data: AppDataHash::new(
            "0x0000000000000000000000000000000000000000000000000000000000000000",
        )?,
        fee_amount: Amount::new("2300000")?,
        kind: OrderKind::Sell,
        partially_fillable: true,
        sell_token_balance: SellTokenSource::Erc20,
        buy_token_balance: BuyTokenDestination::Erc20,
    })
}

fn assert_recovery_byte_is_legacy(signature: &str) -> Result<(), Box<dyn Error>> {
    let bytes = HexData::new(signature)?;
    assert_eq!(bytes.byte_length(), 65);
    assert!(matches!(bytes.as_slice()[64], 27 | 28));
    Ok(())
}
