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
//!     Arc::new(InMemoryQuoteCache::new(Duration::from_secs(30), 256));
//! let builder = TradingSdkBuilder::new().with_quote_cache(cache);
//! ```

use std::collections::HashMap;
use std::fmt;

#[cfg(target_arch = "wasm32")]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use async_trait::async_trait;
use cow_sdk_core::{Address, BuyTokenDestination, OrderKind, SellTokenSource};
use parking_lot::RwLock;

use crate::QuoteResults;

/// Default TTL applied by [`InMemoryQuoteCache`].
///
/// Five minutes mirrors
/// `cow_sdk_signing::cache::DEFAULT_EIP1271_VERIFICATION_CACHE_TTL` and stays
/// within the orderbook's quote validity envelope: standard `valid_to`
/// windows on production quotes are tens of seconds to a few minutes, so a
/// five-minute upper bound makes the cache a hot-path memoizer rather than a
/// stale-quote source. Callers who want a tighter or looser TTL pass it to
/// [`InMemoryQuoteCache::new`].
pub const DEFAULT_QUOTE_CACHE_TTL: Duration = Duration::from_secs(300);

/// Default capacity applied by [`InMemoryQuoteCache`].
///
/// 256 entries fits the observed key fan-out of the trading flow (chain ×
/// env × sell-token × buy-token × side × amount × balance
/// source/destination) for active sessions and keeps the oldest-first
/// eviction scan bounded under the signing-cache scan envelope. Callers with
/// a wider working set pass a higher capacity to [`InMemoryQuoteCache::new`].
pub const DEFAULT_QUOTE_CACHE_CAPACITY: usize = 256;

/// Time source used by [`InMemoryQuoteCache`].
///
/// The default [`SystemClock`] implementation calls [`Instant::now`]. Tests
/// can implement this trait with a deterministic clock to assert TTL
/// boundaries without sleeping. On `wasm32`, [`Instant`] resolves to
/// `web_time::Instant`; on native targets it resolves to
/// [`std::time::Instant`].
pub trait Clock: Send + Sync + 'static {
    /// Returns the current instant for cache timestamp comparisons.
    fn now(&self) -> Instant;
}

/// Wall-clock [`Clock`] used by default cache constructors.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Instant {
        Instant::now()
    }
}

impl<F> Clock for F
where
    F: Fn() -> Instant + Send + Sync + 'static,
{
    fn now(&self) -> Instant {
        self()
    }
}

/// Deterministic cache key derived from the minimal set of inputs that decide
/// whether two quote requests should share a cached result.
///
/// Address fields are normalized through [`Address::to_hex_string`] so legacy
/// checksum and lowercase variants of the same address hash to identical
/// keys. Every field is derived purely from the input request, so user
/// implementations backed by Redis or other shared caches can share entries
/// across processes deterministically.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[non_exhaustive]
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
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination.
    pub buy_token_balance: BuyTokenDestination,
}

impl QuoteCacheKey {
    /// Builds a cache key from pre-normalized inputs.
    ///
    /// Callers that already own the address, amount, and side fields can pass
    /// them directly. Addresses are re-normalized here through the typed
    /// [`Address::to_hex_string`] so the stored key is always in the
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
            sell_token: sell_token.to_hex_string(),
            buy_token: buy_token.to_hex_string(),
            receiver: None,
            owner: None,
            kind,
            amount: amount.into(),
            valid_to: None,
            partially_fillable: false,
            sell_token_balance: SellTokenSource::Erc20,
            buy_token_balance: BuyTokenDestination::Erc20,
        }
    }

    /// Returns a copy of this key with an explicit receiver.
    #[must_use]
    pub fn with_receiver(mut self, receiver: &Address) -> Self {
        self.receiver = Some(receiver.to_hex_string());
        self
    }

    /// Returns a copy of this key with an explicit owner.
    #[must_use]
    pub fn with_owner(mut self, owner: &Address) -> Self {
        self.owner = Some(owner.to_hex_string());
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
        sell_token_balance: SellTokenSource,
        buy_token_balance: BuyTokenDestination,
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

/// Capacity-bounded, TTL-respecting in-memory [`QuoteCache`] backed by
/// [`parking_lot::RwLock`].
///
/// The cache evicts the oldest entry (by insertion timestamp) when inserting
/// beyond the configured capacity, and refuses to return entries older than
/// the configured TTL. The default TTL is five minutes
/// ([`DEFAULT_QUOTE_CACHE_TTL`]) and the default capacity is 256 entries
/// ([`DEFAULT_QUOTE_CACHE_CAPACITY`]).
///
/// The store is `Send + Sync + 'static`, so the cache may be wrapped in
/// [`std::sync::Arc`] and shared across `tokio` tasks.
/// [`InMemoryQuoteCache::with_clock`] accepts a custom [`Clock`] for
/// deterministic TTL tests and embedders that already centralize time;
/// [`InMemoryQuoteCache::new`] preserves the default wall-clock behaviour.
///
/// # Eviction Trade-Off
///
/// Eviction beyond capacity is `O(N)` per insert: the oldest entry is found
/// by scanning the map for the minimum insertion timestamp. The default
/// `256`-entry bound keeps the scan comfortably bounded for the target
/// workloads (interactive sessions and bot loops at human quote rates).
/// Consumers that require a much larger key space should compose a proper
/// LRU-backed impl of [`QuoteCache`] rather than grow the in-memory cache
/// past a few thousand entries.
pub struct InMemoryQuoteCache<C = SystemClock> {
    entries: RwLock<HashMap<QuoteCacheKey, CachedQuote>>,
    ttl: Duration,
    capacity: usize,
    clock: C,
}

impl Default for InMemoryQuoteCache<SystemClock> {
    fn default() -> Self {
        Self::new(DEFAULT_QUOTE_CACHE_TTL, DEFAULT_QUOTE_CACHE_CAPACITY)
    }
}

impl InMemoryQuoteCache<SystemClock> {
    /// Creates a cache with the supplied TTL and capacity bound.
    #[must_use]
    pub fn new(ttl: Duration, capacity: usize) -> Self {
        Self::with_clock(ttl, capacity, SystemClock)
    }
}

impl<C> InMemoryQuoteCache<C>
where
    C: Clock,
{
    /// Creates a cache with the supplied TTL, capacity bound, and clock.
    ///
    /// The provided [`Clock`] is used for both write timestamps and read
    /// expiry checks. This keeps TTL behaviour deterministic in tests while
    /// leaving [`InMemoryQuoteCache::new`] on the production wall clock.
    #[must_use]
    pub fn with_clock(ttl: Duration, capacity: usize, clock: C) -> Self {
        let capacity = capacity.max(1);
        Self {
            entries: RwLock::new(HashMap::with_capacity(capacity)),
            ttl,
            capacity,
            clock,
        }
    }

    /// Returns the configured TTL.
    #[must_use]
    pub const fn ttl(&self) -> Duration {
        self.ttl
    }

    /// Returns the configured capacity bound.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns the current number of entries held in the cache.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.read().len()
    }

    /// Returns whether the cache currently holds zero entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.read().is_empty()
    }

    /// Removes every entry from the cache.
    pub fn clear(&self) {
        self.entries.write().clear();
    }
}

impl<C> fmt::Debug for InMemoryQuoteCache<C>
where
    C: Clock,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("InMemoryQuoteCache")
            .field("ttl", &self.ttl)
            .field("capacity", &self.capacity)
            .finish_non_exhaustive()
    }
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl<C> QuoteCache for InMemoryQuoteCache<C>
where
    C: Clock,
{
    #[allow(
        clippy::significant_drop_tightening,
        reason = "the write guard is held across the entire `lookup` body on purpose: \
                  this cache purges expired entries on lookup, so the guard must \
                  stay live while the entry is observed and potentially removed. \
                  Releasing it early would either require a second acquisition for \
                  the remove (introducing a TOCTOU race with concurrent inserts) \
                  or split lookup into a read-then-write shape that loses the \
                  lazy-expiry-on-lookup property."
    )]
    async fn lookup(&self, key: &QuoteCacheKey) -> Option<QuoteResults> {
        let now = self.clock.now();
        let mut entries = self.entries.write();
        if let Some(entry) = entries.get(key) {
            if now.duration_since(entry.inserted_at) > self.ttl {
                entries.remove(key);
                return None;
            }
            return Some(entry.value.clone());
        }
        None
    }

    async fn insert(&self, key: QuoteCacheKey, value: QuoteResults) {
        let entry = CachedQuote {
            value,
            inserted_at: self.clock.now(),
        };
        let mut entries = self.entries.write();
        entries.insert(key, entry);
        while entries.len() > self.capacity {
            let oldest_key = entries
                .iter()
                .min_by_key(|(_, value)| value.inserted_at)
                .map(|(key, _)| key.clone());
            match oldest_key {
                Some(key) => {
                    entries.remove(&key);
                }
                None => break,
            }
        }
        drop(entries);
    }

    async fn invalidate(&self, key: &QuoteCacheKey) {
        self.entries.write().remove(key);
    }
}
