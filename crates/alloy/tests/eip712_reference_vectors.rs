#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::AlloyClient;
use cow_sdk_core::{
    Address, Amount, AppDataHash, AsyncSigner, AsyncSigningProvider, BuyTokenDestination,
    OrderKind, SellTokenSource, SupportedChainId, UnsignedOrder,
};
use cow_sdk_signing::{ORDER_PRIMARY_TYPE, order_typed_data_payload};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const EXPECTED_ORDER_SIGNATURE: &str = "0x34bc8d9249f7f9399d1db57b96bfc3a2f935a25965fe265292142c305284c7241daf1b3049bc75da81012cf33aeac1de09ec5684bccf03afe7274262703780d01c";
const EXPECTED_LEGACY_FLAT_SIGNATURE: &str = "0x474712d3145a910482c333721c46cb800d7985628701af5954134a92e5fb60263233eb36ba80ae8f77600b1d820df4101c4bfca86ea89f6b7a774c31a47ec28a1c";

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

fn assert_recovery_byte_is_legacy(signature: &str) {
    let bytes = alloy_primitives::hex::decode(signature.trim_start_matches("0x")).unwrap();
    assert_eq!(bytes.len(), 65);
    assert!(
        matches!(bytes[64], 27 | 28),
        "signature recovery byte must be normalized to legacy form"
    );
}
