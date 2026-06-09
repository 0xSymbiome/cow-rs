//! Broadcast-then-poll helpers for workflows that need mined receipts.

use core::time::Duration;

#[cfg(not(target_arch = "wasm32"))]
use futures_timer::Delay;
#[cfg(target_arch = "wasm32")]
use gloo_timers::future::TimeoutFuture;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use cow_sdk_core::{
    Cancelled, Provider, Signer, TransactionBroadcast, TransactionHash, TransactionReceipt,
    TransactionRequest, TransactionStatus,
};

/// Configuration for receipt wait helpers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitOptions {
    /// How often to call `get_transaction_receipt` while the receipt is absent.
    pub poll_interval: Duration,
    /// Maximum duration to wait before returning [`WaitError::Timeout`].
    pub timeout: Duration,
    /// Whether a mined reverted status should return [`WaitError::Reverted`].
    pub require_success: bool,
}

impl WaitOptions {
    /// Creates wait options with `require_success` disabled.
    #[inline]
    #[must_use]
    pub const fn new(poll_interval: Duration, timeout: Duration) -> Self {
        Self {
            poll_interval,
            timeout,
            require_success: false,
        }
    }

    /// Default options for approval flows: two-second polling, sixty-second
    /// timeout, and revert-as-error behavior.
    #[must_use]
    pub const fn approve_default() -> Self {
        Self::new(Duration::from_secs(2), Duration::from_secs(60)).with_require_success(true)
    }

    /// Default options for inclusion-only waits: two-second polling,
    /// sixty-second timeout, and reverted receipts returned as receipts.
    #[must_use]
    pub const fn inclusion_default() -> Self {
        Self::new(Duration::from_secs(2), Duration::from_secs(60))
    }

    /// Returns a copy with a different poll interval.
    #[inline]
    #[must_use]
    pub const fn with_poll_interval(mut self, poll_interval: Duration) -> Self {
        self.poll_interval = poll_interval;
        self
    }

    /// Returns a copy with a different timeout.
    #[inline]
    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Returns a copy with a different success requirement.
    #[inline]
    #[must_use]
    pub const fn with_require_success(mut self, require_success: bool) -> Self {
        self.require_success = require_success;
        self
    }
}

/// Failure outcomes for receipt wait helpers.
#[non_exhaustive]
pub enum WaitError<S, P> {
    /// The signer rejected or failed transaction broadcast.
    Broadcast(S),
    /// A provider receipt lookup failed.
    Lookup(P),
    /// The receipt was not observed before the configured timeout elapsed.
    Timeout {
        /// Broadcast transaction hash that was polled.
        transaction_hash: TransactionHash,
        /// Observed elapsed time when the timeout fired.
        elapsed: Duration,
    },
    /// A receipt was observed with reverted status while success was required.
    Reverted {
        /// Full receipt returned by the provider.
        receipt: TransactionReceipt,
    },
    /// The caller cancelled the wrapped future cooperatively.
    Cancelled(Cancelled),
}

impl<S, P> WaitError<S, P> {
    /// Returns the reverted receipt when the wait failed because the mined
    /// transaction reverted on-chain, and `None` otherwise.
    ///
    /// Only [`WaitError::Reverted`] is a genuine on-chain failure; the other
    /// variants are transient or environmental — [`WaitError::Broadcast`] and
    /// [`WaitError::Lookup`] (signer and provider transport, carrying the
    /// caller's own error types), [`WaitError::Timeout`], and
    /// [`WaitError::Cancelled`]. This accessor never inspects the caller's
    /// signer or provider error, so its verdict is always sound.
    ///
    /// A reverted receipt surfaces through this variant only when
    /// [`WaitOptions::require_success`] is set; an inclusion-only wait returns
    /// `Ok(receipt)` and the caller reads the receipt's `status`.
    ///
    /// ```
    /// use cow_sdk_trading::WaitError;
    ///
    /// fn on_submit_failure<S, P>(error: &WaitError<S, P>) {
    ///     if error.reverted().is_some() {
    ///         // the mined transaction reverted on-chain — a real failure
    ///     } else {
    ///         // transient or environmental — retry the submit or surface it
    ///     }
    /// }
    /// ```
    #[must_use]
    pub const fn reverted(&self) -> Option<&TransactionReceipt> {
        match self {
            Self::Reverted { receipt } => Some(receipt),
            _ => None,
        }
    }
}

impl<S, P> std::fmt::Debug for WaitError<S, P>
where
    S: std::fmt::Debug,
    P: std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Broadcast(inner) => f.debug_tuple("Broadcast").field(inner).finish(),
            Self::Lookup(inner) => f.debug_tuple("Lookup").field(inner).finish(),
            Self::Timeout {
                transaction_hash,
                elapsed,
            } => f
                .debug_struct("Timeout")
                .field("transaction_hash", transaction_hash)
                .field("elapsed", elapsed)
                .finish(),
            Self::Reverted { receipt } => f
                .debug_struct("Reverted")
                .field("receipt", receipt)
                .finish(),
            Self::Cancelled(inner) => f.debug_tuple("Cancelled").field(inner).finish(),
        }
    }
}

impl<S, P> From<Cancelled> for WaitError<S, P> {
    fn from(cancelled: Cancelled) -> Self {
        Self::Cancelled(cancelled)
    }
}

impl<S, P> std::fmt::Display for WaitError<S, P>
where
    S: std::fmt::Display,
    P: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Broadcast(inner) => write!(f, "broadcast failed: {inner}"),
            Self::Lookup(inner) => write!(f, "receipt lookup failed: {inner}"),
            Self::Timeout {
                transaction_hash,
                elapsed,
            } => write!(
                f,
                "receipt for {transaction_hash} not observed after {elapsed:?}"
            ),
            Self::Reverted { receipt } => {
                write!(f, "transaction {} reverted", receipt.transaction_hash)
            }
            Self::Cancelled(_) => f.write_str("operation cancelled"),
        }
    }
}

impl<S, P> std::error::Error for WaitError<S, P>
where
    S: std::error::Error + 'static,
    P: std::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Broadcast(inner) => Some(inner),
            Self::Lookup(inner) => Some(inner),
            Self::Cancelled(inner) => Some(inner),
            Self::Timeout { .. } | Self::Reverted { .. } => None,
        }
    }
}

/// Broadcasts a transaction and polls until the mined receipt is observed.
///
/// This helper calls [`Signer::send_transaction`] exactly once, then
/// repeatedly calls [`Provider::get_transaction_receipt`] for the returned
/// hash until the provider returns a receipt or [`WaitOptions::timeout`] is
/// reached. Wrap the returned future with
/// [`cow_sdk_core::Cancellable::cancel_with`] to propagate cooperative
/// cancellation as [`WaitError::Cancelled`].
///
/// # Errors
///
/// Returns [`WaitError::Broadcast`] for signer failures,
/// [`WaitError::Lookup`] for provider lookup failures,
/// [`WaitError::Timeout`] when the receipt is not observed in time,
/// [`WaitError::Reverted`] when reverted status is observed while success is
/// required, and [`WaitError::Cancelled`] when wrapped through cooperative
/// cancellation.
pub async fn submit_and_wait_for_receipt<S, P>(
    signer: &S,
    provider: &P,
    tx: &TransactionRequest,
    options: WaitOptions,
) -> Result<TransactionReceipt, WaitError<S::Error, P::Error>>
where
    S: Signer,
    P: Provider,
{
    let broadcast = broadcast_transaction(signer, tx)
        .await
        .map_err(WaitError::Broadcast)?;

    poll_for_receipt_inner::<S::Error, P>(provider, &broadcast.transaction_hash, options).await
}

/// Broadcasts the transaction and records the broadcast acknowledgement.
///
/// The `transaction.submit` span records only the returned transaction hash on
/// success. It deliberately records no mined fields, so a submission span never
/// implies inclusion or execution success; the mined outcome belongs to the
/// separate `transaction.receipt` span (ADR 0038).
#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "transaction.submit",
        skip_all,
        fields(tx_hash = tracing::field::Empty),
    ),
)]
async fn broadcast_transaction<S>(
    signer: &S,
    tx: &TransactionRequest,
) -> Result<TransactionBroadcast, S::Error>
where
    S: Signer,
{
    let broadcast = signer.send_transaction(tx).await?;
    #[cfg(feature = "tracing")]
    tracing::Span::current().record(
        "tx_hash",
        tracing::field::display(&broadcast.transaction_hash),
    );
    Ok(broadcast)
}

/// Polls a provider for a receipt when the caller already has a hash.
///
/// The broadcast-side error parameter is [`std::convert::Infallible`] because
/// this helper does not submit a transaction.
///
/// # Errors
///
/// Returns [`WaitError::Lookup`] for provider lookup failures,
/// [`WaitError::Timeout`] when the receipt is not observed in time,
/// [`WaitError::Reverted`] when reverted status is observed while success is
/// required, and [`WaitError::Cancelled`] when wrapped through cooperative
/// cancellation.
pub async fn poll_for_receipt<P>(
    provider: &P,
    transaction_hash: &TransactionHash,
    options: WaitOptions,
) -> Result<TransactionReceipt, WaitError<std::convert::Infallible, P::Error>>
where
    P: Provider,
{
    poll_for_receipt_inner::<std::convert::Infallible, P>(provider, transaction_hash, options).await
}

#[cfg_attr(
    feature = "tracing",
    tracing::instrument(
        name = "transaction.receipt",
        skip_all,
        fields(
            tx_hash = %transaction_hash,
            tx_status = tracing::field::Empty,
            block_number = tracing::field::Empty,
            gas_used = tracing::field::Empty,
        ),
    ),
)]
async fn poll_for_receipt_inner<B, P>(
    provider: &P,
    transaction_hash: &TransactionHash,
    options: WaitOptions,
) -> Result<TransactionReceipt, WaitError<B, P::Error>>
where
    P: Provider,
{
    let started_at = Instant::now();

    loop {
        let lookup = provider
            .get_transaction_receipt(transaction_hash)
            .await
            .map_err(WaitError::Lookup)?;

        if let Some(receipt) = lookup {
            #[cfg(feature = "tracing")]
            record_receipt_fields(&receipt);
            if options.require_success
                && matches!(receipt.status, Some(TransactionStatus::Reverted))
            {
                return Err(WaitError::Reverted { receipt });
            }
            return Ok(receipt);
        }

        let elapsed = started_at.elapsed();
        if elapsed >= options.timeout {
            return Err(WaitError::Timeout {
                transaction_hash: *transaction_hash,
                elapsed,
            });
        }

        delay_for(options.poll_interval).await;
    }
}

/// Sleeps for the supplied polling delay on the active target.
///
/// # Panics
///
/// Panics only on `wasm32` if the explicitly clamped millisecond duration
/// cannot be represented by the timer API.
async fn delay_for(duration: Duration) {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Delay::new(duration).await;
    }

    #[cfg(target_arch = "wasm32")]
    {
        // SAFETY: clamp before converting for the wasm timer API.
        let millis = u32::try_from(duration.as_millis().min(u128::from(u32::MAX)))
            .expect("millisecond delay is clamped to `u32::MAX`");
        TimeoutFuture::new(millis).await;
    }
}

/// Records the mined fields of an observed receipt onto the current
/// `transaction.receipt` span.
///
/// The terminal status is recorded as a low-cardinality label, and the block
/// number and gas used are recorded only when the provider reports them. No
/// signature material, calldata, sender, or recipient address is recorded.
#[cfg(feature = "tracing")]
fn record_receipt_fields(receipt: &TransactionReceipt) {
    let span = tracing::Span::current();
    if let Some(status) = receipt.status.as_ref() {
        let label = match status {
            TransactionStatus::Success => "success",
            TransactionStatus::Reverted => "reverted",
            _ => "unknown",
        };
        span.record("tx_status", label);
    }
    if let Some(block_number) = receipt.block_number {
        span.record("block_number", block_number);
    }
    if let Some(gas_used) = receipt.gas_used.as_ref() {
        span.record("gas_used", tracing::field::display(gas_used));
    }
}
