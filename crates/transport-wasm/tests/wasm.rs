#[cfg(target_arch = "wasm32")]
#[path = "wasm/fetch_smoke.rs"]
mod fetch_smoke;

#[cfg(not(target_arch = "wasm32"))]
#[test]
fn wasm_smoke_tests_are_wasm32_only() {
    // The browser fetch transport is intentionally unavailable on host targets.
}
