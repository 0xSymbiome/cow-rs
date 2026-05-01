#![cfg(target_arch = "wasm32")]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use cow_sdk_core::Address;
use cow_sdk_signing::{Clock, Eip1271VerificationCache, InMemoryEip1271VerificationCache};
use wasm_bindgen_test::wasm_bindgen_test;
use web_time::Instant;

#[wasm_bindgen_test]
fn in_memory_cache_round_trips_without_panicking_on_wasm32() {
    let cache = InMemoryEip1271VerificationCache::default();
    let verifier = Address::new("0x1111111111111111111111111111111111111111")
        .expect("static verifier must stay valid");
    let digest = [0xAB; 32];

    assert_eq!(cache.get(verifier.clone(), digest), None);
    cache.put(verifier.clone(), digest, true);
    assert_eq!(cache.get(verifier, digest), Some(true));
}

#[wasm_bindgen_test]
fn cache_ttl_boundary_holds_at_minus_one_and_misses_at_plus_one_on_wasm32() {
    let start = Instant::now();
    let clock = ManualClock::new(start);
    let cache = InMemoryEip1271VerificationCache::with_clock(
        Duration::from_secs(5 * 60),
        16,
        clock.clone(),
    );
    let verifier = Address::new("0x2222222222222222222222222222222222222222")
        .expect("static verifier must stay valid");
    let digest = [0xCD; 32];

    cache.put(verifier.clone(), digest, true);
    clock.set(start + Duration::from_secs(4 * 60 + 59) + Duration::from_millis(999));
    assert_eq!(cache.get(verifier.clone(), digest), Some(true));

    clock.set(start + Duration::from_secs(5 * 60) + Duration::from_millis(1));
    assert_eq!(cache.get(verifier, digest), None);
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
