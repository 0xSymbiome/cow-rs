#![cfg(not(target_arch = "wasm32"))]

mod common;

use common::sample_order;
use cow_sdk_alloy_signer::{LocalAlloySigner, SignerError};
use cow_sdk_core::{
    Signer, SupportedChainId, TransactionBroadcast, TransactionRequest, TypedDataPayload,
    TypedDataTypes,
};
use cow_sdk_signing::{ORDER_PRIMARY_TYPE, order_typed_data_payload};
use cow_sdk_test_utils::consts::{ANVIL_KEY_1 as TEST_KEY, EXPECTED_ORDER_SIGNATURE};
use cow_sdk_test_utils::eip712::assert_recovery_byte_is_legacy;

const EXPECTED_ADDRESS: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";
/// EIP-712 signature over the `upstream_signing` order fields rewrapped under
/// a `Message` placeholder primary type, signed by `ANVIL_KEY_1`. Committed so
/// the primary-type-preservation check pins two distinct digests.
const EXPECTED_MESSAGE_PLACEHOLDER_SIGNATURE: &str = "0x474712d3145a910482c333721c46cb800d7985628701af5954134a92e5fb60263233eb36ba80ae8f77600b1d820df4101c4bfca86ea89f6b7a774c31a47ec28a1c";
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
    let placeholder = signer
        .sign_typed_data_payload(&message_placeholder_payload(&payload))
        .await
        .unwrap();

    assert_eq!(canonical, EXPECTED_ORDER_SIGNATURE);
    assert_eq!(placeholder, EXPECTED_MESSAGE_PLACEHOLDER_SIGNATURE);
    assert_ne!(
        canonical, placeholder,
        "payload signing must preserve the order primary type instead of a placeholder digest"
    );
}

#[tokio::test]
async fn transaction_methods_return_provider_required() {
    let signer = test_signer();
    let tx = TransactionRequest::default();

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
    signer: &'a LocalAlloySigner,
    tx: &'a TransactionRequest,
) -> impl std::future::Future<Output = Result<TransactionBroadcast, SignerError>> + 'a {
    signer.send_transaction(tx)
}

fn test_signer() -> LocalAlloySigner {
    LocalAlloySigner::builder()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .unwrap()
}

fn order_payload() -> TypedDataPayload {
    order_typed_data_payload(SupportedChainId::Mainnet, &sample_order(), None).unwrap()
}

/// Rewraps the order payload's fields and message under a `Message`
/// placeholder primary type. Signing the rewrapped payload reproduces the
/// digest the retired flat `(domain, fields, value)` form produced, so the
/// committed vectors pin that canonical payload signing keeps the caller's
/// primary type instead of collapsing to a placeholder.
fn message_placeholder_payload(payload: &TypedDataPayload) -> TypedDataPayload {
    let mut types = TypedDataTypes::new();
    types.insert(
        "Message".to_owned(),
        payload.primary_type_fields().unwrap().to_vec(),
    );
    TypedDataPayload::new(
        payload.domain.clone(),
        "Message".to_owned(),
        types,
        payload.message_json().to_owned(),
    )
}
