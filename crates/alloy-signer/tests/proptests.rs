#![cfg(not(target_arch = "wasm32"))]

mod common;

use std::str::FromStr;

use alloy_primitives::Signature as AlloySignature;
use common::{order_digest, sample_order};
use cow_sdk_alloy_signer::LocalAlloyKeystoreSigner;
use cow_sdk_contracts::SigningScheme;
use cow_sdk_core::{Address, Signer, SupportedChainId};
use cow_sdk_signing::order_typed_data_payload;
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

        let address = runtime.block_on(signer.address()).unwrap();
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

        let address = runtime.block_on(signer.address()).unwrap();
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
            runtime.block_on(first.address()).unwrap(),
            runtime.block_on(second.address()).unwrap(),
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

fn prop_assert_legacy_recovery_byte(
    signature: &str,
) -> Result<(), proptest::test_runner::TestCaseError> {
    let bytes = alloy_primitives::hex::decode(signature.trim_start_matches("0x")).unwrap();
    prop_assert_eq!(bytes.len(), 65);
    prop_assert!(matches!(bytes[64], 27 | 28));
    Ok(())
}
