//! Optional log-fetch capability layered on [`Provider`].

use crate::types::{LogQuery, RawLog};

use super::provider::Provider;

/// Log-fetch capability for providers that can serve `eth_getLogs`.
///
/// This is an opt-in capability supertrait layered on [`Provider`], mirroring
/// the [`SigningProvider`](super::SigningProvider) split: read-only adapters
/// implement only `Provider`; adapters that can additionally fetch event logs
/// implement `LogProvider`. A leaf crate bounds on `P: LogProvider` to fetch
/// logs without depending on any concrete provider adapter, and a read-only
/// adapter is never forced to carry log-fetch wiring it cannot serve.
///
/// [`get_logs`](LogProvider::get_logs) is the single bounded-call event scan:
/// one backend query over the caller's `[from_block, to_block]` range, returning
/// the raw logs for the caller to decode. It is deliberately not a watcher,
/// iterator, or indexer loop (ADR 0048); a caller that needs a wider range
/// issues further bounded calls itself.
///
/// ```
/// use cow_sdk_core::{LogProvider, LogQuery, RawLog};
///
/// async fn recent_logs<P: LogProvider>(provider: &P) -> Result<Vec<RawLog>, P::Error> {
///     provider.get_logs(&LogQuery::new(20_000_000, 20_000_100)).await
/// }
/// ```
#[expect(
    async_fn_in_trait,
    reason = "the trait surface adopts native async fn in trait per ADR 0010 runtime-neutral posture; the resulting non-Send futures are covered by the workspace future_not_send allow so wasm callbacks can satisfy the same trait without an explicit Send bound"
)]
pub trait LogProvider: Provider {
    /// Fetches event logs matching `query` in a single backend call.
    ///
    /// Issues exactly one backend log query over the query's caller-bounded
    /// `[from_block, to_block]` range and returns the raw logs for the caller to
    /// decode. A block range yields heterogeneous events, so decoding is left to
    /// the caller's family-specific decoder (`decode_settlement_log`,
    /// `decode_eth_flow_log`, …) applied to each [`RawLog::data`].
    /// Implementations must not expand the range, loop, poll, or watch.
    ///
    /// # Errors
    ///
    /// Returns the implementation-defined provider error when the log query
    /// fails.
    async fn get_logs(&self, query: &LogQuery) -> Result<Vec<RawLog>, Self::Error>;
}
