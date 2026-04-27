//! Canonical cancellation combinator for long-running SDK futures.
//!
//! The [`Cancellable`] extension trait adds the [`Cancellable::cancel_with`]
//! adapter to every [`Future`]. The adapter wraps the inner future with a
//! [`WithCancellation`] selector that checks the borrowed
//! [`CancellationToken`] before each inner poll and, when the token fires,
//! resolves to the [`Cancelled`] marker lifted through the ambient error
//! type's `From<Cancelled>` implementation.
//!
//! The marker is deliberately minimal: every crate-level error aggregate
//! (`CoreError`, `ContractsError`, `SigningError`, `AppDataError`,
//! `OrderbookError`, `TradingError`, `SubgraphError`, `BrowserWalletError`) and
//! the facade `SdkError` implement `From<Cancelled>` into their typed
//! `Cancelled` variant. Operation code
//! therefore propagates cancellation with `?` across every public error
//! boundary without pulling the raw `tokio-util` future type into downstream
//! signatures.
//!
//! ```no_run
//! # async fn run() -> Result<(), cow_sdk_core::CoreError> {
//! use cow_sdk_core::{Cancellable, CancellationToken, CoreError};
//!
//! let token = CancellationToken::new();
//! let body = async { Ok::<_, CoreError>(String::new()) }
//!     .cancel_with(&token)
//!     .await?;
//! # drop(body);
//! # Ok(()) }
//! ```
//!
//! [`Future`]: core::future::Future

use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};

use pin_project_lite::pin_project;
use tokio_util::sync::{CancellationToken, WaitForCancellationFuture};

/// Marker error returned when a future wrapped through
/// [`Cancellable::cancel_with`] observes a fired [`CancellationToken`]
/// before the inner future resolves.
///
/// The marker carries no context by design: every crate-level error
/// aggregate ships a contextual `Cancelled` variant and lifts the marker
/// through a blanket `From<Cancelled>` implementation so cancellation can
/// propagate with `?` across every public error boundary.
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("operation cancelled")]
pub struct Cancelled;

pin_project! {
    /// Future adapter returned by [`Cancellable::cancel_with`].
    ///
    /// The adapter polls the inner future and, in a biased branch, races
    /// the borrowed [`CancellationToken`]'s fired state. When the token
    /// fires before the inner future resolves, the adapter returns
    /// [`Cancelled`] lifted through the ambient error type's
    /// `From<Cancelled>` implementation.
    pub struct WithCancellation<'t, F> {
        #[pin]
        inner: F,
        #[pin]
        cancel: WaitForCancellationFuture<'t>,
    }
}

impl<F> core::fmt::Debug for WithCancellation<'_, F> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("WithCancellation").finish_non_exhaustive()
    }
}

impl<F, T, E> Future for WithCancellation<'_, F>
where
    F: Future<Output = Result<T, E>>,
    E: From<Cancelled>,
{
    type Output = Result<T, E>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();

        // Biased: observe cancellation before polling the inner future.
        if this.cancel.poll(cx).is_ready() {
            #[cfg(feature = "tracing")]
            // Keep the subscriber-facing field spelling stable: cancelled=true.
            tracing::warn!(target: "cow_sdk::cancel", cancelled = true);
            return Poll::Ready(Err(E::from(Cancelled)));
        }

        this.inner.poll(cx)
    }
}

/// Extension trait that adds [`Cancellable::cancel_with`] to every
/// [`Future`].
///
/// The blanket implementation on every `F: Future` means any future can be
/// wrapped into a [`WithCancellation`] selector without per-type
/// boilerplate. The resulting adapter becomes a [`Future`] only when the
/// inner output is a `Result<T, E>` with `E: From<Cancelled>`, which every
/// crate-level SDK error satisfies.
///
/// [`Future`]: core::future::Future
pub trait Cancellable: Future + Sized {
    /// Wraps `self` into a [`WithCancellation`] adapter that resolves to
    /// [`Cancelled`] the moment the borrowed [`CancellationToken`] fires.
    ///
    /// The wrapper observes cancellation through a biased poll so a fired
    /// token wins over a simultaneously-ready inner future, and registers
    /// the token's waker before returning [`Poll::Pending`] so a later
    /// cancellation wakes the task.
    fn cancel_with(self, token: &CancellationToken) -> WithCancellation<'_, Self> {
        WithCancellation {
            inner: self,
            cancel: token.cancelled(),
        }
    }
}

impl<F: Future> Cancellable for F {}
