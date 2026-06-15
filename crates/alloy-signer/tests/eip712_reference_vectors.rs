#![cfg(not(target_arch = "wasm32"))]

mod common;

use common::{order_digest, sample_order};
use cow_sdk_alloy_signer::LocalAlloySigner;
use cow_sdk_contracts::SigningScheme;
use cow_sdk_core::{Signer, SupportedChainId};
use cow_sdk_signing::{ORDER_PRIMARY_TYPE, order_typed_data_payload};
use cow_sdk_test_utils::consts::{ANVIL_KEY_1 as TEST_KEY, EXPECTED_ORDER_SIGNATURE};

#[tokio::test]
async fn sign_typed_data_payload_matches_canonical_cow_order_vector() {
    let signer = LocalAlloySigner::builder()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .unwrap();
    let order = sample_order();
    let payload = order_typed_data_payload(SupportedChainId::Mainnet, &order, None).unwrap();

    assert_eq!(payload.primary_type, ORDER_PRIMARY_TYPE);

    let signature = signer.sign_typed_data_payload(&payload).await.unwrap();
    let digest = order_digest(&order);
    let recovered = cow_sdk_contracts::Signature::Ecdsa {
        scheme: SigningScheme::Eip712,
        data: signature.clone(),
    }
    .recover_ecdsa_address(&digest)
    .unwrap();

    assert_eq!(signature, EXPECTED_ORDER_SIGNATURE);
    assert_eq!(
        recovered,
        signer.address().await.unwrap(),
        "canonical EIP-712 signature must recover to the local signer"
    );
}
