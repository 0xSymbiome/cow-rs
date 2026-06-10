#![cfg(all(target_arch = "wasm32", feature = "in-memory-cache"))]

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use cow_sdk_core::Address;
use cow_sdk_signing::{Clock, Eip1271Cache, InMemoryEip1271Cache};
use wasm_bindgen_test::wasm_bindgen_test;
use web_time::Instant;

#[wasm_bindgen_test]
fn in_memory_cache_round_trips_without_panicking_on_wasm32() {
    let cache = InMemoryEip1271Cache::default();
    let verifier = Address::new("0x1111111111111111111111111111111111111111")
        .expect("static verifier must stay valid");
    let digest = [0xAB; 32];
    let signature_hash = [0xCD; 32];

    assert!(!cache.contains_valid(verifier, digest, signature_hash));
    cache.record_valid(verifier, digest, signature_hash);
    assert!(cache.contains_valid(verifier, digest, signature_hash));
}

#[wasm_bindgen_test]
fn cache_ttl_boundary_holds_at_minus_one_and_misses_at_plus_one_on_wasm32() {
    let start = Instant::now();
    let clock = ManualClock::new(start);
    let cache = InMemoryEip1271Cache::with_clock(Duration::from_secs(5 * 60), 16, clock.clone());
    let verifier = Address::new("0x2222222222222222222222222222222222222222")
        .expect("static verifier must stay valid");
    let digest = [0xCD; 32];
    let signature_hash = [0xEF; 32];

    cache.record_valid(verifier, digest, signature_hash);
    clock.set(start + Duration::from_secs(4 * 60 + 59) + Duration::from_millis(999));
    assert!(cache.contains_valid(verifier, digest, signature_hash));

    clock.set(start + Duration::from_secs(5 * 60) + Duration::from_millis(1));
    assert!(!cache.contains_valid(verifier, digest, signature_hash));
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
