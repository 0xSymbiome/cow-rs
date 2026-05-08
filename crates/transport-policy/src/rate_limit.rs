//! Request rate limiting for SDK HTTP clients.

use std::{
    collections::HashMap,
    future::{Future, poll_fn},
    pin::pin,
    sync::{Arc, Mutex},
    task::Poll,
    time::Duration,
};

use cow_sdk_core::{CancellationToken, Cancelled};
use url::Url;

use crate::time::sleep;

#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

/// Scope used to key limiter buckets.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LimiterScope {
    /// Every request uses one shared bucket.
    Global,
    /// Requests are keyed by `Url::host_str`.
    PerHost,
}

/// Shared request limiter used by SDK transport policies.
#[derive(Debug, Clone)]
pub struct RequestRateLimiter {
    tokens_per_interval: u32,
    interval: Duration,
    interval_label: &'static str,
    scope: LimiterScope,
    state: Arc<Mutex<HashMap<String, LimiterState>>>,
}

#[derive(Debug, Clone)]
struct LimiterState {
    window_started_at: Instant,
    remaining_tokens: u32,
}

impl PartialEq for RequestRateLimiter {
    fn eq(&self, other: &Self) -> bool {
        self.tokens_per_interval == other.tokens_per_interval
            && self.interval == other.interval
            && self.interval_label == other.interval_label
            && self.scope == other.scope
    }
}

impl Eq for RequestRateLimiter {}

impl RequestRateLimiter {
    /// Returns a builder seeded with documented defaults.
    #[must_use]
    pub const fn builder() -> RequestRateLimiterBuilder {
        RequestRateLimiterBuilder::new()
    }

    /// Returns the default orderbook request limiter.
    #[must_use]
    pub fn default_orderbook() -> Self {
        Self::builder().build()
    }

    /// Returns the default subgraph request limiter.
    #[must_use]
    pub fn default_subgraph() -> Self {
        Self::builder().build()
    }

    /// Returns a limiter that never delays requests.
    #[must_use]
    pub fn unlimited() -> Self {
        Self::builder().tokens_per_interval(0).build()
    }

    /// Returns the request budget granted per limiter interval.
    #[must_use]
    pub const fn tokens_per_interval(&self) -> u32 {
        self.tokens_per_interval
    }

    /// Returns the limiter interval.
    #[must_use]
    pub const fn interval(&self) -> Duration {
        self.interval
    }

    /// Returns the human-readable interval label.
    #[must_use]
    pub const fn interval_label(&self) -> &'static str {
        self.interval_label
    }

    /// Returns the bucket scope used by this limiter.
    #[must_use]
    pub const fn scope(&self) -> LimiterScope {
        self.scope
    }

    /// Returns the limiter key for `url` under this limiter's scope.
    #[must_use]
    pub fn key_for_url(&self, url: &Url) -> String {
        match self.scope {
            LimiterScope::Global => "global".to_owned(),
            LimiterScope::PerHost => url.host_str().unwrap_or("").to_ascii_lowercase(),
        }
    }

    /// Acquires one request token for `url`, delaying until one is available.
    ///
    /// # Errors
    ///
    /// Returns [`Cancelled`] if `cancellation_token` is cancelled while the
    /// limiter is waiting for a new token window.
    pub async fn acquire(
        &self,
        url: &Url,
        cancellation_token: &CancellationToken,
    ) -> Result<(), Cancelled> {
        let key = self.key_for_url(url);
        self.acquire_key(&key, cancellation_token).await
    }

    /// Acquires one request token in the global bucket.
    ///
    /// # Errors
    ///
    /// Returns [`Cancelled`] if `cancellation_token` is cancelled while the
    /// limiter is waiting for a new token window.
    pub async fn acquire_global(
        &self,
        cancellation_token: &CancellationToken,
    ) -> Result<(), Cancelled> {
        self.acquire_key("global", cancellation_token).await
    }

    async fn acquire_key(
        &self,
        key: &str,
        cancellation_token: &CancellationToken,
    ) -> Result<(), Cancelled> {
        if self.tokens_per_interval == 0 {
            return Ok(());
        }

        loop {
            if cancellation_token.is_cancelled() {
                return Err(Cancelled);
            }

            let wait = self.try_acquire_or_delay(key);
            if wait.is_zero() {
                return Ok(());
            }
            sleep_or_cancel(wait, cancellation_token).await?;
        }
    }

    #[allow(
        clippy::significant_drop_tightening,
        reason = "the mutex guard is held only while the bucket map is updated and is dropped before any sleep"
    )]
    fn try_acquire_or_delay(&self, key: &str) -> Duration {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let now = Instant::now();
        let bucket = state.entry(key.to_owned()).or_insert_with(|| LimiterState {
            window_started_at: now,
            remaining_tokens: self.tokens_per_interval,
        });

        let elapsed = now.duration_since(bucket.window_started_at);
        if elapsed >= self.interval {
            bucket.window_started_at = now;
            bucket.remaining_tokens = self.tokens_per_interval;
        }

        if bucket.remaining_tokens > 0 {
            bucket.remaining_tokens -= 1;
            Duration::ZERO
        } else {
            self.interval.saturating_sub(elapsed)
        }
    }
}

async fn sleep_or_cancel(
    duration: Duration,
    cancellation_token: &CancellationToken,
) -> Result<(), Cancelled> {
    if cancellation_token.is_cancelled() {
        return Err(Cancelled);
    }

    let mut sleep = pin!(sleep(duration));
    let mut cancelled = pin!(cancellation_token.cancelled());

    poll_fn(|context| {
        if cancellation_token.is_cancelled() {
            return Poll::Ready(Err(Cancelled));
        }
        if cancelled.as_mut().poll(context).is_ready() {
            return Poll::Ready(Err(Cancelled));
        }
        if sleep.as_mut().poll(context).is_ready() {
            return Poll::Ready(Ok(()));
        }
        Poll::Pending
    })
    .await
}

/// Builder for [`RequestRateLimiter`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RequestRateLimiterBuilder {
    tokens_per_interval: u32,
    interval: Duration,
    interval_label: &'static str,
    scope: LimiterScope,
}

impl RequestRateLimiterBuilder {
    /// Creates a builder seeded with documented limiter defaults.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tokens_per_interval: DEFAULT_TOKENS_PER_INTERVAL,
            interval: Duration::from_secs(1),
            interval_label: DEFAULT_INTERVAL_LABEL,
            scope: LimiterScope::PerHost,
        }
    }

    /// Sets the request budget granted per limiter interval.
    ///
    /// A zero budget disables limiting.
    #[must_use]
    pub const fn tokens_per_interval(mut self, tokens_per_interval: u32) -> Self {
        self.tokens_per_interval = tokens_per_interval;
        self
    }

    /// Sets the limiter interval.
    #[must_use]
    pub const fn interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    /// Sets the human-readable interval label.
    #[must_use]
    pub const fn interval_label(mut self, interval_label: &'static str) -> Self {
        self.interval_label = interval_label;
        self
    }

    /// Sets the bucket scope.
    #[must_use]
    pub const fn scope(mut self, scope: LimiterScope) -> Self {
        self.scope = scope;
        self
    }

    /// Builds the limiter.
    #[must_use]
    pub fn build(self) -> RequestRateLimiter {
        RequestRateLimiter {
            tokens_per_interval: self.tokens_per_interval,
            interval: self.interval,
            interval_label: self.interval_label,
            scope: self.scope,
            state: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for RequestRateLimiterBuilder {
    fn default() -> Self {
        Self::new()
    }
}
/// Default request budget granted per limiter interval.
pub const DEFAULT_TOKENS_PER_INTERVAL: u32 = 5;
/// Human-readable label for the default limiter interval.
pub const DEFAULT_INTERVAL_LABEL: &str = "second";
