#[test]
fn trading_fixture_contract_is_pinned() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../parity/fixtures/trading.json")).unwrap();

    assert_eq!(fixture["surface"].as_str().unwrap(), "trading");
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "trading-sell-order-amount-adjustment")
    );
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "trading-quote-app-data-enrichment")
    );
}
