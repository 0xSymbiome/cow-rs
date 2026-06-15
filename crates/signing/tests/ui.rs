#[test]
fn eip1271_error_match_requires_wildcard() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/eip1271_error_match_requires_wildcard.rs");
}
