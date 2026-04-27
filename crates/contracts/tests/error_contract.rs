use cow_sdk_contracts::ContractsError;
use cow_sdk_core::{Cancellable, CancellationToken, Cancelled};

#[test]
fn cancelled_marker_lifts_to_contracts_error_cancelled() {
    let error = ContractsError::from(Cancelled);

    assert!(matches!(error, ContractsError::Cancelled));
}

#[tokio::test]
async fn cancellation_combinator_composes_with_contracts_error() {
    let token = CancellationToken::new();
    token.cancel();

    let result = async { Ok::<_, ContractsError>(()) }
        .cancel_with(&token)
        .await;

    assert!(matches!(result, Err(ContractsError::Cancelled)));
}
