//! Opt-in retry policy for the native Alloy provider adapter.

use std::time::Duration;

use alloy_transport::layers::RetryBackoffLayer;

/// Internal compute-units-per-second budget for the rate-limit backoff layer.
///
/// The adapter keeps this provider-tier tuning constant internal so the public
/// [`RetryConfig`] surface stays SDK-shaped (attempt count and initial backoff)
/// and does not leak the underlying transport layer's accounting model. The
/// value is a conservative default appropriate for shared public endpoints.
pub(crate) const RETRY_COMPUTE_UNITS_PER_SECOND: u64 = 100;

/// Opt-in retry policy for transient, rate-limited RPC reads.
///
/// By default an [`RpcAlloyProvider`] issues each RPC request once and surfaces
/// a transient transport failure — such as a public-endpoint `429 Too Many
/// Requests` — directly to the caller. That runtime-neutral default matches the
/// SDK posture that the consumer owns chain-RPC resilience.
///
/// Supplying a `RetryConfig` through [`RpcAlloyProviderBuilder::with_retry`]
/// opts into a bounded exponential backoff layer that transparently retries
/// rate-limited requests up to [`RetryConfig::max_retries`] times, starting from
/// [`RetryConfig::initial_backoff`]. The policy retries only rate-limit-class
/// transport errors; it never re-broadcasts a transaction, so callers that need
/// nonce-safe write retries still own that logic.
///
/// [`RpcAlloyProvider`]: crate::RpcAlloyProvider
/// [`RpcAlloyProviderBuilder::with_retry`]: crate::RpcAlloyProviderBuilder::with_retry
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetryConfig {
    max_retries: u32,
    initial_backoff: Duration,
}

impl RetryConfig {
    /// Creates a retry policy with the given maximum attempt count and initial
    /// backoff.
    #[must_use]
    pub const fn new(max_retries: u32, initial_backoff: Duration) -> Self {
        Self {
            max_retries,
            initial_backoff,
        }
    }

    /// Returns the maximum number of rate-limit retries.
    #[must_use]
    pub const fn max_retries(&self) -> u32 {
        self.max_retries
    }

    /// Returns the initial backoff applied before the first retry.
    #[must_use]
    pub const fn initial_backoff(&self) -> Duration {
        self.initial_backoff
    }

    /// Returns the initial backoff in whole milliseconds, saturating rather than
    /// wrapping so an out-of-range configuration can never panic.
    fn initial_backoff_millis(&self) -> u64 {
        u64::try_from(self.initial_backoff.as_millis()).unwrap_or(u64::MAX)
    }

    /// Builds the Alloy rate-limit backoff transport layer for this policy.
    ///
    /// Both the read-only provider leaf and the composed umbrella client route
    /// their layered JSON-RPC client through this single constructor so the
    /// retry behaviour and the internal compute-units budget stay defined in one
    /// place.
    pub(crate) fn backoff_layer(&self) -> RetryBackoffLayer {
        RetryBackoffLayer::new(
            self.max_retries,
            self.initial_backoff_millis(),
            RETRY_COMPUTE_UNITS_PER_SECOND,
        )
    }
}

impl Default for RetryConfig {
    /// Five retries with a 200 ms initial backoff — a conservative default for
    /// transient throttling on shared public RPC endpoints.
    fn default() -> Self {
        Self::new(5, Duration::from_millis(200))
    }
}
