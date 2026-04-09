mod common;

use cow_sdk_contracts::{OrderCancellations, SigningScheme, hash_order_cancellations};
use cow_sdk_core::SupportedChainId;
use cow_sdk_signing::{
    SigningError, get_domain, sign_order_cancellation, sign_order_cancellation_async,
    sign_order_cancellation_with_scheme, sign_order_cancellations_async,
    sign_order_cancellations_with_scheme,
};

use common::{MockSigner, sample_order_uid};

#[test]
fn single_and_batch_cancellation_signing_are_first_class() {
    let signer = MockSigner::new();
    let order_uid = sample_order_uid();
    let batch_uids = vec![
        order_uid.clone(),
        cow_sdk_core::OrderUid::new(
            "0x1aaa7dddecccc04cc101a121e3eed017eab4d3927c045d407d5ad6700eea2bf7fb3c7eb936caa12b5a884d612393969a557d430764060343",
        )
        .unwrap(),
    ];

    let single =
        sign_order_cancellation(&order_uid, SupportedChainId::Sepolia, &signer, None).unwrap();
    assert_eq!(single.signing_scheme, SigningScheme::Eip712);

    let batch = sign_order_cancellations_with_scheme(
        &batch_uids,
        SupportedChainId::Sepolia,
        &signer,
        SigningScheme::EthSign,
        None,
    )
    .unwrap();
    assert_eq!(batch.signing_scheme, SigningScheme::EthSign);
}

#[test]
fn cancellation_signing_uses_typed_data_and_ethsign_digest_paths() {
    let signer = MockSigner::new();
    let order_uid = sample_order_uid();

    sign_order_cancellation(&order_uid, SupportedChainId::Sepolia, &signer, None).unwrap();
    assert_eq!(signer.calls.borrow().typed_data.len(), 1);
    assert_eq!(
        signer.calls.borrow().typed_data[0].fields[0].kind,
        "bytes[]"
    );
    assert!(
        signer.calls.borrow().typed_data[0]
            .value_json
            .contains(order_uid.as_str())
    );

    let batch_uids = vec![order_uid.clone()];
    sign_order_cancellations_with_scheme(
        &batch_uids,
        SupportedChainId::Sepolia,
        &signer,
        SigningScheme::EthSign,
        None,
    )
    .unwrap();

    let expected_digest = hash_order_cancellations(
        &get_domain(SupportedChainId::Sepolia, None).unwrap(),
        &OrderCancellations {
            order_uids: batch_uids,
        },
    )
    .unwrap();
    assert_eq!(
        format!("0x{}", hex::encode(&signer.calls.borrow().messages[0])),
        expected_digest
    );
}

#[test]
fn unsupported_cancellation_modes_fail_with_typed_error() {
    let signer = MockSigner::new();
    let order_uid = sample_order_uid();

    let error = sign_order_cancellation_with_scheme(
        &order_uid,
        SupportedChainId::Mainnet,
        &signer,
        SigningScheme::PreSign,
        None,
    )
    .unwrap_err();

    assert_eq!(
        error,
        SigningError::UnsupportedSignerGeneratedScheme {
            scheme: SigningScheme::PreSign
        }
    );
}

#[tokio::test]
async fn async_cancellation_signing_paths_match_sync_variants() {
    let signer = MockSigner::new();
    let order_uid = sample_order_uid();

    let single =
        sign_order_cancellation_async(&order_uid, SupportedChainId::Sepolia, &signer, None)
            .await
            .unwrap();
    assert_eq!(single.signing_scheme, SigningScheme::Eip712);

    let batch = sign_order_cancellations_async(
        std::slice::from_ref(&order_uid),
        SupportedChainId::Sepolia,
        &signer,
        None,
    )
    .await
    .unwrap();
    assert_eq!(batch.signing_scheme, SigningScheme::Eip712);
}
