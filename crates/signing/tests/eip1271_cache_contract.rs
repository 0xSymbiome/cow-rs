#![cfg(all(not(target_arch = "wasm32"), feature = "in-memory-cache"))]
#![allow(
    clippy::too_many_lines,
    reason = "the cache contract suite intentionally keeps related scenarios in one file"
)]
//! Public-surface contract assertions for the EIP-1271 verification
//! cache trait and the default implementations shipped from the signing
//! crate.
//!
//! `Noop` asserts an always-miss / never-record contract; `InMemory`
//! asserts TTL-bounded retention, capacity-bounded eviction,
//! signature-keyed isolation (the M1 regression), positive-only
//! recording, and `Send + Sync` thread-safety under concurrent
//! record-and-observe load. The end-to-end tests drive
//! `verify_eip1271_signature_cached` against a mock provider to prove the
//! verify helper folds the signature into the cache key and never serves
//! a cached verdict for a different signature on the same digest.

use std::{
    cell::RefCell,
    fmt,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use cow_sdk_contracts::{
    ContractsError, Eip1271VerificationRequest, verify_eip1271_signature_cached,
};
use cow_sdk_core::{
    Address, BlockInfo, ContractCall, ContractHandle, Hash32, HexData, Provider,
    TransactionReceipt, TransactionRequest,
};
use cow_sdk_signing::cache::{
    Clock, DEFAULT_EIP1271_VERIFICATION_CACHE_CAPACITY, DEFAULT_EIP1271_VERIFICATION_CACHE_TTL,
    Eip1271VerificationCache, InMemoryEip1271VerificationCache, NoopEip1271VerificationCache,
};

const VALID_MAGIC: &str = "\"0x1626ba7e\"";
const INVALID_MAGIC: &str = "\"0xffffffff\"";

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

const fn sig_hash(seed: u8) -> [u8; 32] {
    [seed; 32]
}

// --- Direct cache-trait contract -------------------------------------------

#[test]
fn noop_cache_always_misses_and_never_records() {
    let cache = NoopEip1271VerificationCache;
    let verifier = sample_address(0);

    assert!(!cache.contains_valid(verifier, digest(1), sig_hash(1)));
    cache.record_valid(verifier, digest(1), sig_hash(1));
    assert!(!cache.contains_valid(verifier, digest(1), sig_hash(1)));
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
fn in_memory_cache_records_and_observes_a_valid_probe() {
    let cache = InMemoryEip1271VerificationCache::default();
    let verifier = sample_address(1);

    assert!(!cache.contains_valid(verifier, digest(7), sig_hash(7)));
    cache.record_valid(verifier, digest(7), sig_hash(7));
    assert!(cache.contains_valid(verifier, digest(7), sig_hash(7)));
    assert_eq!(cache.len(), 1);

    cache.clear();
    assert!(cache.is_empty());
    assert!(!cache.contains_valid(verifier, digest(7), sig_hash(7)));
}

#[test]
fn in_memory_cache_keys_on_signature_so_distinct_signatures_do_not_alias() {
    // M1 regression at the cache level: a recorded VALID for one signature
    // must not be observable for a different signature on the same
    // (verifier, digest).
    let cache = InMemoryEip1271VerificationCache::default();
    let verifier = sample_address(2);

    cache.record_valid(verifier, digest(5), sig_hash(0xAA));
    assert!(cache.contains_valid(verifier, digest(5), sig_hash(0xAA)));
    assert!(
        !cache.contains_valid(verifier, digest(5), sig_hash(0xBB)),
        "a different signature on the same (verifier, digest) must not alias the cached entry"
    );
    assert_eq!(cache.len(), 1);
}

#[test]
fn in_memory_cache_respects_ttl_expiry() {
    let start = Instant::now();
    let clock = ManualClock::new(start);
    let cache = InMemoryEip1271VerificationCache::with_clock(
        Duration::from_millis(40),
        16,
        clock.clone(),
    );
    let verifier = sample_address(3);

    cache.record_valid(verifier, digest(3), sig_hash(3));
    assert!(cache.contains_valid(verifier, digest(3), sig_hash(3)));
    clock.set(start + Duration::from_millis(80));
    assert!(!cache.contains_valid(verifier, digest(3), sig_hash(3)));
}

#[test]
fn cache_ttl_boundary_holds_at_minus_one_and_misses_at_plus_one() {
    let start = Instant::now();
    let clock = ManualClock::new(start);
    let cache = InMemoryEip1271VerificationCache::with_clock(
        Duration::from_secs(5 * 60),
        16,
        clock.clone(),
    );
    let verifier = sample_address(8);

    cache.record_valid(verifier, digest(9), sig_hash(9));
    clock.set(start + Duration::from_secs(4 * 60 + 59) + Duration::from_millis(999));
    assert!(cache.contains_valid(verifier, digest(9), sig_hash(9)));

    clock.set(start + Duration::from_secs(5 * 60));
    assert!(
        cache.contains_valid(verifier, digest(9), sig_hash(9)),
        "entries remain valid at the exact TTL boundary",
    );

    clock.set(start + Duration::from_secs(5 * 60) + Duration::from_millis(1));
    assert!(!cache.contains_valid(verifier, digest(9), sig_hash(9)));
}

#[test]
fn in_memory_cache_evicts_oldest_entry_when_capacity_is_exceeded() {
    let start = Instant::now();
    let clock = ManualClock::new(start);
    let cache = InMemoryEip1271VerificationCache::with_clock(
        Duration::from_secs(60),
        2,
        clock.clone(),
    );
    let verifier = sample_address(3);

    cache.record_valid(verifier, digest(1), sig_hash(1));
    clock.set(start + Duration::from_millis(1));
    cache.record_valid(verifier, digest(2), sig_hash(2));
    clock.set(start + Duration::from_millis(2));
    cache.record_valid(verifier, digest(3), sig_hash(3));

    assert_eq!(cache.len(), 2);
    assert!(!cache.contains_valid(verifier, digest(1), sig_hash(1)));
    assert!(cache.contains_valid(verifier, digest(2), sig_hash(2)));
    assert!(cache.contains_valid(verifier, digest(3), sig_hash(3)));
}

// --- End-to-end through verify_eip1271_signature_cached --------------------

#[tokio::test(flavor = "current_thread")]
async fn verify_cached_does_not_serve_a_cached_valid_for_a_different_signature() {
    // M1 regression end-to-end: a recorded VALID for signature A must not be
    // returned for a different signature B on the same (verifier, digest).
    let verifier = sample_address(7);
    let provider = MagicProvider::with_response(VALID_MAGIC);
    let cache = InMemoryEip1271VerificationCache::default();
    let digest_hash = Hash32::from_bytes(digest(0x42));

    let request_a =
        Eip1271VerificationRequest::new(verifier, digest_hash, HexData::new("0xaaaa").unwrap());
    verify_eip1271_signature_cached(&provider, &request_a, &cache)
        .await
        .expect("signature A verifies and is recorded");
    assert_eq!(provider.read_calls(), 1, "first probe hits the provider");
    assert_eq!(cache.len(), 1);

    // Same verifier + digest, DIFFERENT signature, which the verifier rejects.
    provider.set_response(INVALID_MAGIC);
    let request_b =
        Eip1271VerificationRequest::new(verifier, digest_hash, HexData::new("0xbbbb").unwrap());
    let error = verify_eip1271_signature_cached(&provider, &request_b, &cache)
        .await
        .expect_err("signature B must NOT be served the cached verdict for signature A");
    assert!(matches!(
        error,
        ContractsError::Eip1271MagicValueMismatch { .. }
    ));
    assert_eq!(
        provider.read_calls(),
        2,
        "a different signature must re-hit the provider rather than alias the cached entry",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn verify_cached_never_records_a_mismatch() {
    // Positive-only: a magic-value mismatch is never recorded, so a probe that
    // later becomes valid is not blocked by a stale negative entry.
    let verifier = sample_address(6);
    let provider = MagicProvider::with_response(INVALID_MAGIC);
    let cache = InMemoryEip1271VerificationCache::default();
    let request = Eip1271VerificationRequest::new(
        verifier,
        Hash32::from_bytes(digest(0x21)),
        HexData::new("0x1234").unwrap(),
    );

    let error = verify_eip1271_signature_cached(&provider, &request, &cache)
        .await
        .expect_err("mismatch fails closed");
    assert!(matches!(
        error,
        ContractsError::Eip1271MagicValueMismatch { .. }
    ));
    assert_eq!(cache.len(), 0, "a mismatch must not be recorded");

    // The verifier now accepts the same probe; the next call must re-hit and succeed.
    provider.set_response(VALID_MAGIC);
    verify_eip1271_signature_cached(&provider, &request, &cache)
        .await
        .expect("the activated probe must re-hit the chain and verify");
    assert_eq!(
        provider.read_calls(),
        2,
        "the second probe re-hits the provider"
    );
    assert!(cache.contains_valid(verifier, digest(0x21), alloy_keccak(&[0x12, 0x34]),));
}

#[tokio::test(flavor = "current_thread")]
async fn verify_cached_replays_identical_probe_from_cache() {
    let verifier = sample_address(5);
    let provider = MagicProvider::with_response(VALID_MAGIC);
    let cache = InMemoryEip1271VerificationCache::default();
    let request = Eip1271VerificationRequest::new(
        verifier,
        Hash32::from_bytes(digest(0x33)),
        HexData::new("0xfeed").unwrap(),
    );

    for _ in 0..3 {
        verify_eip1271_signature_cached(&provider, &request, &cache)
            .await
            .expect("identical valid probe verifies");
    }
    assert_eq!(
        provider.read_calls(),
        1,
        "an identical probe is served from cache after the first hit",
    );
}

#[tokio::test(flavor = "current_thread")]
async fn cache_skips_every_non_cacheable_error_class() {
    let verifier = sample_address(7);
    let request = Eip1271VerificationRequest::new(
        verifier,
        Hash32::from_bytes(digest(0xA5)),
        HexData::new("0x1234").unwrap(),
    );

    for scenario in NonCacheableScenario::ALL {
        let provider = ScenarioProvider::new(scenario);
        let cache = InMemoryEip1271VerificationCache::default();

        let first = verify_eip1271_signature_cached(&provider, &request, &cache)
            .await
            .expect_err("non-cacheable scenario must fail closed");
        scenario.assert_expected_error(&first, &verifier);
        assert_eq!(
            cache.len(),
            0,
            "cache recorded an entry for non-cacheable error class: {scenario:?}",
        );
        let first_counts = provider.counts();

        let second = verify_eip1271_signature_cached(&provider, &request, &cache)
            .await
            .expect_err("non-cacheable scenario must fail closed again");
        scenario.assert_expected_error(&second, &verifier);
        assert_eq!(cache.len(), 0);
        let second_counts = provider.counts();
        assert!(
            second_counts.total() > first_counts.total(),
            "second call must re-hit the provider for non-cacheable error class {scenario:?}",
        );
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn in_memory_cache_is_thread_safe_under_concurrent_record_and_observe_load() {
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
                let key = u8::try_from(probe).unwrap() % KEY_SPACE;
                cache.record_valid(verifier, digest(key), sig_hash(key));
                let _ = cache.contains_valid(verifier, digest(key), sig_hash(key));
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
        "concurrent hammer must finish within the timeout"
    );

    for verifier_seed in 0..VERIFIER_SPACE {
        let verifier = sample_address(verifier_seed);
        for key in 0..KEY_SPACE {
            assert!(
                cache.contains_valid(verifier, digest(key), sig_hash(key)),
                "every recorded probe must be observable after the concurrent hammer joins",
            );
        }
    }
}

// --- Test scaffolding ------------------------------------------------------

fn alloy_keccak(bytes: &[u8]) -> [u8; 32] {
    alloy_primitives::keccak256(bytes).0
}

#[derive(Debug, Clone)]
struct ManualClock {
    now: Arc<Mutex<Instant>>,
}

impl ManualClock {
    fn new(now: Instant) -> Self {
        Self {
            now: Arc::new(Mutex::new(now)),
        }
    }

    fn set(&self, now: Instant) {
        *self.now.lock().unwrap() = now;
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Instant {
        *self.now.lock().unwrap()
    }
}

/// Provider whose `isValidSignature` response is settable, with a
/// read-contract call counter, used by the end-to-end cache tests.
#[derive(Debug)]
struct MagicProvider {
    response: RefCell<String>,
    read_calls: RefCell<usize>,
}

impl MagicProvider {
    fn with_response(response: &str) -> Self {
        Self {
            response: RefCell::new(response.to_owned()),
            read_calls: RefCell::new(0),
        }
    }

    fn set_response(&self, response: &str) {
        let mut slot = self.response.borrow_mut();
        slot.clear();
        slot.push_str(response);
    }

    fn read_calls(&self) -> usize {
        *self.read_calls.borrow()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProviderError(&'static str);

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl Provider for MagicProvider {
    type Error = ProviderError;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(1)
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        Ok(Some(HexData::new("0x6001600055").unwrap()))
    }

    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &Hash32,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(None)
    }

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x").unwrap())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x").unwrap())
    }

    async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
        *self.read_calls.borrow_mut() += 1;
        Ok(self.response.borrow().clone())
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(0, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}

#[derive(Debug, Clone, Copy)]
enum NonCacheableScenario {
    MissingCode,
    CodeProviderError,
    ReadProviderError,
    MalformedObjectResponse,
    MalformedShortMagicValue,
    MalformedNonHexMagicValue,
}

impl NonCacheableScenario {
    const ALL: [Self; 6] = [
        Self::MissingCode,
        Self::CodeProviderError,
        Self::ReadProviderError,
        Self::MalformedObjectResponse,
        Self::MalformedShortMagicValue,
        Self::MalformedNonHexMagicValue,
    ];

    fn assert_expected_error(self, error: &ContractsError, verifier: &Address) {
        match self {
            Self::MissingCode => match error {
                ContractsError::UnsupportedEip1271Verifier { verifier: got } => {
                    assert_eq!(got, verifier);
                }
                other => panic!("expected UnsupportedEip1271Verifier, got {other:?}"),
            },
            Self::CodeProviderError => match error {
                ContractsError::Eip1271Provider { operation, message } => {
                    assert_eq!(*operation, "get_code");
                    assert_eq!(message.as_inner(), "code unavailable");
                }
                other => panic!("expected get_code Eip1271Provider, got {other:?}"),
            },
            Self::ReadProviderError => match error {
                ContractsError::Eip1271Provider { operation, message } => {
                    assert_eq!(*operation, "read_contract");
                    assert_eq!(message.as_inner(), "read unavailable");
                }
                other => panic!("expected read_contract Eip1271Provider, got {other:?}"),
            },
            Self::MalformedObjectResponse
            | Self::MalformedShortMagicValue
            | Self::MalformedNonHexMagicValue => {
                assert!(
                    matches!(error, ContractsError::MalformedEip1271Response { .. }),
                    "expected MalformedEip1271Response, got {error:?}",
                );
            }
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct ProviderCounts {
    get_code: usize,
    read_contract: usize,
}

impl ProviderCounts {
    const fn total(self) -> usize {
        self.get_code + self.read_contract
    }
}

#[derive(Debug)]
struct ScenarioProvider {
    scenario: NonCacheableScenario,
    counts: RefCell<ProviderCounts>,
}

impl ScenarioProvider {
    const fn new(scenario: NonCacheableScenario) -> Self {
        Self {
            scenario,
            counts: RefCell::new(ProviderCounts {
                get_code: 0,
                read_contract: 0,
            }),
        }
    }

    fn counts(&self) -> ProviderCounts {
        *self.counts.borrow()
    }
}

impl Provider for ScenarioProvider {
    type Error = ProviderError;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(1)
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        self.counts.borrow_mut().get_code += 1;
        match self.scenario {
            NonCacheableScenario::MissingCode => Ok(None),
            NonCacheableScenario::CodeProviderError => Err(ProviderError("code unavailable")),
            NonCacheableScenario::ReadProviderError
            | NonCacheableScenario::MalformedObjectResponse
            | NonCacheableScenario::MalformedShortMagicValue
            | NonCacheableScenario::MalformedNonHexMagicValue => {
                Ok(Some(HexData::new("0x6001600055").unwrap()))
            }
        }
    }

    async fn get_transaction_receipt(
        &self,
        _transaction_hash: &Hash32,
    ) -> Result<Option<TransactionReceipt>, Self::Error> {
        Ok(None)
    }

    async fn get_storage_at(
        &self,
        _address: &Address,
        _slot: &str,
    ) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x").unwrap())
    }

    async fn call(&self, _tx: &TransactionRequest) -> Result<HexData, Self::Error> {
        Ok(HexData::new("0x").unwrap())
    }

    async fn read_contract(&self, _request: &ContractCall) -> Result<String, Self::Error> {
        self.counts.borrow_mut().read_contract += 1;
        match self.scenario {
            NonCacheableScenario::ReadProviderError => Err(ProviderError("read unavailable")),
            NonCacheableScenario::MalformedObjectResponse => Ok("{\"unexpected\":true}".to_owned()),
            NonCacheableScenario::MalformedShortMagicValue => Ok("\"0x1234\"".to_owned()),
            NonCacheableScenario::MalformedNonHexMagicValue => Ok("\"0xzzzzzzzz\"".to_owned()),
            NonCacheableScenario::MissingCode | NonCacheableScenario::CodeProviderError => {
                Ok("\"0x1626ba7e\"".to_owned())
            }
        }
    }

    async fn get_block(&self, _block_tag: &str) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo::new(0, None))
    }

    async fn get_contract(
        &self,
        address: &Address,
        abi_json: &str,
    ) -> Result<ContractHandle, Self::Error> {
        Ok(ContractHandle::new(*address, abi_json.to_owned()))
    }
}
