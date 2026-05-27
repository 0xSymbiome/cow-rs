#![cfg(not(target_arch = "wasm32"))]

#[test]
fn public_surface_rejects_provider_contracts() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/trybuild/no_provider.rs");
    tests.compile_fail("tests/trybuild/no_signing_provider.rs");
    tests.compile_fail("tests/trybuild/external_marker_construction_fails.rs");
}
