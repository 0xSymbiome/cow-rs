#![cfg(feature = "tracing")]
//! Telemetry contract for the transaction receipt-wait lifecycle.
//!
//! The submission and receipt observations are separated into two spans so a
//! submission span never implies inclusion: `transaction.submit` records only
//! the broadcast hash, while `transaction.receipt` records mined fields only
//! after a receipt is observed (ADR 0038).

mod common;

use std::time::Duration;

use cow_sdk_core::TransactionRequest;
use cow_sdk_test_utils::trace::TraceCapture;
use cow_sdk_trading::{WaitOptions, poll_for_receipt, submit_and_wait_for_receipt};

#[tokio::test(flavor = "current_thread")]
async fn submit_and_wait_emits_separated_submit_and_receipt_spans() {
    let capture = TraceCapture::install();
    let signer = common::FakeSigner::with_broadcast(common::test_hash());
    let provider =
        common::FakeProvider::with_receipt_after_polls(1, common::rich_receipt_fixture());
    let tx = TransactionRequest::default();

    submit_and_wait_for_receipt(
        &signer,
        &provider,
        &tx,
        WaitOptions::inclusion_default().with_poll_interval(Duration::from_millis(10)),
    )
    .await
    .expect("the configured receipt should be observed");

    let spans = capture.spans();
    let hash_hex = common::test_hash().to_string();

    let submit = spans
        .iter()
        .find(|span| span.name() == "transaction.submit")
        .unwrap_or_else(|| panic!("transaction.submit span must be emitted: {spans:#?}"));
    assert_eq!(submit.field("tx_hash"), Some(hash_hex.as_str()));
    // A submission span must never imply inclusion or execution success.
    for forbidden in ["tx_status", "block_number", "gas_used"] {
        assert!(
            submit.field(forbidden).is_none(),
            "submit span must not record the mined field {forbidden}: {submit:#?}"
        );
    }

    let receipt = spans
        .iter()
        .find(|span| span.name() == "transaction.receipt")
        .unwrap_or_else(|| panic!("transaction.receipt span must be emitted: {spans:#?}"));
    assert_eq!(receipt.field("tx_hash"), Some(hash_hex.as_str()));
    assert_eq!(receipt.field("tx_status"), Some("success"));
    assert_eq!(receipt.field("block_number"), Some("1234"));
    assert_eq!(receipt.field("gas_used"), Some("21000"));
}

#[tokio::test(flavor = "current_thread")]
async fn poll_for_receipt_emits_only_the_receipt_span() {
    let capture = TraceCapture::install();
    let provider =
        common::FakeProvider::with_receipt_immediately_available(common::rich_receipt_fixture());

    poll_for_receipt(
        &provider,
        &common::test_hash(),
        WaitOptions::inclusion_default(),
    )
    .await
    .expect("an immediately available receipt should be returned");

    let spans = capture.spans();
    assert!(
        spans.iter().any(|span| span.name() == "transaction.receipt"),
        "poll_for_receipt must emit the receipt span: {spans:#?}"
    );
    assert!(
        !spans.iter().any(|span| span.name() == "transaction.submit"),
        "poll_for_receipt does not broadcast, so it must not emit a submit span: {spans:#?}"
    );
}
