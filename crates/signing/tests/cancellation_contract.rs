#![cfg(not(target_arch = "wasm32"))]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, and perf lints acceptable in test helper code"
)]

mod common;

use cow_sdk_contracts::{OrderCancellations, SigningScheme, hash_order_cancellations};
use cow_sdk_core::SupportedChainId;
use cow_sdk_signing::{
    ORDER_CANCELLATIONS_PRIMARY_TYPE, SigningError, get_domain,
    order_cancellations_typed_data_payload, sign_order_cancellation,
    sign_order_cancellation_with_scheme, sign_order_cancellations,
    sign_order_cancellations_with_scheme,
};
use cow_sdk_test_utils::mocks::RecordingSigner;

use common::sample_order_uid;

#[tokio::test]
async fn single_and_batch_cancellation_signing_are_first_class() {
    let signer = RecordingSigner::new();
    let order_uid = sample_order_uid();
    let batch_uids = vec![
        order_uid,
        cow_sdk_core::OrderUid::new(
            "0x1aaa7dddecccc04cc101a121e3eed017eab4d3927c045d407d5ad6700eea2bf7fb3c7eb936caa12b5a884d612393969a557d430764060343",
        )
        .unwrap(),
    ];

    let single = sign_order_cancellation(&order_uid, SupportedChainId::Sepolia, &signer, None)
        .await
        .unwrap();
    assert_eq!(single.signing_scheme, SigningScheme::Eip712);

    let batch = sign_order_cancellations_with_scheme(
        &batch_uids,
        SupportedChainId::Sepolia,
        &signer,
        SigningScheme::EthSign,
        None,
    )
    .await
    .unwrap();
    assert_eq!(batch.signing_scheme, SigningScheme::EthSign);
}

#[tokio::test]
async fn cancellation_signing_uses_typed_data_and_ethsign_digest_paths() {
    let signer = RecordingSigner::new();
    let order_uid = sample_order_uid();
    let payload = order_cancellations_typed_data_payload(
        std::slice::from_ref(&order_uid),
        SupportedChainId::Sepolia,
        None,
    )
    .unwrap();

    assert_eq!(payload.primary_type, ORDER_CANCELLATIONS_PRIMARY_TYPE);
    assert_eq!(payload.types["OrderCancellations"][0].kind, "bytes[]");
    let order_uid_hex = order_uid.to_hex_string();
    assert!(payload.message.contains(&order_uid_hex));

    sign_order_cancellation(&order_uid, SupportedChainId::Sepolia, &signer, None)
        .await
        .unwrap();
    assert_eq!(signer.calls.borrow().typed_data.len(), 1);
    assert_eq!(
        signer.calls.borrow().typed_data[0].fields[0].kind,
        "bytes[]"
    );
    assert!(
        signer.calls.borrow().typed_data[0]
            .value_json
            .contains(&order_uid_hex)
    );

    let batch_uids = vec![order_uid];
    sign_order_cancellations_with_scheme(
        &batch_uids,
        SupportedChainId::Sepolia,
        &signer,
        SigningScheme::EthSign,
        None,
    )
    .await
    .unwrap();

    let expected_digest = hash_order_cancellations(
        &get_domain(SupportedChainId::Sepolia, None).unwrap(),
        &OrderCancellations::new(batch_uids),
    )
    .unwrap();
    assert_eq!(
        format!(
            "0x{}",
            alloy_primitives::hex::encode(&signer.calls.borrow().messages[0])
        ),
        expected_digest.to_hex_string()
    );
}

#[tokio::test]
async fn unsupported_cancellation_modes_fail_with_typed_error() {
    let signer = RecordingSigner::new();
    let order_uid = sample_order_uid();

    let error = sign_order_cancellation_with_scheme(
        &order_uid,
        SupportedChainId::Mainnet,
        &signer,
        SigningScheme::PreSign,
        None,
    )
    .await
    .unwrap_err();

    assert!(matches!(
        error,
        SigningError::UnsupportedSignerGeneratedScheme {
            scheme: SigningScheme::PreSign
        }
    ));
}

#[tokio::test]
async fn batch_cancellation_signing_routes_to_typed_data_for_default_scheme() {
    let signer = RecordingSigner::new();
    let order_uid = sample_order_uid();

    let single = sign_order_cancellation(&order_uid, SupportedChainId::Sepolia, &signer, None)
        .await
        .unwrap();
    assert_eq!(single.signing_scheme, SigningScheme::Eip712);

    let batch = sign_order_cancellations(
        std::slice::from_ref(&order_uid),
        SupportedChainId::Sepolia,
        &signer,
        None,
    )
    .await
    .unwrap();
    assert_eq!(batch.signing_scheme, SigningScheme::Eip712);
}

#[cfg(feature = "tracing")]
mod tracing_contract {
    use super::*;
    use tracing::Level;

    use cow_sdk_test_utils::trace::TraceCapture;

    #[tokio::test]
    async fn cancellation_emits_debug_event_with_uid_field() {
        let capture = TraceCapture::install_global();
        let signer = RecordingSigner::new();
        let order_uid = sample_order_uid();

        sign_order_cancellation_with_scheme(
            &order_uid,
            SupportedChainId::Sepolia,
            &signer,
            SigningScheme::Eip712,
            None,
        )
        .await
        .expect("cancellation signing should succeed");

        let events = capture.events();
        let expected_uid = order_uid.to_hex_string();
        assert!(
            events.iter().any(|event| {
                event.target() == "cow_sdk::signing"
                    && event.level() == Level::DEBUG
                    && event.field("order_uid") == Some(expected_uid.as_str())
                    && event.field("order_uid_count") == Some("1")
            }),
            "cancellation signing must emit a debug event with the UID field: {events:#?}"
        );
    }
}
