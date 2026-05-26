#![cfg(target_arch = "wasm32")]

mod common;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use cow_sdk_core::{BuyTokenDestination, CowEnv, OrderKind, SellTokenSource, SupportedChainId};
use cow_sdk_trading::{
    Clock, InMemoryQuoteCache, QuoteCache, QuoteCacheKey, QuoteResults, get_quote_results,
};
use wasm_bindgen_test::wasm_bindgen_test;
use web_time::Instant;

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

#[wasm_bindgen_test]
async fn in_memory_quote_cache_round_trips_without_panicking_on_wasm32() {
    let cache = InMemoryQuoteCache::default();
    let key = sample_cache_key();
    let value = sample_quote_results().await;

    assert!(cache.lookup(&key).await.is_none());
    cache.insert(key.clone(), value.clone()).await;
    assert_eq!(cache.lookup(&key).await, Some(value));
}

#[wasm_bindgen_test]
async fn quote_cache_ttl_boundary_holds_at_minus_one_and_misses_at_plus_one_on_wasm32() {
    let start = Instant::now();
    let clock = ManualClock::new(start);
    let cache = InMemoryQuoteCache::with_clock(Duration::from_secs(5 * 60), 16, clock.clone());
    let key = sample_cache_key();
    let value = sample_quote_results().await;

    cache.insert(key.clone(), value.clone()).await;

    clock.set(start + Duration::from_secs(4 * 60 + 59) + Duration::from_millis(999));
    assert_eq!(cache.lookup(&key).await, Some(value));

    clock.set(start + Duration::from_secs(5 * 60) + Duration::from_millis(1));
    assert!(cache.lookup(&key).await.is_none());
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
