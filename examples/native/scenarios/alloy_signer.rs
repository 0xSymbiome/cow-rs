//! Alloy signer leaf in isolation: EIP-712 order signing.
//!
//! Uses `LocalAlloyKeystoreSigner` (the signer leaf, no provider) to sign a real
//! CoW order's EIP-712 typed-data payload (`order_typed_data_payload`,
//! `ORDER_PRIMARY_TYPE`) entirely in memory — no RPC — and checks the recovery
//! byte normalizes to the legacy 27/28 range.

use std::error::Error;

use cow_sdk::alloy_signer::LocalAlloyKeystoreSigner;
use cow_sdk::core::{
    Address, Amount, AppDataHash, BuyTokenDestination, OrderData, OrderKind, SellTokenSource,
    Signer, SupportedChainId,
};
use cow_sdk::signing::{ORDER_PRIMARY_TYPE, order_typed_data_payload};
use serde_json::json;

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Build the signer leaf from a private key — entirely in memory, no provider.
    let signer = LocalAlloyKeystoreSigner::builder()
        .private_key(TEST_KEY)?
        .chain_id(SupportedChainId::Mainnet)
        .build()?;

    // Build the EIP-712 typed-data payload for a CoW order and confirm its
    // primary type matches the protocol constant.
    let payload = order_typed_data_payload(SupportedChainId::Mainnet, &sample_order(), None)?;
    assert_eq!(payload.primary_type, ORDER_PRIMARY_TYPE);

    // Sign it; the SDK normalizes the recovery byte to the legacy 27/28 range.
    let signature = signer.sign_typed_data_payload(&payload).await?;
    assert_recovery_byte_is_legacy(&signature);

    let report = json!({
        "surface": "cow-sdk::alloy_signer::LocalAlloyKeystoreSigner",
        "signer": signer.address().await?.to_hex_string(),
        "primaryType": payload.primary_type,
        "signatureBytes": (signature.len() - 2) / 2
    });
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn sample_order() -> OrderData {
    OrderData::new(
        Address::new("0xd057b63f5e69cf1b929b356b579cba08d7688048").unwrap(),
        Address::new("0x7b878668cd1a3adf89764d3a331e0a7bb832192d").unwrap(),
        Address::new("0xa6ddbd0de6b310819b49f680f65871bee85f517e").unwrap(),
        Amount::new("500000000000000").unwrap(),
        Amount::new("23000020000").unwrap(),
        5_000_222,
        AppDataHash::new("0x0000000000000000000000000000000000000000000000000000000000000000")
            .unwrap(),
        Amount::new("2300000").unwrap(),
        OrderKind::Sell,
        true,
        SellTokenSource::Erc20,
        BuyTokenDestination::Erc20,
    )
}

fn assert_recovery_byte_is_legacy(signature: &str) {
    let bytes = alloy_primitives::hex::decode(signature.trim_start_matches("0x")).unwrap();
    assert_eq!(bytes.len(), 65);
    assert!(matches!(bytes[64], 27 | 28));
}
