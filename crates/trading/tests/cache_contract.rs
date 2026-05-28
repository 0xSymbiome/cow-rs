mod common;

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use cow_sdk_core::{BuyTokenDestination, CowEnv, OrderKind, SellTokenSource, SupportedChainId};
use cow_sdk_trading::{
    Clock, DEFAULT_QUOTE_CACHE_CAPACITY, DEFAULT_QUOTE_CACHE_TTL, InMemoryQuoteCache,
    NoopQuoteCache, QuoteCache, QuoteCacheKey, QuoteResults, TradingSdkBuilder, get_quote_results,
};

use crate::common::{
    MockOrderbook, MockSigner, address, sample_trade_parameters, sell_quote_response,
};

async fn sample_quote_results() -> QuoteResults {
    let orderbook = MockOrderbook::new(SupportedChainId::Sepolia, sell_quote_response());
    let signer = MockSigner::default();
    let trader = cow_sdk_trading::TraderParameters::new(SupportedChainId::Sepolia, "cache-test")
        .expect("app code should validate")
        .with_env(CowEnv::Prod);
    let trade = sample_trade_parameters(OrderKind::Sell);

    get_quote_results(&trade, &trader, &signer, None, &orderbook)
        .await
        .expect("mock orderbook must return a quote result for the cache test fixture")
}

fn sample_cache_key() -> QuoteCacheKey {
    QuoteCacheKey::new(
        u64::from(SupportedChainId::Sepolia),
        "prod",
        &address(common::WETH),
        &address(common::COW),
        "1000000000000000000",
        OrderKind::Sell,
    )
    .with_owner(&address(common::OWNER))
    .with_token_balances(SellTokenSource::Erc20, BuyTokenDestination::Erc20)
}

#[tokio::test]
async fn noop_quote_cache_is_pass_through_for_every_operation() {
    let cache = NoopQuoteCache::new();
    let key = sample_cache_key();
    let value = sample_quote_results().await;

    assert!(
        cache.lookup(&key).await.is_none(),
        "pass-through cache must miss on an empty lookup"
    );

    cache.insert(key.clone(), value.clone()).await;
    assert!(
        cache.lookup(&key).await.is_none(),
        "pass-through cache must never retain inserted entries"
    );

    cache.invalidate(&key).await;
    assert!(
        cache.lookup(&key).await.is_none(),
        "pass-through cache must remain a miss after invalidate"
    );
}

#[tokio::test]
async fn in_memory_cache_returns_hit_within_the_configured_ttl() {
    let cache = InMemoryQuoteCache::new(Duration::from_secs(30), 16);
    let key = sample_cache_key();
    let value = sample_quote_results().await;

    cache.insert(key.clone(), value.clone()).await;
    let hit = cache
        .lookup(&key)
        .await
        .expect("an entry inserted within the TTL must hit on lookup");
    assert_eq!(hit, value, "hit value must equal the inserted payload");
}

#[tokio::test]
async fn in_memory_cache_reports_miss_after_ttl_has_elapsed() {
    let cache = InMemoryQuoteCache::new(Duration::from_millis(1), 16);
    let key = sample_cache_key();
    let value = sample_quote_results().await;

    cache.insert(key.clone(), value).await;
    tokio::time::sleep(Duration::from_millis(20)).await;

    assert!(
        cache.lookup(&key).await.is_none(),
        "TTL expiry must evict the stored entry on the next lookup"
    );
    assert!(
        cache.lookup(&key).await.is_none(),
        "repeated lookup after TTL expiry must remain a miss"
    );
}

#[tokio::test]
async fn in_memory_cache_invalidate_removes_the_stored_entry() {
    let cache = InMemoryQuoteCache::new(Duration::from_secs(30), 16);
    let key = sample_cache_key();
    let value = sample_quote_results().await;

    cache.insert(key.clone(), value).await;
    assert!(cache.lookup(&key).await.is_some());

    cache.invalidate(&key).await;
    assert!(
        cache.lookup(&key).await.is_none(),
        "explicit invalidate must drop the entry even before TTL"
    );
}

#[tokio::test]
async fn quote_cache_keys_match_across_instances_built_from_the_same_inputs() {
    let first = sample_cache_key();
    let second = sample_cache_key();
    assert_eq!(
        first, second,
        "identical inputs must produce identical cache keys across builders"
    );

    let checksum_sell = address("0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14");
    let lowercase_sell = address("0xfff9976782d46cc05630d1f6ebab18b2324d6b14");
    let checksum_variant = QuoteCacheKey::new(
        u64::from(SupportedChainId::Sepolia),
        "prod",
        &checksum_sell,
        &address(common::COW),
        "1000000000000000000",
        OrderKind::Sell,
    );
    let lowercase_variant = QuoteCacheKey::new(
        u64::from(SupportedChainId::Sepolia),
        "prod",
        &lowercase_sell,
        &address(common::COW),
        "1000000000000000000",
        OrderKind::Sell,
    );
    assert_eq!(
        checksum_variant, lowercase_variant,
        "address case variants must produce identical cache keys"
    );
}

#[tokio::test]
async fn trading_sdk_builder_accepts_an_injected_quote_cache_instance() {
    let cache: Arc<dyn QuoteCache> = Arc::new(InMemoryQuoteCache::new(Duration::from_secs(60), 16));
    let sdk = TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Sepolia)
        .with_app_code("cache-injection")
        .with_quote_cache(cache.clone())
        .build_ready()
        .expect("builder configured with a quote cache must build successfully");

    let configured = sdk
        .options()
        .quote_cache()
        .expect("injected cache must be retrievable through TradingSdk::options");
    assert!(
        Arc::ptr_eq(&configured, &cache),
        "retrieved cache must reference the same injected Arc"
    );
}

#[tokio::test]
async fn in_memory_quote_cache_default_ttl_and_capacity_match_documented_constants() {
    let cache = InMemoryQuoteCache::default();
    assert_eq!(cache.ttl(), DEFAULT_QUOTE_CACHE_TTL);
    assert_eq!(cache.capacity(), DEFAULT_QUOTE_CACHE_CAPACITY);
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[tokio::test]
async fn in_memory_quote_cache_reports_population_and_clear_state() {
    let cache = InMemoryQuoteCache::default();
    let key = sample_cache_key();
    let value = sample_quote_results().await;

    cache.insert(key.clone(), value.clone()).await;
    assert!(!cache.is_empty());
    assert_eq!(cache.len(), 1);
    assert_eq!(cache.lookup(&key).await, Some(value));

    cache.clear();
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
    assert!(cache.lookup(&key).await.is_none());
}

#[tokio::test]
async fn in_memory_quote_cache_evicts_oldest_entry_when_capacity_is_exceeded() {
    let cache = InMemoryQuoteCache::new(Duration::from_secs(60), 2);
    let value = sample_quote_results().await;

    let key_a = sample_cache_key().with_valid_to(1);
    let key_b = sample_cache_key().with_valid_to(2);
    let key_c = sample_cache_key().with_valid_to(3);

    cache.insert(key_a.clone(), value.clone()).await;
    tokio::time::sleep(Duration::from_millis(2)).await;
    cache.insert(key_b.clone(), value.clone()).await;
    tokio::time::sleep(Duration::from_millis(2)).await;
    cache.insert(key_c.clone(), value.clone()).await;

    assert_eq!(cache.len(), 2);
    assert!(
        cache.lookup(&key_a).await.is_none(),
        "oldest entry must be evicted when capacity is exceeded",
    );
    assert!(
        cache.lookup(&key_b).await.is_some(),
        "second-oldest entry must remain after the capacity-triggered eviction",
    );
    assert!(
        cache.lookup(&key_c).await.is_some(),
        "newest entry must remain after the capacity-triggered eviction",
    );
}

#[tokio::test]
async fn quote_cache_ttl_boundary_holds_at_minus_one_and_misses_at_plus_one() {
    let start = Instant::now();
    let clock = ManualClock::new(start);
    let cache = InMemoryQuoteCache::with_clock(Duration::from_secs(5 * 60), 16, clock.clone());
    let key = sample_cache_key();
    let value = sample_quote_results().await;

    cache.insert(key.clone(), value.clone()).await;

    clock.set(start + Duration::from_secs(4 * 60 + 59) + Duration::from_millis(999));
    assert_eq!(
        cache.lookup(&key).await,
        Some(value.clone()),
        "cache entries remain valid one millisecond before the TTL boundary",
    );

    clock.set(start + Duration::from_secs(5 * 60));
    assert_eq!(
        cache.lookup(&key).await,
        Some(value.clone()),
        "cache entries remain valid at the exact TTL boundary",
    );

    clock.set(start + Duration::from_secs(5 * 60) + Duration::from_millis(1));
    assert!(
        cache.lookup(&key).await.is_none(),
        "cache entries expire one millisecond after the TTL boundary",
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn in_memory_quote_cache_is_thread_safe_under_concurrent_probe_and_populate_load() {
    const TASKS: usize = 64;
    const PROBES: usize = 256;
    const KEY_SEED_SPACE: u32 = 8;
    const VARIANT_SPACE: u32 = 4;

    let cache = Arc::new(InMemoryQuoteCache::new(Duration::from_secs(60), 4096));
    let value = sample_quote_results().await;

    let mut handles = Vec::with_capacity(TASKS);
    for task_id in 0..TASKS {
        let cache = Arc::clone(&cache);
        let value = value.clone();
        handles.push(tokio::spawn(async move {
            for probe in 0..PROBES {
                let seed = u32::try_from(probe).unwrap() % KEY_SEED_SPACE;
                let variant = u32::try_from(task_id).unwrap() % VARIANT_SPACE;
                let key = sample_cache_key()
                    .with_valid_to(seed)
                    .with_partially_fillable(variant.is_multiple_of(2));
                cache.insert(key.clone(), value.clone()).await;
                let _ = cache.lookup(&key).await;
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

    // Final-value observability: every (valid_to, partially_fillable) key the
    // hammer populated must be observable as `Some(_)` after all racing tasks
    // joined. The capacity bound (4096) is well above the populated key count
    // (KEY_SEED_SPACE * VARIANT_SPACE = 32), so no populated entry can have
    // been evicted by the capacity scan.
    for seed in 0..KEY_SEED_SPACE {
        for variant in 0..VARIANT_SPACE {
            let key = sample_cache_key()
                .with_valid_to(seed)
                .with_partially_fillable(variant.is_multiple_of(2));
            assert!(
                cache.lookup(&key).await.is_some(),
                "every populated key must be observable after the concurrent hammer joins \
                 (seed={seed}, variant={variant})",
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
