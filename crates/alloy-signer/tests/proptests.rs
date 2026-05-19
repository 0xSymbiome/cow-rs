#![cfg(not(target_arch = "wasm32"))]

use std::str::FromStr;

use alloy_primitives::Signature as AlloySignature;
use cow_sdk_alloy_signer::LocalAlloyKeystoreSigner;
use cow_sdk_contracts::{Order as ContractsOrder, SigningScheme, hash_order};
use cow_sdk_core::{
    Address, Amount, AppDataHash, AsyncSigner, BuyTokenDestination, Hash32, OrderKind,
    SellTokenSource, SupportedChainId, UnsignedOrder,
};
use cow_sdk_signing::{get_domain, order_typed_data_payload};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(48))]

    #[test]
    fn eip191_signatures_recover_to_generated_signer(
        key in any::<[u8; 32]>(),
        message in proptest::collection::vec(any::<u8>(), 0..128),
    ) {
        let Some(signer) = signer_from_key(key) else {
            return Ok(());
        };
        let runtime = runtime();

        let address = runtime.block_on(signer.get_address()).unwrap();
        let signature = runtime.block_on(signer.sign_message(&message)).unwrap();
        let recovered = AlloySignature::from_str(&signature)
            .unwrap()
            .recover_address_from_msg(&message)
            .unwrap();

        prop_assert_eq!(Address::from_bytes(recovered.into_array()), address);
        prop_assert_legacy_recovery_byte(&signature)?;
    }

    #[test]
    fn eip712_order_signatures_recover_to_generated_signer(key in any::<[u8; 32]>()) {
        let Some(signer) = signer_from_key(key) else {
            return Ok(());
        };
        let runtime = runtime();
        let order = sample_order();
        let payload = order_typed_data_payload(SupportedChainId::Mainnet, &order, None).unwrap();
        let digest = order_digest(&order);

        let address = runtime.block_on(signer.get_address()).unwrap();
        let signature = runtime.block_on(signer.sign_typed_data_payload(&payload)).unwrap();
        let recovered = cow_sdk_contracts::Signature::Ecdsa {
            scheme: SigningScheme::Eip712,
            data: signature.clone(),
        }
        .recover_ecdsa_address(&digest)
        .unwrap();

        prop_assert_eq!(recovered, address);
        prop_assert_legacy_recovery_byte(&signature)?;
    }

    #[test]
    fn signer_address_is_deterministic_for_valid_key(key in any::<[u8; 32]>()) {
        let Some(first) = signer_from_key(key) else {
            return Ok(());
        };
        let Some(second) = signer_from_key(key) else {
            return Ok(());
        };
        let runtime = runtime();

        prop_assert_eq!(
            runtime.block_on(first.get_address()).unwrap(),
            runtime.block_on(second.get_address()).unwrap(),
        );
    }
}

fn signer_from_key(bytes: [u8; 32]) -> Option<LocalAlloyKeystoreSigner> {
    LocalAlloyKeystoreSigner::builder()
        .private_key_bytes(bytes)
        .ok()?
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .ok()
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn order_digest(order: &UnsignedOrder) -> Hash32 {
    hash_order(
        &get_domain(SupportedChainId::Mainnet, None).unwrap(),
        &ContractsOrder::new(
            order.sell_token,
            order.buy_token,
            Some(order.receiver),
            order.sell_amount.clone(),
            order.buy_amount.clone(),
            order.valid_to,
            order.app_data.clone(),
            order.fee_amount.clone(),
            order.kind,
            order.partially_fillable,
            Some(order.sell_token_balance),
            Some(order.buy_token_balance),
        ),
    )
    .unwrap()
}

fn sample_order() -> UnsignedOrder {
    UnsignedOrder::new(
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

fn prop_assert_legacy_recovery_byte(
    signature: &str,
) -> Result<(), proptest::test_runner::TestCaseError> {
    let bytes = hex::decode(signature.trim_start_matches("0x")).unwrap();
    prop_assert_eq!(bytes.len(), 65);
    prop_assert!(matches!(bytes[64], 27 | 28));
    Ok(())
}
