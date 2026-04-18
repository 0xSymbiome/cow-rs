//! Contract coverage for the [`Cancellable`] combinator.
//!
//! These scenarios lock the observable behaviour of the additive
//! combinator introduced on `cow-sdk-core`: the biased cancellation poll,
//! the pass-through on a quiescent token, drop semantics when the wrapper
//! is discarded, and the generic composition shape consumers rely on.

#![allow(
    clippy::missing_const_for_fn,
    reason = "pedantic lints acceptable in test helper code"
)]

use core::future::Future;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use cow_sdk_core::{Cancellable, CancellationToken, CoreError};

#[tokio::test]
async fn pre_cancelled_token_resolves_to_cancelled_error() {
    let token = CancellationToken::new();
    token.cancel();

    let inner = async { Ok::<(), CoreError>(()) };
    let error = inner
        .cancel_with(&token)
        .await
        .expect_err("fired token must short-circuit the inner future");

    assert!(matches!(error, CoreError::Cancelled));
}

#[tokio::test]
async fn inner_future_passes_through_when_token_stays_quiescent() {
    let token = CancellationToken::new();
    let inner = async { Ok::<u32, CoreError>(42) };

    let value = inner
        .cancel_with(&token)
        .await
        .expect("inner must resolve when the token never fires");

    assert_eq!(value, 42);
}

#[tokio::test]
async fn dropping_the_wrapper_drops_the_inner_future() {
    struct DropMarker(Arc<AtomicBool>);

    impl Drop for DropMarker {
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }

    let dropped = Arc::new(AtomicBool::new(false));
    let marker = DropMarker(Arc::clone(&dropped));
    let token = CancellationToken::new();

    let inner = async move {
        // Keep the drop marker alive until the wrapper is discarded.
        let _keep = marker;
        core::future::pending::<Result<(), CoreError>>().await
    };

    let wrapped = inner.cancel_with(&token);
    drop(wrapped);

    assert!(
        dropped.load(Ordering::SeqCst),
        "the wrapper must drop the inner future when it is itself dropped",
    );
}

#[tokio::test]
async fn blanket_impl_composes_generic_futures() {
    async fn run_with_cancellation<F, T>(
        future: F,
        token: &CancellationToken,
    ) -> Result<T, CoreError>
    where
        F: Future<Output = Result<T, CoreError>>,
    {
        future.cancel_with(token).await
    }

    let token = CancellationToken::new();
    let value = run_with_cancellation(async { Ok::<u32, CoreError>(7) }, &token)
        .await
        .expect("quiescent token must let the inner resolve");
    assert_eq!(value, 7);

    token.cancel();
    let error = run_with_cancellation(async { Ok::<u32, CoreError>(7) }, &token)
        .await
        .expect_err("fired token must short-circuit through the generic adapter");
    assert!(matches!(error, CoreError::Cancelled));
}
