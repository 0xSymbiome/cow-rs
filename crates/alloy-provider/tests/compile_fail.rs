#[test]
fn negative_impls_and_sealed_markers_hold() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/trybuild/no_signing_provider.rs");
    tests.compile_fail("tests/trybuild/no_async_signer.rs");
    tests.compile_fail("tests/trybuild/no_sync_signer.rs");
    tests.compile_fail("tests/trybuild/external_marker_construction_fails.rs");
}
