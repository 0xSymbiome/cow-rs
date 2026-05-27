//! Optional caching seam for EIP-1271 signature verification.
//!
//! [`Eip1271VerificationCache`] is the narrow trait consumed by
//! [`cow_sdk_contracts::verify_eip1271_signature_cached`]. The cache stores
//! the boolean outcome of an EIP-1271 magic-value check so compositions
//! that replay the same `(verifier, digest)` probe (composable orders,
//! flash-loans, bridging) avoid hitting the chain on every call. Two
//! default implementations ship alongside the trait re-export:
//! [`NoopEip1271VerificationCache`] for callers that do not want
//! caching, and [`InMemoryEip1271VerificationCache`] for callers that
//! want a capacity-bounded, TTL-respecting in-memory store.
//! The in-memory implementation uses [`SystemClock`] by default and
//! exposes [`InMemoryEip1271VerificationCache::with_clock`] so tests
//! and deterministic runtimes can inject a controlled clock without
//! changing production wall-clock behaviour.
//!
//! # Cached-value semantics
//!
//! The cache stores `bool` values with one mapping:
//!
//! - `true` corresponds to a successful magic-value match (`Ok(())` from
//!   the verifier).
//! - `false` corresponds to a magic-value mismatch
//!   (`Err(ContractsError::Eip1271MagicValueMismatch { .. })`).
//!
//! Every other failure mode (transport, missing contract code,
//! serialization, hex decode) is **never cached** — those probes must
//! re-hit the chain on the next call so the caller observes the live
//! state of the on-chain verifier.

use std::collections::HashMap;

#[cfg(target_arch = "wasm32")]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant};
#[cfg(target_arch = "wasm32")]
use web_time::Instant;

use cow_sdk_core::Address;
use parking_lot::RwLock;

pub use cow_sdk_contracts::Eip1271VerificationCache;

/// Default TTL applied by [`InMemoryEip1271VerificationCache`].
pub const DEFAULT_EIP1271_VERIFICATION_CACHE_TTL: Duration = Duration::from_secs(300);
/// Default capacity applied by [`InMemoryEip1271VerificationCache`].
pub const DEFAULT_EIP1271_VERIFICATION_CACHE_CAPACITY: usize = 1024;

/// Zero-sized [`Eip1271VerificationCache`] that never caches anything.
///
/// Every [`get`](Eip1271VerificationCache::get) call returns
/// [`None`]; every [`put`](Eip1271VerificationCache::put) call is a
/// no-op. Callers that do not want EIP-1271 caching pass a reference
/// to this type to keep the cache parameter on
/// `verify_eip1271_signature_cached` mandatory without paying any
/// allocation or synchronization overhead.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NoopEip1271VerificationCache;

impl Eip1271VerificationCache for NoopEip1271VerificationCache {
    fn get(&self, _verifier: Address, _digest: [u8; 32]) -> Option<bool> {
        None
    }

    fn put(&self, _verifier: Address, _digest: [u8; 32], _result: bool) {}
}

/// Time source used by [`InMemoryEip1271VerificationCache`].
///
/// The default [`SystemClock`] implementation calls [`Instant::now`].
/// Tests can implement this trait with a deterministic clock to assert
/// TTL boundaries without sleeping. On `wasm32`, [`Instant`] resolves
/// to `web_time::Instant`; on native targets it resolves to
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

#[derive(Debug)]
struct CacheEntry {
    inserted_at: Instant,
    result: bool,
}

/// Capacity-bounded, TTL-respecting in-memory
/// [`Eip1271VerificationCache`] backed by [`parking_lot::RwLock`].
///
/// The cache evicts the oldest entry (by insertion timestamp) when
/// inserting beyond the configured capacity, and refuses to return
/// entries older than the configured TTL. The default TTL is five
/// minutes ([`DEFAULT_EIP1271_VERIFICATION_CACHE_TTL`]) and the
/// default capacity is 1024 entries
/// ([`DEFAULT_EIP1271_VERIFICATION_CACHE_CAPACITY`]).
///
/// The store is `Send + Sync + 'static`, so the cache may be wrapped in
/// [`std::sync::Arc`] and shared across `tokio` tasks.
/// [`InMemoryEip1271VerificationCache::with_clock`] accepts a custom
/// [`Clock`] for deterministic TTL tests and embedders that already
/// centralize time; [`InMemoryEip1271VerificationCache::new`] preserves
/// the default wall-clock behaviour.
///
/// # Eviction Trade-Off
///
/// Eviction beyond capacity is `O(N)` per insert: the oldest entry is
/// found by scanning the map for the minimum insertion timestamp. The
/// default `1024`-entry bound keeps the scan comfortably bounded for
/// the target workloads (composable orders, flash loans, bridging).
/// Consumers that require a much larger key space, or that probe a
/// very high fan-out of verifier addresses, should compose a proper
/// LRU-backed impl of [`Eip1271VerificationCache`] rather than grow
/// the in-memory cache past a few thousand entries.
#[derive(Debug)]
pub struct InMemoryEip1271VerificationCache<C = SystemClock> {
    inner: RwLock<HashMap<(Address, [u8; 32]), CacheEntry>>,
    ttl: Duration,
    capacity: usize,
    clock: C,
}

impl Default for InMemoryEip1271VerificationCache<SystemClock> {
    fn default() -> Self {
        Self::new(
            DEFAULT_EIP1271_VERIFICATION_CACHE_TTL,
            DEFAULT_EIP1271_VERIFICATION_CACHE_CAPACITY,
        )
    }
}

impl InMemoryEip1271VerificationCache<SystemClock> {
    /// Creates a cache with the supplied TTL and capacity bound.
    #[must_use]
    pub fn new(ttl: Duration, capacity: usize) -> Self {
        Self::with_clock(ttl, capacity, SystemClock)
    }
}

impl<C> InMemoryEip1271VerificationCache<C>
where
    C: Clock,
{
    /// Creates a cache with the supplied TTL, capacity bound, and clock.
    ///
    /// # Clock Injection
    ///
    /// The provided [`Clock`] is used for both write timestamps and read
    /// expiry checks. This keeps TTL behaviour deterministic in tests
    /// while leaving [`InMemoryEip1271VerificationCache::new`] on the
    /// production wall clock.
    #[must_use]
    pub fn with_clock(ttl: Duration, capacity: usize, clock: C) -> Self {
        let capacity = capacity.max(1);
        Self {
            inner: RwLock::new(HashMap::with_capacity(capacity)),
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
        self.inner.read().len()
    }

    /// Returns whether the cache currently holds zero entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.read().is_empty()
    }

    /// Removes every entry from the cache.
    pub fn clear(&self) {
        self.inner.write().clear();
    }
}

impl<C> Eip1271VerificationCache for InMemoryEip1271VerificationCache<C>
where
    C: Clock,
{
    fn get(&self, verifier: Address, digest: [u8; 32]) -> Option<bool> {
        let now = self.clock.now();
        let snapshot = {
            let read = self.inner.read();
            read.get(&(verifier, digest))
                .map(|entry| (entry.inserted_at, entry.result))
        };
        let (inserted_at, result) = snapshot?;
        if now.duration_since(inserted_at) > self.ttl {
            None
        } else {
            Some(result)
        }
    }

    fn put(&self, verifier: Address, digest: [u8; 32], result: bool) {
        let entry = CacheEntry {
            inserted_at: self.clock.now(),
            result,
        };
        let mut write = self.inner.write();
        write.insert((verifier, digest), entry);
        while write.len() > self.capacity {
            let oldest_key = write
                .iter()
                .min_by_key(|(_, value)| value.inserted_at)
                .map(|(key, _)| *key);
            match oldest_key {
                Some(key) => {
                    write.remove(&key);
                }
                None => break,
            }
        }
        drop(write);
    }
}
