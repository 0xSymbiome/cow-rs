//! Optional caching seam for EIP-1271 signature verification.
//!
//! [`Eip1271Cache`] is the narrow trait consumed by
//! [`cow_sdk_contracts::verify_eip1271_signature_cached`]. The cache records
//! the `(verifier, digest, signature_hash)` probes that have verified VALID
//! so compositions that replay the same probe (composable orders,
//! flash-loans, bridging) avoid hitting the chain on every call. The trait
//! and the always-available [`NoopEip1271Cache`] carry no extra
//! dependencies; the capacity-bounded, TTL-respecting
//! [`InMemoryEip1271Cache`] is gated behind the opt-in
//! `in-memory-cache` feature, which is the only reason the signing crate
//! pulls `parking_lot` (and, on `wasm32`, `web-time`).
//!
//! # Cache key
//!
//! The key is the full probe identity `(verifier, digest, signature_hash)`,
//! where `signature_hash` is the `keccak256` of the signature bytes. The
//! on-chain `isValidSignature(hash, signature)` verdict is a function of the
//! signature as well as the digest, so the signature is part of the key; the
//! verify helper computes the hash before consulting the cache.
//!
//! # Cached-value semantics (positive-only)
//!
//! The cache is a *set* of probes observed VALID, not a `bool` map. Only a
//! successful magic-value match is recorded ([`Eip1271Cache::record_valid`]);
//! a magic-value mismatch and every other failure mode (transport, missing
//! contract code, serialization, hex decode) are **never recorded**, so those
//! probes re-hit the chain on the next call. A
//! [`get`](Eip1271Cache::contains_valid) miss means "unknown",
//! never "known invalid", so a not-yet-valid signature that becomes valid
//! on-chain within the TTL is never blocked by a stale negative entry. The
//! in-memory implementation uses [`SystemClock`] by default and exposes
//! [`InMemoryEip1271Cache::with_clock`] so tests and deterministic
//! runtimes can inject a controlled clock without changing production
//! wall-clock behaviour.

use cow_sdk_core::Address;

pub use cow_sdk_contracts::Eip1271Cache;

/// Zero-sized [`Eip1271Cache`] that never records anything.
///
/// Every [`contains_valid`](Eip1271Cache::contains_valid) call
/// returns `false`; every
/// [`record_valid`](Eip1271Cache::record_valid) call is a no-op.
/// Callers that do not want EIP-1271 caching pass a reference to this type to
/// keep the cache parameter on `verify_eip1271_signature_cached` mandatory
/// without paying any allocation or synchronization overhead. This is the
/// always-available default; it carries no dependencies and needs no feature.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct NoopEip1271Cache;

impl Eip1271Cache for NoopEip1271Cache {
    fn contains_valid(
        &self,
        _verifier: Address,
        _digest: [u8; 32],
        _signature_hash: [u8; 32],
    ) -> bool {
        false
    }

    fn record_valid(&self, _verifier: Address, _digest: [u8; 32], _signature_hash: [u8; 32]) {}
}

#[cfg(feature = "in-memory-cache")]
pub use in_memory::{Clock, InMemoryEip1271Cache, SystemClock};

#[cfg(feature = "in-memory-cache")]
mod in_memory {
    use std::collections::HashMap;

    #[cfg(target_arch = "wasm32")]
    use std::time::Duration;
    #[cfg(not(target_arch = "wasm32"))]
    use std::time::{Duration, Instant};
    #[cfg(target_arch = "wasm32")]
    use web_time::Instant;

    use cow_sdk_contracts::Eip1271Cache;
    use cow_sdk_core::Address;
    use parking_lot::RwLock;

    /// Probe key: the full `(verifier, digest, signature_hash)` identity.
    type ProbeKey = (Address, [u8; 32], [u8; 32]);

    /// Time source used by [`InMemoryEip1271Cache`].
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

    /// Capacity-bounded, TTL-respecting in-memory
    /// [`Eip1271Cache`] backed by [`parking_lot::RwLock`].
    ///
    /// The cache stores, for each `(verifier, digest, signature_hash)` probe
    /// observed VALID, the instant it was recorded. It evicts the oldest
    /// entry (by insertion timestamp) when recording beyond the configured
    /// capacity, and treats entries older than the configured TTL as absent.
    /// The default TTL is five minutes
    /// ([`InMemoryEip1271Cache::DEFAULT_TTL`]) and the default
    /// capacity is 1024 entries
    /// ([`InMemoryEip1271Cache::DEFAULT_CAPACITY`]).
    ///
    /// The store is `Send + Sync + 'static`, so the cache may be wrapped in
    /// [`std::sync::Arc`] and shared across `tokio` tasks.
    /// [`InMemoryEip1271Cache::with_clock`] accepts a custom
    /// [`Clock`] for deterministic TTL tests and embedders that already
    /// centralize time; [`InMemoryEip1271Cache::new`] preserves
    /// the default wall-clock behaviour.
    ///
    /// # Eviction Trade-Off
    ///
    /// Eviction beyond capacity is `O(N)` per record: the oldest entry is
    /// found by scanning the map for the minimum insertion timestamp. The
    /// default `1024`-entry bound keeps the scan comfortably bounded for
    /// the target workloads (composable orders, flash loans, bridging).
    /// Consumers that require a much larger key space, or that probe a
    /// very high fan-out of verifier addresses, should compose a proper
    /// LRU-backed impl of [`Eip1271Cache`] rather than grow
    /// the in-memory cache past a few thousand entries.
    #[derive(Debug)]
    pub struct InMemoryEip1271Cache<C = SystemClock> {
        inner: RwLock<HashMap<ProbeKey, Instant>>,
        ttl: Duration,
        capacity: usize,
        clock: C,
    }

    impl Default for InMemoryEip1271Cache<SystemClock> {
        fn default() -> Self {
            Self::new(Self::DEFAULT_TTL, Self::DEFAULT_CAPACITY)
        }
    }

    impl InMemoryEip1271Cache<SystemClock> {
        /// Default TTL applied by [`InMemoryEip1271Cache::default`].
        pub const DEFAULT_TTL: Duration = Duration::from_secs(300);
        /// Default capacity applied by [`InMemoryEip1271Cache::default`].
        pub const DEFAULT_CAPACITY: usize = 1024;

        /// Creates a cache with the supplied TTL and capacity bound.
        #[must_use]
        pub fn new(ttl: Duration, capacity: usize) -> Self {
            Self::with_clock(ttl, capacity, SystemClock)
        }
    }

    impl<C> InMemoryEip1271Cache<C>
    where
        C: Clock,
    {
        /// Creates a cache with the supplied TTL, capacity bound, and clock.
        ///
        /// # Clock Injection
        ///
        /// The provided [`Clock`] is used for both write timestamps and read
        /// expiry checks. This keeps TTL behaviour deterministic in tests
        /// while leaving [`InMemoryEip1271Cache::new`] on the
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

    impl<C> Eip1271Cache for InMemoryEip1271Cache<C>
    where
        C: Clock,
    {
        fn contains_valid(
            &self,
            verifier: Address,
            digest: [u8; 32],
            signature_hash: [u8; 32],
        ) -> bool {
            let now = self.clock.now();
            let read = self.inner.read();
            read.get(&(verifier, digest, signature_hash))
                .is_some_and(|inserted_at| now.duration_since(*inserted_at) <= self.ttl)
        }

        fn record_valid(&self, verifier: Address, digest: [u8; 32], signature_hash: [u8; 32]) {
            let now = self.clock.now();
            let mut write = self.inner.write();
            write.insert((verifier, digest, signature_hash), now);
            while write.len() > self.capacity {
                let oldest_key = write
                    .iter()
                    .min_by_key(|(_, inserted_at)| **inserted_at)
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
}
