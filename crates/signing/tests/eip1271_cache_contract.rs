#![cfg(not(target_arch = "wasm32"))]
//! Public-surface contract assertions for the EIP-1271 verification
//! cache trait and the two default implementations shipped from the
//! signing crate.
//!
//! The `Noop` impl asserts an always-miss contract; the `InMemory`
//! impl asserts TTL-bounded retention, capacity-bounded eviction, and
//! `Send + Sync` thread-safety under concurrent probe-and-populate
//! load.

use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use cow_sdk_core::Address;
use cow_sdk_signing::cache::{
    DEFAULT_EIP1271_VERIFICATION_CACHE_CAPACITY, DEFAULT_EIP1271_VERIFICATION_CACHE_TTL,
    Eip1271VerificationCache, InMemoryEip1271VerificationCache, NoopEip1271VerificationCache,
};

fn sample_address(seed: u8) -> Address {
    Address::new(format!(
        "0x{}",
        String::from_utf8(vec![b'1' + seed % 9; 40]).unwrap()
    ))
    .unwrap()
}

const fn digest(seed: u8) -> [u8; 32] {
    [seed; 32]
}

#[test]
fn noop_cache_always_misses_and_never_records_writes() {
    let cache = NoopEip1271VerificationCache;
    let verifier = sample_address(0);
    let key_digest = digest(1);

    assert_eq!(cache.get(verifier.clone(), key_digest), None);
    cache.put(verifier.clone(), key_digest, true);
    assert_eq!(cache.get(verifier, key_digest), None);
}

#[test]
fn in_memory_cache_default_ttl_and_capacity_match_documented_constants() {
    let cache = InMemoryEip1271VerificationCache::default();
    assert_eq!(cache.ttl(), DEFAULT_EIP1271_VERIFICATION_CACHE_TTL);
    assert_eq!(
        cache.capacity(),
        DEFAULT_EIP1271_VERIFICATION_CACHE_CAPACITY
    );
    assert!(cache.is_empty());
}

#[test]
fn in_memory_cache_round_trips_a_recorded_outcome() {
    let cache = InMemoryEip1271VerificationCache::default();
    let verifier = sample_address(1);
    let key_digest = digest(7);

    assert_eq!(cache.get(verifier.clone(), key_digest), None);
    cache.put(verifier.clone(), key_digest, true);
    assert_eq!(cache.get(verifier.clone(), key_digest), Some(true));
    cache.put(verifier.clone(), key_digest, false);
    assert_eq!(cache.get(verifier, key_digest), Some(false));
}

#[test]
fn in_memory_cache_respects_ttl_expiry() {
    let cache = InMemoryEip1271VerificationCache::new(Duration::from_millis(40), 16);
    let verifier = sample_address(2);
    let key_digest = digest(3);

    cache.put(verifier.clone(), key_digest, true);
    assert_eq!(cache.get(verifier.clone(), key_digest), Some(true));
    sleep(Duration::from_millis(80));
    assert_eq!(cache.get(verifier, key_digest), None);
}

#[test]
fn in_memory_cache_evicts_oldest_entry_when_capacity_is_exceeded() {
    let cache = InMemoryEip1271VerificationCache::new(Duration::from_secs(60), 2);
    let verifier = sample_address(3);

    cache.put(verifier.clone(), digest(1), true);
    sleep(Duration::from_millis(2));
    cache.put(verifier.clone(), digest(2), true);
    sleep(Duration::from_millis(2));
    cache.put(verifier.clone(), digest(3), true);

    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get(verifier.clone(), digest(1)), None);
    assert_eq!(cache.get(verifier.clone(), digest(2)), Some(true));
    assert_eq!(cache.get(verifier, digest(3)), Some(true));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn in_memory_cache_is_thread_safe_under_concurrent_probe_and_populate_load() {
    const TASKS: usize = 64;
    const PROBES: usize = 256;
    const KEY_SPACE: u8 = 8;
    const VERIFIER_SPACE: u8 = 4;

    let cache = Arc::new(InMemoryEip1271VerificationCache::new(
        Duration::from_secs(60),
        4096,
    ));

    let mut handles = Vec::with_capacity(TASKS);
    for task_id in 0..TASKS {
        let cache = Arc::clone(&cache);
        handles.push(tokio::spawn(async move {
            let verifier =
                sample_address(u8::try_from(task_id % usize::from(VERIFIER_SPACE)).unwrap());
            for probe in 0..PROBES {
                let key_digest = digest(u8::try_from(probe).unwrap() % KEY_SPACE);
                let result = (probe + task_id) % 2 == 0;
                cache.put(verifier.clone(), key_digest, result);
                let _ = cache.get(verifier.clone(), key_digest);
            }
        }));
    }

    let join = tokio::time::timeout(Duration::from_secs(10), async {
        for handle in handles {
            handle.await.expect("hammer task must not panic");
        }
    })
    .await;
    assert!(
        join.is_ok(),
        "concurrent hammer must finish within the 10-second timeout",
    );

    // Final-value observability: every (verifier, digest) key the
    // hammer populated must be observable as `Some(_)` after all
    // racing tasks joined. Racing writes may reorder the final bool,
    // but at least one write for every key must be visible — the
    // cache's capacity bound is well above the populated key count
    // (VERIFIER_SPACE * KEY_SPACE = 32 << 4096) so no populated
    // entry can have been evicted.
    let verifiers: Vec<Address> = (0..VERIFIER_SPACE).map(sample_address).collect();
    for verifier in verifiers {
        for probe in 0..KEY_SPACE {
            assert!(
                cache.get(verifier.clone(), digest(probe)).is_some(),
                "every populated (verifier, digest) key must be observable \
                 after the concurrent hammer joins (verifier={verifier:?}, probe={probe})",
            );
        }
    }
}
