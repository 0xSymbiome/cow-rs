use cow_sdk_subgraph::{
    LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY, SubgraphQueryRequest, TOTALS_QUERY,
};
use graphql_client::GraphQLQuery;
use serde_json::json;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/schema_evidence/schema.graphql",
    query_path = "src/query_documents/totals.graphql",
    response_derives = "Debug, PartialEq"
)]
struct Totals;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/schema_evidence/schema.graphql",
    query_path = "src/query_documents/last_days_volume.graphql",
    response_derives = "Debug, PartialEq"
)]
struct LastDaysVolume;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "tests/schema_evidence/schema.graphql",
    query_path = "src/query_documents/last_hours_volume.graphql",
    response_derives = "Debug, PartialEq"
)]
struct LastHoursVolume;

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
    assert_eq!(
        TOTALS_QUERY,
        include_str!("../src/query_documents/totals.graphql")
    );
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
    assert_eq!(
        LAST_DAYS_VOLUME_QUERY,
        include_str!("../src/query_documents/last_days_volume.graphql")
    );
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
    assert_eq!(
        LAST_HOURS_VOLUME_QUERY,
        include_str!("../src/query_documents/last_hours_volume.graphql")
    );
    assert!(LAST_HOURS_VOLUME_QUERY.starts_with("query LastHoursVolume"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("$hours: Int!"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("hourlyTotals"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("orderBy: timestamp"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("orderDirection: desc"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("first: $hours"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("timestamp"));
    assert!(LAST_HOURS_VOLUME_QUERY.contains("volumeUsd"));
}

#[test]
fn subgraph_query_request_keeps_document_variables_and_operation_name_explicit() {
    let request = SubgraphQueryRequest::new(
        "query TokensByVolume($limit: Int!) { tokens(first: $limit) { symbol } }",
    )
    .with_variables(json!({ "limit": 5 }))
    .with_operation_name("TokensByVolume");

    assert_eq!(
        request.document(),
        "query TokensByVolume($limit: Int!) { tokens(first: $limit) { symbol } }"
    );
    assert_eq!(request.variables(), Some(&json!({ "limit": 5 })));
    assert_eq!(request.operation_name(), Some("TokensByVolume"));
}

#[test]
fn subgraph_query_request_from_plain_document_keeps_operation_name_absent() {
    let request = SubgraphQueryRequest::from("{ totals { orders } }");

    assert_eq!(request.document(), "{ totals { orders } }");
    assert_eq!(request.variables(), None);
    assert_eq!(request.operation_name(), None);
}

#[test]
fn totals_saved_query_document_builds_typed_test_only_request_body() {
    let request_body = Totals::build_query(totals::Variables);

    assert_eq!(request_body.query, TOTALS_QUERY);
    assert_eq!(request_body.operation_name, "Totals");
}

#[test]
fn last_days_saved_query_document_builds_typed_test_only_request_body() {
    let request_body = LastDaysVolume::build_query(last_days_volume::Variables { days: 7 });

    assert_eq!(request_body.query, LAST_DAYS_VOLUME_QUERY);
    assert_eq!(request_body.operation_name, "LastDaysVolume");
    assert_eq!(request_body.variables.days, 7);
}

#[test]
fn last_hours_saved_query_document_builds_typed_test_only_request_body() {
    let request_body = LastHoursVolume::build_query(last_hours_volume::Variables { hours: 24 });

    assert_eq!(request_body.query, LAST_HOURS_VOLUME_QUERY);
    assert_eq!(request_body.operation_name, "LastHoursVolume");
    assert_eq!(request_body.variables.hours, 24);
}
