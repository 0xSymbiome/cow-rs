#[test]
fn trading_sdk_inherent_constructors_stay_absent() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/trading_sdk_no_free_constructors.rs");
    cases.compile_fail("tests/ui/helper_only_sdk_no_quote_methods.rs");
    cases.compile_fail("tests/ui/helper_only_sdk_no_offchain_cancel.rs");
}

#[test]
fn client_rejection_external_match_requires_wildcard() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/client_rejection_external_match_requires_wildcard.rs");
}
