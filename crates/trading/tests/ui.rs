#[test]
fn trading_sdk_inherent_constructors_stay_absent() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/trading_sdk_no_free_constructors.rs");
}
