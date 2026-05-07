mod common;

use std::time::Duration;

use cow_sdk_core::{Cancellable, CancellationToken, TransactionRequest, TransactionStatus};
use cow_sdk_trading::{WaitError, WaitOptions, poll_for_receipt, submit_and_wait_for_receipt};

#[tokio::test]
async fn submit_and_wait_returns_receipt_when_provider_yields_success() {
    let signer = common::FakeSigner::with_broadcast(common::test_hash());
    let provider =
        common::FakeProvider::with_receipt_after_polls(2, common::rich_receipt_fixture());
    let tx = TransactionRequest::default();

    let receipt = submit_and_wait_for_receipt(
        &signer,
        &provider,
        &tx,
        WaitOptions::approve_default().with_poll_interval(Duration::from_millis(10)),
    )
    .await
    .unwrap();

    assert_eq!(receipt.status, Some(TransactionStatus::Success));
    assert_eq!(receipt.block_number, Some(1_234));
    assert_eq!(provider.poll_count(), 2);
}

#[tokio::test]
async fn submit_and_wait_returns_reverted_when_require_success_and_status_reverted() {
    let signer = common::FakeSigner::with_broadcast(common::test_hash());
    let reverted = common::rich_receipt_fixture().with_status(TransactionStatus::Reverted);
    let provider = common::FakeProvider::with_receipt_after_polls(1, reverted);
    let tx = TransactionRequest::default();

    let result =
        submit_and_wait_for_receipt(&signer, &provider, &tx, WaitOptions::approve_default()).await;

    match result {
        Err(WaitError::Reverted { receipt }) => {
            assert_eq!(receipt.status, Some(TransactionStatus::Reverted));
        }
        other => panic!("expected Reverted, got {other:?}"),
    }
}

#[tokio::test]
async fn submit_and_wait_returns_receipt_when_require_success_false_and_status_reverted() {
    let signer = common::FakeSigner::with_broadcast(common::test_hash());
    let reverted = common::rich_receipt_fixture().with_status(TransactionStatus::Reverted);
    let provider = common::FakeProvider::with_receipt_after_polls(1, reverted);
    let tx = TransactionRequest::default();

    let receipt =
        submit_and_wait_for_receipt(&signer, &provider, &tx, WaitOptions::inclusion_default())
            .await
            .unwrap();

    assert_eq!(receipt.status, Some(TransactionStatus::Reverted));
}

#[tokio::test]
async fn submit_and_wait_returns_timeout_when_provider_never_yields_receipt() {
    let signer = common::FakeSigner::with_broadcast(common::test_hash());
    let provider = common::FakeProvider::never_yields_receipt();
    let tx = TransactionRequest::default();
    let options = WaitOptions::new(Duration::from_millis(25), Duration::from_millis(100));

    let result = submit_and_wait_for_receipt(&signer, &provider, &tx, options).await;

    match result {
        Err(WaitError::Timeout {
            transaction_hash,
            elapsed,
        }) => {
            assert_eq!(transaction_hash, common::test_hash());
            assert!(elapsed >= Duration::from_millis(100));
        }
        other => panic!("expected Timeout, got {other:?}"),
    }
}

#[tokio::test]
async fn submit_and_wait_propagates_signer_error_as_broadcast_variant() {
    let signer = common::FakeSigner::with_error(common::FakeSignerError::Boom);
    let provider = common::FakeProvider::never_polled();
    let tx = TransactionRequest::default();

    let result =
        submit_and_wait_for_receipt(&signer, &provider, &tx, WaitOptions::inclusion_default())
            .await;

    assert!(matches!(
        result,
        Err(WaitError::Broadcast(common::FakeSignerError::Boom))
    ));
    assert_eq!(provider.poll_count(), 0);
}

#[tokio::test]
async fn submit_and_wait_propagates_provider_error_as_lookup_variant() {
    let signer = common::FakeSigner::with_broadcast(common::test_hash());
    let provider = common::FakeProvider::with_lookup_error_on_first_poll();
    let tx = TransactionRequest::default();

    let result =
        submit_and_wait_for_receipt(&signer, &provider, &tx, WaitOptions::inclusion_default())
            .await;

    assert!(matches!(
        result,
        Err(WaitError::Lookup(common::FakeProviderError::LookupFailed))
    ));
}

#[tokio::test]
async fn poll_for_receipt_returns_receipt_when_already_available() {
    let provider =
        common::FakeProvider::with_receipt_immediately_available(common::rich_receipt_fixture());

    let receipt = poll_for_receipt(
        &provider,
        &common::test_hash(),
        WaitOptions::inclusion_default(),
    )
    .await
    .unwrap();

    assert_eq!(receipt.transaction_hash, common::test_hash());
    assert_eq!(provider.poll_count(), 1);
}

#[tokio::test]
async fn cancellation_through_cancellable_propagates_through_helper() {
    let token = CancellationToken::new();
    let signer = common::FakeSigner::with_broadcast(common::test_hash());
    let provider = common::FakeProvider::never_yields_receipt();
    let tx = TransactionRequest::default();
    let options = WaitOptions::inclusion_default()
        .with_poll_interval(Duration::from_millis(100))
        .with_timeout(Duration::from_millis(500));

    let token_for_task = token.clone();
    let task = tokio::spawn({
        let signer = signer.clone();
        let provider = provider.clone();
        async move {
            submit_and_wait_for_receipt(&signer, &provider, &tx, options)
                .cancel_with(&token_for_task)
                .await
        }
    });

    token.cancel();
    let result = task.await.expect("helper task should not panic");

    assert!(matches!(result, Err(WaitError::Cancelled(_))));
}
