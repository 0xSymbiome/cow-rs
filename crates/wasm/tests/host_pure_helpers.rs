#![cfg(not(target_arch = "wasm32"))]
//! Host-target smoke test for the `cow-sdk-wasm` scaffolding.
//!
//! The scaffolding commit ships with empty `pure/` and `exports/`
//! module skeletons; this smoke test asserts that the crate
//! compiles cleanly for the native target so the host gate has a
//! concrete `cargo test -p cow-sdk-wasm --test host_pure_helpers`
//! command to run. The full host pure-helper coverage lands
//! alongside the public surface in a follow-up commit.

#[test]
fn crate_compiles_on_host() {
    // Compiling the crate for the native target with the smoke
    // test entry point is the assertion. The body is intentionally
    // empty: any `pure/` regression that breaks the host build
    // surfaces here as a compile failure.
}
