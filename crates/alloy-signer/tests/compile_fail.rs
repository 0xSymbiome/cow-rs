#![cfg(not(target_arch = "wasm32"))]

#[test]
fn public_surface_rejects_provider_and_sync_signer_contracts() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/trybuild/no_async_provider.rs");
    tests.compile_fail("tests/trybuild/no_async_signing_provider.rs");
    tests.compile_fail("tests/trybuild/no_sync_signer.rs");
    tests.compile_fail("tests/trybuild/external_marker_construction_fails.rs");
}
