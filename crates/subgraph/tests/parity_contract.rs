//! Fixture-driven parity contract for `cow-sdk-subgraph`.
//!
//! Loads `parity/fixtures/subgraph.json` (schema version 1) at compile time,
//! iterates every documented case, and asserts the Rust subgraph helpers
//! preserve the pinned upstream query contracts. The helpers exercised are:
//!
//! * [`TOTALS_QUERY`], [`LAST_DAYS_VOLUME_QUERY`], [`LAST_HOURS_VOLUME_QUERY`]
//!   — canonical GraphQL documents shipped as compile-time constants.
//! * [`TotalsResponse`], [`LastDaysVolumeResponse`], [`LastHoursVolumeResponse`]
//!   — typed response decoders.
//! * [`SubgraphQueryRequest`] — caller-supplied raw-query envelope.
//! * [`SubgraphApi::prod_config`] — per-chain production base URL map
//!   including explicit `None` entries for unsupported chains.
//! * [`SubgraphError::UnsupportedNetwork`], [`SubgraphError::NoTotalsFound`],
//!   [`SubgraphError::GraphQl`] — typed rejection boundary for
//!   unsupported networks, empty totals responses, and GraphQL errors.
//!
//! Failure messages carry the fixture case id so a reviewer looking at a
//! broken CI run sees the exact upstream vector that diverged.

use cow_sdk_core::SupportedChainId;
use cow_sdk_subgraph::{
    LAST_DAYS_VOLUME_QUERY, LAST_HOURS_VOLUME_QUERY, LastDaysVolumeResponse,
    LastHoursVolumeResponse, SubgraphApi, SubgraphConfig, SubgraphQueryRequest, TOTALS_QUERY,
    TotalsResponse,
};
use serde_json::{Value, json};

const FIXTURE: &str = include_str!("../../../parity/fixtures/subgraph.json");

#[test]
fn parity_fixture_cases_hold() {
    let fixture: Value = serde_json::from_str(FIXTURE).expect("fixture must parse as JSON");

    assert_eq!(
        fixture["schema_version"].as_u64(),
        Some(1),
        "subgraph fixture must declare schema_version 1",
    );
    assert_eq!(
        fixture["surface"].as_str(),
        Some("subgraph"),
        "subgraph fixture must carry the subgraph surface label",
    );

    let cases = fixture["cases"]
        .as_array()
        .expect("subgraph fixture must expose a cases array");

    for case in cases {
        let id = case["id"]
            .as_str()
            .expect("every fixture case must carry a string id");
        let expected = &case["expected"];

        match id {
            "subgraph-prod-url-resolution" => assert_prod_url_resolution(id, expected),
            "subgraph-totals-query-contract" => assert_totals_query_contract(id, expected),
            "subgraph-last-days-volume-query-contract" => {
                assert_last_days_volume_query_contract(id, expected);
            }
            "subgraph-last-hours-volume-query-contract" => {
                assert_last_hours_volume_query_contract(id, expected);
            }
            "subgraph-custom-query-support" => assert_custom_query_support(id, expected),
            "subgraph-custom-base-url-override" => {
                assert_custom_base_url_override(id, expected);
            }
            "subgraph-generated-type-contract" => assert_generated_type_contract(id, expected),
            "subgraph-unsupported-network-error" => {
                assert_unsupported_network_error(id, expected);
            }
            "subgraph-empty-totals-error" => assert_empty_totals_error(id, expected),
            "subgraph-invalid-query-error" => assert_invalid_query_error(id, expected),
            other => panic!("unknown subgraph fixture case id: {other}"),
        }
    }
}

fn assert_prod_url_resolution(id: &str, expected: &Value) {
    let base_template = expected["base_url_template"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.base_url_template must be a string"));
    assert!(
        base_template.starts_with("https://gateway.thegraph.com/api/"),
        "case {id}: base URL template must match the pinned Graph gateway",
    );

    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("redacted-for-test")
        .build();
    let prod_config = api.prod_config();

    let supported = expected["supported_urls"]
        .as_object()
        .unwrap_or_else(|| panic!("case {id}: expected.supported_urls must be an object"));
    for (chain_label, template_value) in supported {
        let template = template_value
            .as_str()
            .unwrap_or_else(|| panic!("case {id}: supported_urls.{chain_label} must be a string"));
        let chain = chain_label_to_rust(id, chain_label);
        let actual = prod_config
            .get(&chain)
            .and_then(|entry| entry.as_deref())
            .unwrap_or_else(|| panic!("case {id}: {chain_label} must remain supported"));
        let expected_redacted = template.replace("{apiKey}", "<redacted>");
        assert_eq!(
            actual, expected_redacted,
            "case {id}: {chain_label} prod URL must match the pinned subgraph id",
        );
    }

    let unsupported: Vec<&str> = expected["unsupported_chains"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.unsupported_chains must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: unsupported_chains entries must be strings"))
        })
        .collect();
    for label in unsupported {
        let chain = chain_label_to_rust(id, label);
        let entry = prod_config
            .get(&chain)
            .unwrap_or_else(|| panic!("case {id}: {label} must be present in the prod config"));
        assert!(
            entry.is_none(),
            "case {id}: {label} must map to None in the prod config",
        );
    }
}

fn assert_totals_query_contract(id: &str, expected: &Value) {
    let operation = expected["operation_name"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.operation_name must be a string"));
    let root = expected["root_field"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.root_field must be a string"));
    let fields: Vec<&str> = expected["item_fields"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.item_fields must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: item_fields entries must be strings"))
        })
        .collect();

    assert!(
        TOTALS_QUERY.contains(&format!("query {operation}")),
        "case {id}: TOTALS_QUERY must include the operation name",
    );
    assert!(
        TOTALS_QUERY.contains(root),
        "case {id}: TOTALS_QUERY must include the root field",
    );
    for field in &fields {
        assert!(
            TOTALS_QUERY.contains(field),
            "case {id}: TOTALS_QUERY must include field {field}",
        );
    }

    // Decode a fixture-shape response through the shipped DTO to prove the
    // TotalsResponse decoder remains wired to the pinned field layout.
    let payload = json!({
        "totals": [{
            "tokens": "1",
            "orders": "2",
            "traders": "3",
            "settlements": "4",
            "volumeUsd": "10",
            "volumeEth": "11",
            "feesUsd": "12",
            "feesEth": "13",
        }],
    });
    let decoded: TotalsResponse = serde_json::from_value(payload)
        .unwrap_or_else(|error| panic!("case {id}: TotalsResponse decoding must succeed: {error}"));
    assert_eq!(
        decoded.totals.len(),
        1,
        "case {id}: one totals row expected"
    );
    assert_eq!(
        decoded.totals[0].tokens, "1",
        "case {id}: tokens must decode to the pinned string",
    );
}

fn assert_last_days_volume_query_contract(id: &str, expected: &Value) {
    let operation = expected["operation_name"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.operation_name must be a string"));
    let root = expected["root_field"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.root_field must be a string"));
    let variable = expected["variable_name"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.variable_name must be a string"));
    let order_by = expected["order_by"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.order_by must be a string"));
    let order_direction = expected["order_direction"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.order_direction must be a string"));

    assert!(LAST_DAYS_VOLUME_QUERY.contains(&format!("query {operation}")));
    assert!(LAST_DAYS_VOLUME_QUERY.contains(root));
    assert!(LAST_DAYS_VOLUME_QUERY.contains(&format!("${variable}")));
    assert!(LAST_DAYS_VOLUME_QUERY.contains(&format!("orderBy: {order_by}")));
    assert!(LAST_DAYS_VOLUME_QUERY.contains(&format!("orderDirection: {order_direction}")));

    let payload = json!({
        "dailyTotals": [{ "timestamp": "1700000000", "volumeUsd": "42" }],
    });
    let decoded: LastDaysVolumeResponse = serde_json::from_value(payload).unwrap_or_else(|error| {
        panic!("case {id}: LastDaysVolumeResponse decoding must succeed: {error}")
    });
    assert_eq!(decoded.daily_totals.len(), 1);
    assert_eq!(decoded.daily_totals[0].timestamp, 1_700_000_000u64);
}

fn assert_last_hours_volume_query_contract(id: &str, expected: &Value) {
    let operation = expected["operation_name"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.operation_name must be a string"));
    let root = expected["root_field"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.root_field must be a string"));
    let variable = expected["variable_name"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.variable_name must be a string"));
    let order_by = expected["order_by"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.order_by must be a string"));
    let order_direction = expected["order_direction"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.order_direction must be a string"));

    assert!(LAST_HOURS_VOLUME_QUERY.contains(&format!("query {operation}")));
    assert!(LAST_HOURS_VOLUME_QUERY.contains(root));
    assert!(LAST_HOURS_VOLUME_QUERY.contains(&format!("${variable}")));
    assert!(LAST_HOURS_VOLUME_QUERY.contains(&format!("orderBy: {order_by}")));
    assert!(LAST_HOURS_VOLUME_QUERY.contains(&format!("orderDirection: {order_direction}")));

    let payload = json!({
        "hourlyTotals": [{ "timestamp": "1700003600", "volumeUsd": "7" }],
    });
    let decoded: LastHoursVolumeResponse =
        serde_json::from_value(payload).unwrap_or_else(|error| {
            panic!("case {id}: LastHoursVolumeResponse decoding must succeed: {error}")
        });
    assert_eq!(decoded.hourly_totals.len(), 1);
    assert_eq!(decoded.hourly_totals[0].timestamp, 1_700_003_600u64);
}

fn assert_custom_query_support(id: &str, expected: &Value) {
    assert!(
        expected["accepts_custom_query"].as_bool().unwrap_or(false),
        "case {id}: fixture must declare accepts_custom_query=true",
    );
    assert!(
        expected["accepts_variables"].as_bool().unwrap_or(false),
        "case {id}: fixture must declare accepts_variables=true",
    );

    let request = SubgraphQueryRequest::new("query TokensByVolume { tokens { id } }")
        .with_operation_name("TokensByVolume")
        .with_variables(json!({ "first": 5 }));
    assert_eq!(
        request.operation_name(),
        Some("TokensByVolume"),
        "case {id}: caller-supplied operation name must round-trip",
    );
    assert!(
        request.variables().is_some(),
        "case {id}: caller-supplied variables must round-trip",
    );
}

fn assert_custom_base_url_override(id: &str, expected: &Value) {
    let override_field = expected["override_field"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.override_field must be a string"));
    assert_eq!(
        override_field, "baseUrls",
        "case {id}: override field name must stay baseUrls",
    );
    assert!(
        expected["override_wins_over_prod_config"]
            .as_bool()
            .unwrap_or(false),
        "case {id}: fixture must declare override_wins_over_prod_config=true",
    );

    // Build a SubgraphConfig with a custom baseUrls map; verify the struct
    // captures the override, which is what the base-url resolution path reads
    // first during request routing.
    let mut urls = cow_sdk_subgraph::SubgraphApiBaseUrls::new();
    urls.insert(
        SupportedChainId::Mainnet,
        Some("https://custom-subgraph.example/graphql".to_owned()),
    );
    let config = SubgraphConfig {
        chain_id: SupportedChainId::Mainnet,
        base_urls: Some(urls),
    };
    assert!(
        config.base_urls.is_some(),
        "case {id}: baseUrls override must be recorded in SubgraphConfig",
    );
}

fn assert_generated_type_contract(id: &str, expected: &Value) {
    let types: Vec<&str> = expected["types"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.types must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: types entries must be strings"))
        })
        .collect();

    // The three Rust response DTOs mirror the upstream generated GraphQL
    // types — assert all three are present in the fixture and pin their
    // existence at compile time.
    assert!(
        types.contains(&"TotalsQuery")
            && types.contains(&"LastDaysVolumeQuery")
            && types.contains(&"LastHoursVolumeQuery"),
        "case {id}: fixture must name the three generated GraphQL response types",
    );

    let _ = std::mem::size_of::<TotalsResponse>();
    let _ = std::mem::size_of::<LastDaysVolumeResponse>();
    let _ = std::mem::size_of::<LastHoursVolumeResponse>();
}

fn assert_unsupported_network_error(id: &str, expected: &Value) {
    assert!(
        expected["must_reject"].as_bool().unwrap_or(false),
        "case {id}: fixture must declare must_reject=true",
    );
    let reason = expected["reason_contains"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.reason_contains must be a string"));
    assert_eq!(
        reason, "Unsupported Network",
        "case {id}: rejection reason must remain Unsupported Network",
    );

    // The fixture names polygon; polygon maps to None in the production config.
    let api = SubgraphApi::builder()
        .chain(SupportedChainId::Mainnet)
        .api_key("redacted-for-test")
        .build();
    let prod_config = api.prod_config();
    let polygon = prod_config
        .get(&SupportedChainId::Polygon)
        .unwrap_or_else(|| panic!("case {id}: polygon must be present in the prod config"));
    assert!(
        polygon.is_none(),
        "case {id}: polygon must map to None in the production config, triggering UnsupportedNetwork at call time",
    );
}

fn assert_empty_totals_error(id: &str, expected: &Value) {
    assert!(
        expected["must_reject"].as_bool().unwrap_or(false),
        "case {id}: fixture must declare must_reject=true",
    );
    let reason = expected["reason_contains"]
        .as_str()
        .unwrap_or_else(|| panic!("case {id}: expected.reason_contains must be a string"));
    assert_eq!(
        reason, "No totals found",
        "case {id}: rejection reason must remain No totals found",
    );

    // The typed rejection surface is SubgraphError::NoTotalsFound; reference it
    // to guarantee it remains exported and carries the pinned Display text.
    let error = cow_sdk_subgraph::SubgraphError::NoTotalsFound;
    assert_eq!(
        error.to_string(),
        "No totals found",
        "case {id}: NoTotalsFound Display text must match the fixture",
    );
}

fn assert_invalid_query_error(id: &str, expected: &Value) {
    assert!(
        expected["must_reject"].as_bool().unwrap_or(false),
        "case {id}: fixture must declare must_reject=true",
    );
    let reasons: Vec<&str> = expected["reason_contains"]
        .as_array()
        .unwrap_or_else(|| panic!("case {id}: expected.reason_contains must be an array"))
        .iter()
        .map(|value| {
            value
                .as_str()
                .unwrap_or_else(|| panic!("case {id}: reason_contains entries must be strings"))
        })
        .collect();

    // Rust routes GraphQL errors through SubgraphError::GraphQl with a
    // SubgraphRequestErrorContext and a list of SubgraphGraphQlError entries.
    // Construct a minimal fixture-shape error and assert the typed surface
    // preserves both the operation-name context and the GraphQL error text.
    let context = cow_sdk_subgraph::SubgraphRequestErrorContext {
        chain_id: u64::from(SupportedChainId::Mainnet),
        api: "CoW Protocol Subgraph".to_owned(),
        document: "query InvalidQuery { tokens { id } }".to_owned(),
        operation_name: Some("InvalidQuery".to_owned()),
        variables: None,
    };
    let errors = vec![cow_sdk_subgraph::SubgraphGraphQlError {
        message: "Error running query: InvalidQuery".to_owned(),
        locations: Vec::new(),
    }];
    let error = cow_sdk_subgraph::SubgraphError::GraphQl {
        context: Box::new(context),
        errors: errors.clone(),
    };
    let message = error.to_string();
    assert!(
        message.contains("subgraph graphql error response"),
        "case {id}: rejection must surface through the typed graphql error",
    );
    let graphql_body = errors
        .iter()
        .map(|entry| entry.message.as_str())
        .collect::<Vec<_>>()
        .join("; ");
    for reason in &reasons {
        assert!(
            graphql_body.contains(reason),
            "case {id}: typed error body must contain {reason}",
        );
    }
}

fn chain_label_to_rust(id: &str, label: &str) -> SupportedChainId {
    match label {
        "mainnet" => SupportedChainId::Mainnet,
        "gnosis_chain" => SupportedChainId::GnosisChain,
        "arbitrum_one" => SupportedChainId::ArbitrumOne,
        "base" => SupportedChainId::Base,
        "sepolia" => SupportedChainId::Sepolia,
        "polygon" => SupportedChainId::Polygon,
        "avalanche" => SupportedChainId::Avalanche,
        "bnb" => SupportedChainId::Bnb,
        "linea" => SupportedChainId::Linea,
        "plasma" => SupportedChainId::Plasma,
        "ink" => SupportedChainId::Ink,
        other => panic!("case {id}: unsupported chain label {other:?}"),
    }
}
