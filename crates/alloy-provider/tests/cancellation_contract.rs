use cow_sdk_alloy_provider::AsyncProviderError;
use cow_sdk_core::{Cancellable, CancellationToken};

#[tokio::test]
async fn cancel_with_propagates_cancelled_through_question_mark() {
    let token = CancellationToken::new();
    token.cancel();

    let result: Result<(), AsyncProviderError> = async {
        async { Ok::<_, AsyncProviderError>(()) }
            .cancel_with(&token)
            .await?;
        Ok(())
    }
    .await;

    assert!(matches!(result, Err(AsyncProviderError::Cancelled)));
}
