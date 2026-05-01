#[test]
fn orderbook_client_reachable_through_trading_re_export() {
    let cases = trybuild::TestCases::new();
    cases.pass("tests/ui/orderbook_client_reachable_through_trading_re_export.rs");
}
