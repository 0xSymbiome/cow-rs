#![cfg(target_arch = "wasm32")]

use cow_sdk_core::Address;
use cow_sdk_signing::{Eip1271VerificationCache, InMemoryEip1271VerificationCache};
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

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
