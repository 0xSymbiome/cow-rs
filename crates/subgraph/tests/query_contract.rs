use cow_sdk_subgraph::{LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY, TOTALS_QUERY};

#[test]
fn parity_fixture_surface_and_cases_are_present() {
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../../parity/fixtures/subgraph.json")).unwrap();

    assert_eq!(fixture["surface"].as_str(), Some("subgraph"));
    assert_eq!(fixture["schema_version"].as_u64(), Some(1));
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "subgraph-last-hours-volume-query-contract")
    );
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "subgraph-custom-query-support")
    );
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "subgraph-empty-totals-error")
    );
}

#[test]
fn totals_query_matches_required_operation_and_fields() {
    assert!(TOTALS_QUERY.starts_with("query Totals"));
    for field in [
        "totals",
        "tokens",
        "orders",
        "traders",
        "settlements",
        "volumeUsd",
        "volumeEth",
        "feesUsd",
        "feesEth",
    ] {
        assert!(TOTALS_QUERY.contains(field), "missing field {field}");
    }
}

#[test]
fn last_days_query_matches_required_operation_and_variable_contract() {
    assert!(LAST_DAYS_VOLUME_QUERY.starts_with("query LastDaysVolume"));
    assert!(LAST_DAYS_VOLUME_QUERY.contains("$days: Int!"));
    assert!(LAST_DAYS_VOLUME_QUERY.contains("dailyTotals"));
    assert!(LAST_DAYS_VOLUME_QUERY.contains("orderBy: timestamp"));
    assert!(LAST_DAYS_VOLUME_QUERY.contains("orderDirection: desc"));
    assert!(LAST_DAYS_VOLUME_QUERY.contains("first: $days"));
    assert!(LAST_DAYS_VOLUME_QUERY.contains("timestamp"));
    assert!(LAST_DAYS_VOLUME_QUERY.contains("volumeUsd"));
}

#[test]
fn last_hours_query_matches_required_operation_and_variable_contract() {
    assert!(LAST_HOURS_VOLUME_QUERY.starts_with("query LastHoursVolume"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("$hours: Int!"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("hourlyTotals"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("orderBy: timestamp"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("orderDirection: desc"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("first: $hours"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("timestamp"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("volumeUsd"));
}
