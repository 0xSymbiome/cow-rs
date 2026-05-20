#![cfg(not(target_arch = "wasm32"))]
#![allow(
    clippy::match_like_matches_macro,
    clippy::match_same_arms,
    clippy::too_many_lines,
    reason = "the cache matrix intentionally mirrors every current error variant"
)]
//! Public-surface contract assertions for the EIP-1271 verification
//! cache trait and the two default implementations shipped from the
//! signing crate.
//!
//! The `Noop` impl asserts an always-miss contract; the `InMemory`
//! impl asserts TTL-bounded retention, capacity-bounded eviction,
//! non-cacheable verifier errors, controlled-clock boundaries, and
//! `Send + Sync` thread-safety under concurrent probe-and-populate
//! load.

use std::{
    cell::RefCell,
    fmt,
    sync::{Arc, Mutex},
    thread::sleep,
    time::{Duration, Instant},
};

use cow_sdk_contracts::{
    ContractsError, Eip1271VerificationRequest, verify_eip1271_signature_async,
};
use cow_sdk_core::{
    Address, AsyncProvider, BlockInfo, ContractCall, ContractHandle, Hash32, HexData,
    TransactionReceipt, TransactionRequest,
};
use cow_sdk_signing::cache::{
    Clock, DEFAULT_EIP1271_VERIFICATION_CACHE_CAPACITY, DEFAULT_EIP1271_VERIFICATION_CACHE_TTL,
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

    assert_eq!(cache.get(verifier, key_digest), None);
    cache.put(verifier, key_digest, true);
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
fn in_memory_cache_reports_population_and_clear_state() {
    let cache = InMemoryEip1271VerificationCache::default();
    let verifier = sample_address(4);
    let key_digest = digest(4);

    cache.put(verifier, key_digest, true);
    assert!(!cache.is_empty());
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.get(verifier, key_digest), Some(true));

    cache.clear();
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
    assert_eq!(cache.get(verifier, key_digest), None);
}

#[test]
fn in_memory_cache_round_trips_a_recorded_outcome() {
    let cache = InMemoryEip1271VerificationCache::default();
    let verifier = sample_address(1);
    let key_digest = digest(7);

    assert_eq!(cache.get(verifier, key_digest), None);
    cache.put(verifier, key_digest, true);
    assert_eq!(cache.get(verifier, key_digest), Some(true));
    cache.put(verifier, key_digest, false);
    assert_eq!(cache.get(verifier, key_digest), Some(false));
}

#[test]
fn in_memory_cache_respects_ttl_expiry() {
    let cache = InMemoryEip1271VerificationCache::new(Duration::from_millis(40), 16);
    let verifier = sample_address(2);
    let key_digest = digest(3);

    cache.put(verifier, key_digest, true);
    assert_eq!(cache.get(verifier, key_digest), Some(true));
    sleep(Duration::from_millis(80));
    assert_eq!(cache.get(verifier, key_digest), None);
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
    let key_digest = digest(9);

    cache.put(verifier, key_digest, true);
    clock.set(start + Duration::from_secs(4 * 60 + 59) + Duration::from_millis(999));
    assert_eq!(cache.get(verifier, key_digest), Some(true));

    clock.set(start + Duration::from_secs(5 * 60));
    assert_eq!(
        cache.get(verifier, key_digest),
        Some(true),
        "cache entries remain valid at the exact TTL boundary",
    );

    clock.set(start + Duration::from_secs(5 * 60) + Duration::from_millis(1));
    assert_eq!(cache.get(verifier, key_digest), None);
}

#[test]
fn in_memory_cache_evicts_oldest_entry_when_capacity_is_exceeded() {
    let cache = InMemoryEip1271VerificationCache::new(Duration::from_secs(60), 2);
    let verifier = sample_address(3);

    cache.put(verifier, digest(1), true);
    sleep(Duration::from_millis(2));
    cache.put(verifier, digest(2), true);
    sleep(Duration::from_millis(2));
    cache.put(verifier, digest(3), true);

    assert_eq!(cache.len(), 2);
    assert_eq!(cache.get(verifier, digest(1)), None);
    assert_eq!(cache.get(verifier, digest(2)), Some(true));
    assert_eq!(cache.get(verifier, digest(3)), Some(true));
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

        let first = verify_eip1271_signature_async(&provider, &request, &cache)
            .await
            .expect_err("non-cacheable scenario must fail closed");
        assert_known_non_cacheable_error(&first);
        scenario.assert_expected_error(&first, &verifier);
        assert_eq!(
            cache.len(),
            0,
            "cache stored entry for non-cacheable error class: {scenario:?}",
        );
        let first_counts = provider.counts();

        let second = verify_eip1271_signature_async(&provider, &request, &cache)
            .await
            .expect_err("non-cacheable scenario must fail closed again");
        assert_known_non_cacheable_error(&second);
        scenario.assert_expected_error(&second, &verifier);
        assert_eq!(
            cache.len(),
            0,
            "cache state changed after repeated non-cacheable error class: {scenario:?}",
        );
        let second_counts = provider.counts();
        assert!(
            second_counts.total() > first_counts.total(),
            "second call must re-hit the provider for non-cacheable error class \
             {scenario:?} (first={first_counts:?}, second={second_counts:?})",
        );
    }
}

#[tokio::test(flavor = "current_thread")]
async fn eip1271_eoa_verifier_does_not_cache() {
    let verifier = sample_address(6);
    let request = Eip1271VerificationRequest::new(
        verifier,
        Hash32::from_bytes(digest(0x66)),
        HexData::new("0x1234").unwrap(),
    );
    let provider = ScenarioProvider::new(NonCacheableScenario::MissingCode);
    let cache = InMemoryEip1271VerificationCache::default();

    for attempt in 1..=2 {
        let error = verify_eip1271_signature_async(&provider, &request, &cache)
            .await
            .expect_err("EOA verifier must fail before signature verification");
        assert!(matches!(
            error,
            ContractsError::UnsupportedEip1271Verifier { verifier: ref got }
                if got == &verifier
        ));
        assert_eq!(
            cache.len(),
            0,
            "EOA verifier attempt {attempt} must not populate the verification cache",
        );
        assert_eq!(
            provider.counts().get_code,
            attempt,
            "EOA verifier attempt {attempt} must re-hit provider code lookup",
        );
    }
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
                cache.put(verifier, key_digest, result);
                let _ = cache.get(verifier, key_digest);
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
                cache.get(verifier, digest(probe)).is_some(),
                "every populated (verifier, digest) key must be observable \
                 after the concurrent hammer joins (verifier={verifier:?}, probe={probe})",
            );
        }
    }
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

fn assert_known_non_cacheable_error(error: &ContractsError) {
    assert!(
        current_contracts_error_is_non_cacheable(error),
        "cache matrix unexpectedly treated error as cacheable: {error:?}",
    );
}

const fn current_contracts_error_is_non_cacheable(error: &ContractsError) -> bool {
    match error {
        ContractsError::Eip1271MagicValueMismatch { .. } => false,
        ContractsError::Core(_)
        | ContractsError::Cancelled
        | ContractsError::UnsupportedChain(_)
        | ContractsError::InvalidOrderUidLength { .. }
        | ContractsError::InvalidNumeric { .. }
        | ContractsError::NumericOverflow { .. }
        | ContractsError::InvalidFlags(_)
        | ContractsError::UnsupportedSigningScheme(_)
        | ContractsError::InvalidEip1271SignatureData
        | ContractsError::UnsupportedEip1271Verifier { .. }
        | ContractsError::Eip1271Provider { .. }
        | ContractsError::MalformedEip1271Response { .. }
        | ContractsError::MissingClearingPrice { .. }
        | ContractsError::MissingExecutedAmount
        | ContractsError::MissingTrade
        | ContractsError::ZeroReceiver
        | ContractsError::InvalidTokenIndex { .. }
        | ContractsError::ForbiddenInteractionTarget { .. }
        | ContractsError::Provider { .. }
        | ContractsError::Abi(_)
        | ContractsError::DecodeHex { .. }
        | ContractsError::InvalidHexPrefix { .. }
        | ContractsError::InvalidDecodedLength { .. }
        | ContractsError::Serialization(_)
        | ContractsError::InvalidSignatureLength { .. }
        | ContractsError::InvalidSignatureRecoveryByte { .. }
        | ContractsError::SignatureSchemeNotEcdsa
        | ContractsError::SignatureRecovery { .. } => true,
        _ => true,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ScenarioProviderError(&'static str);

impl fmt::Display for ScenarioProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
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

impl AsyncProvider for ScenarioProvider {
    type Error = ScenarioProviderError;

    async fn get_chain_id(&self) -> Result<u64, Self::Error> {
        Ok(1)
    }

    async fn get_code(&self, _address: &Address) -> Result<Option<HexData>, Self::Error> {
        self.counts.borrow_mut().get_code += 1;
        match self.scenario {
            NonCacheableScenario::MissingCode => Ok(None),
            NonCacheableScenario::CodeProviderError => {
                Err(ScenarioProviderError("code unavailable"))
            }
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
            NonCacheableScenario::ReadProviderError => {
                Err(ScenarioProviderError("read unavailable"))
            }
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
