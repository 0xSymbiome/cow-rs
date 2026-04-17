//! Opt-in quote cache seam for the trading SDK.
//!
//! The [`QuoteCache`] trait exposes an async lookup, insert, and invalidate
//! contract that callers can plug into their trading flow alongside a
//! configured [`crate::TradingSdk`]. Policy decisions (TTL, keying,
//! invalidation) stay caller-owned and instance-scoped so Redis-backed or
//! other shared-process user implementations remain possible without changes
//! to the SDK surface.
//!
//! Two reference implementations are provided. [`NoopQuoteCache`] is a
//! pass-through default that always misses, keeping the trading flow
//! identical to the uncached case. [`InMemoryQuoteCache`] is a TTL-driven
//! in-process implementation aimed at short-lived deployments that do not
//! require cross-process sharing.
//!
//! # Usage
//!
//! ```ignore
//! use std::sync::Arc;
//! use std::time::Duration;
//!
//! use cow_sdk_trading::{InMemoryQuoteCache, QuoteCache, TradingSdkBuilder};
//!
//! let cache: Arc<dyn QuoteCache> =
//!     Arc::new(InMemoryQuoteCache::new(Duration::from_secs(30)));
//! let builder = TradingSdkBuilder::new().with_quote_cache(cache);
//! ```

use std::collections::HashMap;
use std::fmt;
use std::sync::Mutex;

#[cfg(target_arch = "wasm32")]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use async_trait::async_trait;
use cow_sdk_core::{Address, OrderBalance, OrderKind};

use crate::QuoteResults;

/// Deterministic cache key derived from the minimal set of inputs that decide
/// whether two quote requests should share a cached result.
///
/// Address fields are normalized through [`Address::normalized_key`] so legacy
/// checksum and lowercase variants of the same address hash to identical
/// keys. Every field is derived purely from the input request, so user
/// implementations backed by Redis or other shared caches can share entries
/// across processes deterministically.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QuoteCacheKey {
    /// Numeric chain id of the quote request.
    pub chain_id: u64,
    /// Environment label used when routing the quote request.
    pub env: &'static str,
    /// Lowercase-normalized sell-token address.
    pub sell_token: String,
    /// Lowercase-normalized buy-token address.
    pub buy_token: String,
    /// Optional lowercase-normalized receiver address.
    pub receiver: Option<String>,
    /// Optional lowercase-normalized owner address.
    pub owner: Option<String>,
    /// Order side.
    pub kind: OrderKind,
    /// Canonical decimal amount being quoted.
    pub amount: String,
    /// Optional absolute validity timestamp.
    pub valid_to: Option<u32>,
    /// Whether the quote request allows partial fills.
    pub partially_fillable: bool,
    /// Sell-token balance source.
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance source.
    pub buy_token_balance: OrderBalance,
}

impl QuoteCacheKey {
    /// Builds a cache key from pre-normalized inputs.
    ///
    /// Callers that already own the address, amount, and side fields can pass
    /// them directly. Addresses are re-normalized here through the typed
    /// [`Address::normalized_key`] so the stored key is always in the
    /// lowercase canonical form regardless of how the caller formatted the
    /// input.
    #[must_use]
    pub fn new(
        chain_id: u64,
        env: &'static str,
        sell_token: &Address,
        buy_token: &Address,
        amount: impl Into<String>,
        kind: OrderKind,
    ) -> Self {
        Self {
            chain_id,
            env,
            sell_token: sell_token.normalized_key(),
            buy_token: buy_token.normalized_key(),
            receiver: None,
            owner: None,
            kind,
            amount: amount.into(),
            valid_to: None,
            partially_fillable: false,
            sell_token_balance: OrderBalance::Erc20,
            buy_token_balance: OrderBalance::Erc20,
        }
    }

    /// Returns a copy of this key with an explicit receiver.
    #[must_use]
    pub fn with_receiver(mut self, receiver: &Address) -> Self {
        self.receiver = Some(receiver.normalized_key());
        self
    }

    /// Returns a copy of this key with an explicit owner.
    #[must_use]
    pub fn with_owner(mut self, owner: &Address) -> Self {
        self.owner = Some(owner.normalized_key());
        self
    }

    /// Returns a copy of this key with an explicit absolute validity timestamp.
    #[must_use]
    pub const fn with_valid_to(mut self, valid_to: u32) -> Self {
        self.valid_to = Some(valid_to);
        self
    }

    /// Returns a copy of this key with an explicit partially-fillable flag.
    #[must_use]
    pub const fn with_partially_fillable(mut self, partially_fillable: bool) -> Self {
        self.partially_fillable = partially_fillable;
        self
    }

    /// Returns a copy of this key with explicit token-balance sources.
    #[must_use]
    pub const fn with_token_balances(
        mut self,
        sell_token_balance: OrderBalance,
        buy_token_balance: OrderBalance,
    ) -> Self {
        self.sell_token_balance = sell_token_balance;
        self.buy_token_balance = buy_token_balance;
        self
    }
}

/// Async quote-cache seam for the trading SDK.
///
/// Implementations are expected to be idempotent on repeated calls with the
/// same key. Eviction, TTL, and cross-process sharing are caller-owned
/// policy; the trait only standardizes the lookup, insert, and invalidate
/// contract.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait QuoteCache: fmt::Debug {
    /// Returns a cached quote for `key`, or `None` when no entry is available.
    async fn lookup(&self, key: &QuoteCacheKey) -> Option<QuoteResults>;

    /// Inserts `value` into the cache under `key`.
    async fn insert(&self, key: QuoteCacheKey, value: QuoteResults);

    /// Removes any entry associated with `key`.
    async fn invalidate(&self, key: &QuoteCacheKey);
}

/// Pass-through [`QuoteCache`] implementation.
///
/// `NoopQuoteCache` is the trading SDK's default cache. Every lookup reports
/// a miss and inserts are silently dropped, so wiring this implementation
/// preserves the uncached call path byte-for-byte.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopQuoteCache;

impl NoopQuoteCache {
    /// Creates a new pass-through cache instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl QuoteCache for NoopQuoteCache {
    async fn lookup(&self, _key: &QuoteCacheKey) -> Option<QuoteResults> {
        None
    }

    async fn insert(&self, _key: QuoteCacheKey, _value: QuoteResults) {}

    async fn invalidate(&self, _key: &QuoteCacheKey) {}
}

struct CachedQuote {
    value: QuoteResults,
    inserted_at: Instant,
}

/// In-process TTL-based [`QuoteCache`] reference implementation.
///
/// Entries are evicted lazily the next time they are looked up after the TTL
/// elapses; this keeps the implementation lock-free on the insert path and
/// matches the usage pattern of short-lived deployments.
pub struct InMemoryQuoteCache {
    ttl: Duration,
    entries: Mutex<HashMap<QuoteCacheKey, CachedQuote>>,
}

impl InMemoryQuoteCache {
    /// Creates a new TTL-driven quote cache.
    #[must_use]
    pub fn new(ttl: Duration) -> Self {
        Self {
            ttl,
            entries: Mutex::new(HashMap::new()),
        }
    }

    /// Returns the configured TTL for this cache.
    #[must_use]
    pub const fn ttl(&self) -> Duration {
        self.ttl
    }

    fn lock_entries(&self) -> std::sync::MutexGuard<'_, HashMap<QuoteCacheKey, CachedQuote>> {
        self.entries
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl fmt::Debug for InMemoryQuoteCache {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryQuoteCache")
            .field("ttl", &self.ttl)
            .finish_non_exhaustive()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl QuoteCache for InMemoryQuoteCache {
    async fn lookup(&self, key: &QuoteCacheKey) -> Option<QuoteResults> {
        let mut entries = self.lock_entries();
        let expired = entries
            .get(key)
            .is_some_and(|entry| entry.inserted_at.elapsed() > self.ttl);
        if expired {
            entries.remove(key);
            return None;
        }
        entries.get(key).map(|entry| entry.value.clone())
    }

    async fn insert(&self, key: QuoteCacheKey, value: QuoteResults) {
        let mut entries = self.lock_entries();
        entries.insert(
            key,
            CachedQuote {
                value,
                inserted_at: Instant::now(),
            },
        );
    }

    async fn invalidate(&self, key: &QuoteCacheKey) {
        let mut entries = self.lock_entries();
        entries.remove(key);
    }
}
