#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy_signer::{AsyncSignerError, LocalAlloyKeystoreSigner};
use cow_sdk_core::{
    Address, Amount, AppDataHash, AsyncSigner, BuyTokenDestination, OrderKind, SellTokenSource,
    SupportedChainId, TransactionRequest, UnsignedOrder,
};
use cow_sdk_signing::{ORDER_PRIMARY_TYPE, order_typed_data_payload};

const TEST_KEY: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const EXPECTED_ADDRESS: &str = "0x70997970c51812dc3a010c7d01b50e0d17dc79c8";
const EXPECTED_MESSAGE_SIGNATURE: &str = "0x267c1300572586cc72a2780636139a843ce20866dcc515c62c02909f0bbf3ce71468a683b857347aced6470cd911828201eb0fe21e2ba3bcf14f903916407d101b";
const EXPECTED_ORDER_SIGNATURE: &str = "0x34bc8d9249f7f9399d1db57b96bfc3a2f935a25965fe265292142c305284c7241daf1b3049bc75da81012cf33aeac1de09ec5684bccf03afe7274262703780d01c";
const EXPECTED_LEGACY_FLAT_SIGNATURE: &str = "0x474712d3145a910482c333721c46cb800d7985628701af5954134a92e5fb60263233eb36ba80ae8f77600b1d820df4101c4bfca86ea89f6b7a774c31a47ec28a1c";

#[tokio::test]
async fn get_address_matches_known_vector() {
    let signer = test_signer();

    assert_eq!(
        signer.get_address().await.unwrap().as_str(),
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
        Err(AsyncSignerError::ProviderRequired {
            method: "sign_transaction"
        })
    ));
    assert!(matches!(
        signer.send_transaction(&tx).await,
        Err(AsyncSignerError::ProviderRequired {
            method: "send_transaction"
        })
    ));
    assert!(matches!(
        signer.estimate_gas(&tx).await,
        Err(AsyncSignerError::ProviderRequired {
            method: "estimate_gas"
        })
    ));
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
    let bytes = hex::decode(signature.trim_start_matches("0x")).unwrap();
    assert_eq!(bytes.len(), 65);
    assert!(
        matches!(bytes[64], 27 | 28),
        "signature recovery byte must be normalized to legacy form"
    );
}
