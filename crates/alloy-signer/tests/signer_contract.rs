#![cfg(not(target_arch = "wasm32"))]

mod common;

use common::sample_order;
use cow_sdk_alloy_signer::{LocalAlloyKeystoreSigner, SignerError};
use cow_sdk_core::{Signer, SupportedChainId, TransactionBroadcast, TransactionRequest};
use cow_sdk_signing::{ORDER_PRIMARY_TYPE, order_typed_data_payload};
use cow_sdk_test_utils::consts::{
    ANVIL_KEY_1 as TEST_KEY, EXPECTED_LEGACY_FLAT_SIGNATURE, EXPECTED_ORDER_SIGNATURE,
};
use cow_sdk_test_utils::eip712::assert_recovery_byte_is_legacy;

const EXPECTED_ADDRESS: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";
const EXPECTED_MESSAGE_SIGNATURE: &str = "0x267c1300572586cc72a2780636139a843ce20866dcc515c62c02909f0bbf3ce71468a683b857347aced6470cd911828201eb0fe21e2ba3bcf14f903916407d101b";

#[tokio::test]
async fn get_address_matches_known_vector() {
    let signer = test_signer();

    assert_eq!(
        signer.address().await.unwrap().to_hex_string(),
        EXPECTED_ADDRESS
    );
}

#[tokio::test]
async fn sign_message_against_eip191_vector() {
    let signer = test_signer();
    let signature = signer.sign_message(b"hello cow").await.unwrap();

    assert_eq!(signature, EXPECTED_MESSAGE_SIGNATURE);
    assert_recovery_byte_is_legacy(&signature);
}

#[tokio::test]
async fn sign_typed_data_payload_against_cow_order_vector() {
    let signer = test_signer();
    let payload = order_payload();

    assert_eq!(payload.primary_type, ORDER_PRIMARY_TYPE);
    let signature = signer.sign_typed_data_payload(&payload).await.unwrap();

    assert_eq!(signature, EXPECTED_ORDER_SIGNATURE);
    assert_recovery_byte_is_legacy(&signature);
}

#[tokio::test]
async fn sign_typed_data_payload_preserves_order_primary_type() {
    let signer = test_signer();
    let payload = order_payload();
    let canonical = signer.sign_typed_data_payload(&payload).await.unwrap();
    let legacy = signer
        .sign_typed_data(
            &payload.domain,
            payload.primary_type_fields().unwrap(),
            payload.message_json(),
        )
        .await
        .unwrap();

    assert_eq!(canonical, EXPECTED_ORDER_SIGNATURE);
    assert_eq!(legacy, EXPECTED_LEGACY_FLAT_SIGNATURE);
    assert_ne!(
        canonical, legacy,
        "payload signing must not delegate to the flat-fields fallback"
    );
}

#[tokio::test]
async fn sign_typed_data_flat_uses_message_placeholder_primary_type() {
    let signer = test_signer();
    let payload = order_payload();
    let legacy = signer
        .sign_typed_data(
            &payload.domain,
            payload.primary_type_fields().unwrap(),
            payload.message_json(),
        )
        .await
        .unwrap();

    assert_eq!(legacy, EXPECTED_LEGACY_FLAT_SIGNATURE);
    assert_ne!(legacy, EXPECTED_ORDER_SIGNATURE);
}

#[tokio::test]
async fn transaction_methods_return_provider_required() {
    let signer = test_signer();
    let tx = TransactionRequest::default();

    assert!(matches!(
        signer.sign_transaction(&tx).await,
        Err(SignerError::ProviderRequired {
            method: "sign_transaction"
        })
    ));
    assert!(matches!(
        send_transaction_signature_is_broadcast(&signer, &tx).await,
        Err(SignerError::ProviderRequired {
            method: "send_transaction"
        })
    ));
    assert!(matches!(
        signer.estimate_gas(&tx).await,
        Err(SignerError::ProviderRequired {
            method: "estimate_gas"
        })
    ));
}

fn send_transaction_signature_is_broadcast<'a>(
    signer: &'a LocalAlloyKeystoreSigner,
    tx: &'a TransactionRequest,
) -> impl std::future::Future<Output = Result<TransactionBroadcast, SignerError>> + 'a {
    signer.send_transaction(tx)
}

fn test_signer() -> LocalAlloyKeystoreSigner {
    LocalAlloyKeystoreSigner::builder()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .unwrap()
}

fn order_payload() -> cow_sdk_core::TypedDataPayload {
    order_typed_data_payload(SupportedChainId::Mainnet, &sample_order(), None).unwrap()
}
