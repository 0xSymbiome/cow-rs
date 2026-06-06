//! Contract tests: drive each double through its public trait and assert the
//! canned responses, recorded calls, injected failures, and `Trading` wiring.

use cow_sdk_core::{
    Address, Amount, AppDataHash, ContractCall, OrderUid, Provider, Signer, SigningProvider,
    SupportedChainId, TransactionReceipt, TransactionRequest, TransactionStatus,
};
use cow_sdk_orderbook::{
    OrderCancellations, OrderCreation, OrderKind, OrderQuoteRequest, OrderQuoteSide,
    OrderbookClient, SigningScheme,
};
use cow_sdk_test::{MockOrderbook, MockProvider, MockSigner, OrderbookFailure, defaults, trading};

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
        .send_signed_order_cancellations(&OrderCancellations::new(
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
async fn signer_returns_canned_and_records() {
    let signer = MockSigner::new();
    assert_eq!(signer.address(), defaults::address());

    let signature = signer
        .sign_message(b"hello")
        .await
        .expect("message signature");
    assert_eq!(signature, defaults::message_signature());

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
