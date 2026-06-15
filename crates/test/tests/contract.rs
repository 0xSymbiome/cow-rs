//! Contract tests: drive each double through its public trait and assert the
//! canned responses, recorded calls, injected failures, and `Trading` wiring.

use std::collections::BTreeMap;

use cow_sdk_contracts::{RecoverableSignature, SigningScheme as RecoverScheme};
use cow_sdk_core::{
    Address, Amount, AppDataHash, ContractCall, Hash32, OrderUid, Provider, Signer,
    SigningProvider, SupportedChainId, TransactionReceipt, TransactionRequest, TransactionStatus,
    TypedDataDomain, TypedDataField, TypedDataPayload,
};
use cow_sdk_orderbook::{
    OrderCancellations, OrderCreation, OrderKind, OrderQuoteRequest, OrderQuoteSide,
    OrderbookClient, SigningScheme,
};
use cow_sdk_test::{MockOrderbook, MockProvider, MockSigner, OrderbookFailure, defaults, trading};

/// A minimal valid EIP-712 typed-data payload to exercise real signing.
fn sample_typed_data_payload() -> TypedDataPayload {
    let mut types = BTreeMap::new();
    types.insert(
        "EIP712Domain".to_owned(),
        vec![
            TypedDataField::new("name".to_owned(), "string".to_owned()),
            TypedDataField::new("version".to_owned(), "string".to_owned()),
            TypedDataField::new("chainId".to_owned(), "uint256".to_owned()),
            TypedDataField::new("verifyingContract".to_owned(), "address".to_owned()),
        ],
    );
    types.insert(
        "Mail".to_owned(),
        vec![TypedDataField::new(
            "contents".to_owned(),
            "string".to_owned(),
        )],
    );
    TypedDataPayload::new(
        TypedDataDomain::new(
            "cow-sdk-test".to_owned(),
            "1".to_owned(),
            11_155_111,
            Address::ZERO,
        ),
        "Mail".to_owned(),
        types,
        r#"{"contents":"hello"}"#.to_owned(),
    )
}

/// The EIP-712 signing hash of `payload`, recomputed independently of the
/// signer through the canonical Alloy typed-data shape.
fn eip712_digest(payload: &TypedDataPayload) -> Hash32 {
    let message: serde_json::Value =
        serde_json::from_str(payload.message_json()).expect("message json");
    let typed: alloy_dyn_abi::eip712::TypedData = serde_json::from_value(serde_json::json!({
        "domain": payload.domain,
        "types": payload.types,
        "primaryType": payload.primary_type,
        "message": message,
    }))
    .expect("typed-data shape");
    Hash32::from_bytes(typed.eip712_signing_hash().expect("signing hash").0)
}

fn quote_request() -> OrderQuoteRequest {
    OrderQuoteRequest::new(
        Address::ZERO,
        Address::ZERO,
        Address::ZERO,
        OrderQuoteSide::sell(Amount::from(1_u64)),
    )
}

fn order_creation() -> OrderCreation {
    OrderCreation::new(
        Address::ZERO,
        Address::ZERO,
        Amount::from(1_u64),
        Amount::from(2_u64),
        defaults::FAR_FUTURE_VALID_TO,
        OrderKind::Sell,
        SigningScheme::Eip712,
        defaults::message_signature(),
        Address::ZERO,
    )
}

#[tokio::test]
async fn trading_wires_aligned_doubles() {
    let testing = trading(SupportedChainId::Sepolia, "cow-sdk-test-suite")
        .expect("the default app code is valid");
    assert!(testing.orderbook.recorded().sent_orders.is_empty());
    assert!(testing.signer.recorded().sent_transactions.is_empty());
}

#[tokio::test]
async fn orderbook_returns_canned_and_records() {
    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia);

    let quote = orderbook
        .quote(&quote_request())
        .await
        .expect("canned quote");
    assert_eq!(quote.id, Some(1));

    let uid = orderbook
        .send_order(&order_creation())
        .await
        .expect("canned uid");
    assert_eq!(uid, defaults::order_uid());

    orderbook
        .upload_app_data(&AppDataHash::ZERO, "{}")
        .await
        .expect("upload ok");
    orderbook
        .send_cancellations(&OrderCancellations::new(
            vec![defaults::order_uid()],
            defaults::message_signature(),
        ))
        .await
        .expect("cancel ok");

    let recorded = orderbook.recorded();
    assert_eq!(recorded.quote_requests.len(), 1);
    assert_eq!(recorded.sent_orders.len(), 1);
    assert_eq!(recorded.uploads.len(), 1);
    assert_eq!(recorded.cancellations.len(), 1);
}

#[tokio::test]
async fn orderbook_get_order_resolves_registered_and_rejects_unknown() {
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .order(defaults::order())
        .build();

    let found = orderbook
        .order(&defaults::order_uid())
        .await
        .expect("registered order resolves");
    assert_eq!(found.uid, defaults::order_uid());

    assert!(orderbook.order(&OrderUid::ZERO).await.is_err());
}

#[tokio::test]
async fn orderbook_injected_failure_surfaces() {
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .fail_quote(OrderbookFailure::RateLimited)
        .build();
    assert!(orderbook.quote(&quote_request()).await.is_err());
}

#[tokio::test]
async fn injected_failure_still_records_the_attempt() {
    // Record-first: an injected failure still leaves the request in the log, so
    // an error-path test can assert the call was attempted.
    let orderbook = MockOrderbook::builder(SupportedChainId::Sepolia)
        .fail_send(OrderbookFailure::Rejected("nope".to_owned()))
        .build();
    assert!(orderbook.send_order(&order_creation()).await.is_err());
    assert_eq!(orderbook.recorded().sent_orders.len(), 1);
}

#[tokio::test]
async fn signer_signs_and_records() {
    let signer = MockSigner::new();
    assert_eq!(signer.address(), defaults::address());

    // The default signer really signs: a canonical 65-byte recoverable
    // signature, not a fixed constant.
    let signature = signer
        .sign_message(b"hello")
        .await
        .expect("message signature");
    assert!(signature.starts_with("0x") && signature.len() == 132);

    let tx = TransactionRequest::new(
        Some(Address::ZERO),
        None,
        Some(Amount::ZERO),
        Some(Amount::from(21_000_u64)),
    );
    let broadcast = signer.send_transaction(&tx).await.expect("broadcast");
    assert_eq!(broadcast.transaction_hash, defaults::transaction_hash());

    let recorded = signer.recorded();
    assert_eq!(recorded.signed_messages.len(), 1);
    assert_eq!(recorded.sent_transactions.len(), 1);
}

#[tokio::test]
async fn default_signature_recovers_to_the_reported_address() {
    // The default signer's typed-data signature recovers to the address it
    // reports, so a signed order clears the SDK's owner-recovery gate.
    let signer = MockSigner::new();
    let payload = sample_typed_data_payload();

    let signature = signer
        .sign_typed_data_payload(&payload)
        .await
        .expect("typed-data signature");
    let recovered = RecoverableSignature::parse_hex(&signature)
        .expect("canonical recoverable signature")
        .recover(&eip712_digest(&payload), RecoverScheme::Eip712)
        .expect("recovery");

    assert_eq!(recovered, signer.address());
}

#[tokio::test]
async fn reporting_a_different_address_models_a_mismatched_signer() {
    // A signer that reports an address other than its signing key's address
    // produces a signature that recovers elsewhere — the owner-recovery gate's
    // mismatch case.
    let signer = MockSigner::builder().address(Address::ZERO).build();
    let payload = sample_typed_data_payload();

    let signature = signer
        .sign_typed_data_payload(&payload)
        .await
        .expect("typed-data signature");
    let recovered = RecoverableSignature::parse_hex(&signature)
        .expect("canonical recoverable signature")
        .recover(&eip712_digest(&payload), RecoverScheme::Eip712)
        .expect("recovery");

    assert_ne!(recovered, signer.address());
    assert_eq!(recovered, defaults::address());
}

#[tokio::test]
async fn canned_signature_override_returns_the_fixed_value() {
    // The fixed-signature overrides bypass real signing for error-path and
    // wire-shape tests.
    let fixed = format!("0x{}1b", "ab".repeat(64));
    let signer = MockSigner::builder()
        .typed_data_signature(fixed.clone())
        .message_signature(fixed.clone())
        .build();

    assert_eq!(
        signer
            .sign_typed_data_payload(&sample_typed_data_payload())
            .await
            .expect("typed-data signature"),
        fixed
    );
    assert_eq!(
        signer.sign_message(b"hi").await.expect("message signature"),
        fixed
    );
}

#[tokio::test]
async fn signer_injected_failure_surfaces() {
    let signer = MockSigner::builder().fail_send("declined").build();
    let tx = TransactionRequest::new(None, None, None, None);
    assert!(signer.send_transaction(&tx).await.is_err());
}

#[tokio::test]
async fn provider_reports_chain_and_records_reads() {
    let provider = MockProvider::builder()
        .chain_id(SupportedChainId::Sepolia)
        .allowance(Amount::from(42_u64))
        .build();
    assert_eq!(
        provider.get_chain_id().await.expect("chain id"),
        u64::from(SupportedChainId::Sepolia)
    );

    let call = ContractCall::new(
        Address::ZERO,
        "allowance".to_owned(),
        "[]".to_owned(),
        "[]".to_owned(),
    );
    let read = provider.read_contract(&call).await.expect("read");
    assert_eq!(read, Amount::from(42_u64).to_string());
    assert_eq!(provider.recorded().contract_reads.len(), 1);

    let signer = provider.create_signer("hint").await.expect("signer");
    assert_eq!(signer.address(), defaults::address());
}

#[tokio::test]
async fn provider_receipt_sequence_scripts_each_poll() {
    let mined = TransactionReceipt::new(defaults::transaction_hash())
        .with_status(TransactionStatus::Success)
        .with_block_number(100);
    let provider = MockProvider::builder()
        .receipt_sequence([None, None, Some(mined.clone())])
        .build();
    let hash = defaults::transaction_hash();

    // Not mined for the first two polls, then the scripted receipt.
    assert_eq!(
        provider.get_transaction_receipt(&hash).await.expect("poll"),
        None
    );
    assert_eq!(
        provider.get_transaction_receipt(&hash).await.expect("poll"),
        None
    );
    assert_eq!(
        provider.get_transaction_receipt(&hash).await.expect("poll"),
        Some(mined)
    );
    // Script exhausted: falls back to the absent static receipt, i.e. a timeout.
    assert_eq!(
        provider.get_transaction_receipt(&hash).await.expect("poll"),
        None
    );
}

#[tokio::test]
async fn provider_receipt_sequence_can_script_a_revert() {
    let reverted = TransactionReceipt::new(defaults::transaction_hash())
        .with_status(TransactionStatus::Reverted);
    let provider = MockProvider::builder()
        .receipt_sequence([Some(reverted)])
        .build();

    let receipt = provider
        .get_transaction_receipt(&defaults::transaction_hash())
        .await
        .expect("poll")
        .expect("a receipt is scripted for the first poll");
    assert_eq!(receipt.status, Some(TransactionStatus::Reverted));
}

#[test]
fn doubles_are_send() {
    const fn assert_send<T: Send>() {}
    assert_send::<MockOrderbook>();
    assert_send::<MockSigner>();
    assert_send::<MockProvider>();
}
