#![cfg(not(target_arch = "wasm32"))]

#[test]
fn public_surface_rejects_signer_on_client_and_provider_on_handle() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/trybuild/no_signer_on_client.rs");
    tests.compile_fail("tests/trybuild/no_provider_on_handle.rs");
}
