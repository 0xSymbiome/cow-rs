#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{
    OrderData, Signer, SigningProvider, SupportedChainId, TypedDataPayload, TypedDataTypes,
};
use cow_sdk_signing::{ORDER_PRIMARY_TYPE, order_typed_data_payload};
use cow_sdk_test_utils::builders::OrderBuilder;
use cow_sdk_test_utils::consts::{ANVIL_KEY_1 as TEST_KEY, EXPECTED_ORDER_SIGNATURE};
use cow_sdk_test_utils::eip712::assert_recovery_byte_is_legacy;

/// EIP-712 signature over the `upstream_signing` order fields rewrapped under
/// a `Message` placeholder primary type, signed by `ANVIL_KEY_1`. Committed so
/// the primary-type-preservation check pins two distinct digests.
const EXPECTED_MESSAGE_PLACEHOLDER_SIGNATURE: &str = "0x474712d3145a910482c333721c46cb800d7985628701af5954134a92e5fb60263233eb36ba80ae8f77600b1d820df4101c4bfca86ea89f6b7a774c31a47ec28a1c";

#[tokio::test]
async fn sign_typed_data_payload_matches_cow_order_vector() {
    let signer = test_client()
        .await
        .create_signer("local-key")
        .await
        .unwrap();
    let payload = order_payload();

    assert_eq!(payload.primary_type, ORDER_PRIMARY_TYPE);
    let signature = signer.sign_typed_data_payload(&payload).await.unwrap();

    assert_eq!(signature, EXPECTED_ORDER_SIGNATURE);
    assert_recovery_byte_is_legacy(&signature);
}

#[tokio::test]
async fn sign_typed_data_payload_preserves_primary_type() {
    let signer = test_client()
        .await
        .create_signer("local-key")
        .await
        .unwrap();
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

async fn test_client() -> AlloyClient {
    AlloyClient::builder()
        .http("http://127.0.0.1:9")
        .unwrap()
        .private_key(TEST_KEY)
        .unwrap()
        .chain_id(SupportedChainId::Mainnet)
        .build()
        .await
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

fn sample_order() -> OrderData {
    OrderBuilder::upstream_signing().build()
}
