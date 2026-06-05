#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{OrderData, Signer, SigningProvider, SupportedChainId};
use cow_sdk_signing::{ORDER_PRIMARY_TYPE, order_typed_data_payload};
use cow_sdk_test_utils::builders::OrderBuilder;
use cow_sdk_test_utils::consts::{
    ANVIL_KEY_1 as TEST_KEY, EXPECTED_LEGACY_FLAT_SIGNATURE, EXPECTED_ORDER_SIGNATURE,
};
use cow_sdk_test_utils::eip712::assert_recovery_byte_is_legacy;

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

fn order_payload() -> cow_sdk_core::TypedDataPayload {
    order_typed_data_payload(SupportedChainId::Mainnet, &sample_order(), None).unwrap()
}

fn sample_order() -> OrderData {
    OrderBuilder::upstream_signing().build()
}
