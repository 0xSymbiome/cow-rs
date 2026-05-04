mod common;

use std::sync::Arc;
use std::time::Duration;

use cow_sdk_core::{BuyTokenDestination, CowEnv, OrderKind, SellTokenSource, SupportedChainId};
use cow_sdk_trading::{
    InMemoryQuoteCache, NoopQuoteCache, QuoteCache, QuoteCacheKey, QuoteResults, TradingSdkBuilder,
    get_quote_results,
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
    let cache = InMemoryQuoteCache::new(Duration::from_secs(30));
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
    let cache = InMemoryQuoteCache::new(Duration::from_millis(1));
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
    let cache = InMemoryQuoteCache::new(Duration::from_secs(30));
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
    let cache: Arc<dyn QuoteCache> = Arc::new(InMemoryQuoteCache::new(Duration::from_secs(60)));
    let sdk = TradingSdkBuilder::new()
        .with_chain_id(SupportedChainId::Sepolia)
        .with_app_code("cache-injection")
        .with_owner(address(common::OWNER))
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
