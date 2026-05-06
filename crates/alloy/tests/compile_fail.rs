#![cfg(not(target_arch = "wasm32"))]

#[test]
fn public_surface_rejects_sync_and_provider_contracts() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/trybuild/no_sync_signer_on_client.rs");
    tests.compile_fail("tests/trybuild/no_sync_signer_on_handle.rs");
    tests.compile_fail("tests/trybuild/no_async_provider_on_handle.rs");
}
