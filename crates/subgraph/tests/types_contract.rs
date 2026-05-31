use cow_sdk_subgraph::{
    LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphGraphQlError,
    SubgraphGraphQlErrorLocation, TotalsResponse,
};
use serde_json::json;

#[test]
fn totals_response_deserializes_authoritative_fields() {
    let response: TotalsResponse = serde_json::from_value(json!({
        "totals": [
            {
                "tokens": "192",
                "orders": "365210",
                "traders": "50731",
                "settlements": "160092",
                "volumeUsd": "49548634.23978489392550883815112596",
                "volumeEth": "20349080.82753326160179174564685693",
                "feesUsd": "1495.18088540037791409373835505834",
                "feesEth": "632.7328748466552906975758491191759"
            }
        ]
    }))
    .unwrap();

    assert_eq!(response.totals.len(), 1);
    assert_eq!(response.totals[0].tokens, "192");
    assert_eq!(
        response.totals[0].volume_usd.as_deref(),
        Some("49548634.23978489392550883815112596")
    );
}

#[test]
fn last_days_volume_response_accepts_string_backed_scalars() {
    let response: LastDaysVolumeResponse = serde_json::from_value(json!({
        "dailyTotals": [
            {
                "timestamp": "1651104000",
                "volumeUsd": "32085.1639220805155999650325844739"
            },
            {
                "timestamp": 1_651_017_600,
                "volumeUsd": 34_693.620_077_172_98
            }
        ]
    }))
    .unwrap();

    assert_eq!(response.daily_totals.len(), 2);
    assert_eq!(response.daily_totals[0].timestamp, 1_651_104_000);
    assert_eq!(
        response.daily_totals[0].volume_usd.as_deref(),
        Some("32085.1639220805155999650325844739")
    );
    assert_eq!(response.daily_totals[1].timestamp, 1_651_017_600);
}

#[test]
fn last_hours_volume_response_accepts_string_backed_scalars() {
    let response: LastHoursVolumeResponse = serde_json::from_value(json!({
        "hourlyTotals": [
            {
                "timestamp": "1651186800",
                "volumeUsd": "190.9404913756501392195019404899438"
            },
            {
                "timestamp": 1_651_183_200,
                "volumeUsd": 529.994_623_800_056_2
            }
        ]
    }))
    .unwrap();

    assert_eq!(response.hourly_totals.len(), 2);
    assert_eq!(response.hourly_totals[0].timestamp, 1_651_186_800);
    assert_eq!(
        response.hourly_totals[0].volume_usd.as_deref(),
        Some("190.9404913756501392195019404899438")
    );
    assert_eq!(response.hourly_totals[1].timestamp, 1_651_183_200);
}

#[test]
fn graphql_error_payload_preserves_message_and_locations() {
    let error: SubgraphGraphQlError = serde_json::from_value(json!({
        "message": "Type `Query` has no field `invalidQuery`",
        "locations": [
            {
                "line": 2,
                "column": 9
            }
        ]
    }))
    .unwrap();

    assert_eq!(
        error,
        SubgraphGraphQlError::new(
            "Type `Query` has no field `invalidQuery`",
            vec![SubgraphGraphQlErrorLocation::new(2, 9)],
        )
    );
}

#[test]
fn graphql_error_payload_allows_missing_locations() {
    let error: SubgraphGraphQlError = serde_json::from_value(json!({
        "message": "Something went wrong"
    }))
    .unwrap();

    assert_eq!(error.message.as_inner(), "Something went wrong");
    assert!(error.locations.is_empty());
}

#[test]
fn subgraph_graphql_error_decodes_the_graph_wire_shape() {
    // The Graph returns GraphQL errors as `message` plus optional `locations`,
    // with no `extensions` and no machine-readable error code. A
    // gateway-authored failure (here, no available indexers) decodes with
    // empty locations and absent extensions.
    let gateway_error: SubgraphGraphQlError =
        serde_json::from_value(json!({ "message": "no indexers found" })).unwrap();
    assert!(gateway_error.locations.is_empty());
    assert!(gateway_error.extensions.is_none());

    // An indexer query error carries source locations but still exposes no
    // extensions or code.
    let indexer_error: SubgraphGraphQlError = serde_json::from_value(json!({
        "message": "Failed to decode value for field `number`",
        "locations": [{ "line": 3, "column": 5 }]
    }))
    .unwrap();
    assert_eq!(
        indexer_error.locations,
        vec![SubgraphGraphQlErrorLocation::new(3, 5)]
    );
    assert!(indexer_error.extensions.is_none());

    // `extensions` stays an opaque pass-through for any GraphQL endpoint that
    // does populate it: the SDK preserves the field without ascribing
    // coded-reason semantics to it.
    let with_extensions: SubgraphGraphQlError = serde_json::from_value(json!({
        "message": "GraphQL request failed",
        "extensions": { "code": "GRAPHQL_VALIDATION_FAILED" }
    }))
    .unwrap();
    assert!(with_extensions.extensions.is_some());
}

#[test]
fn scalar_decode_rejects_non_finite_floats_and_overflows() {
    for invalid_volume in ["NaN", "Infinity", "-Infinity"] {
        let error = serde_json::from_value::<LastDaysVolumeResponse>(json!({
            "dailyTotals": [
                {
                    "timestamp": "1651104000",
                    "volumeUsd": invalid_volume
                }
            ]
        }))
        .expect_err("non-finite scalar strings must be rejected");
        assert!(
            error.to_string().contains("finite"),
            "error should identify finite scalar requirement: {error}",
        );
    }

    let overflow = serde_json::from_value::<LastDaysVolumeResponse>(json!({
        "dailyTotals": [
            {
                "timestamp": "18446744073709551616",
                "volumeUsd": "1"
            }
        ]
    }))
    .expect_err("timestamp values above u64::MAX must be rejected");
    assert!(
        overflow.to_string().contains("number too large")
            || overflow.to_string().contains("out of range")
            || overflow.to_string().contains("invalid digit"),
        "overflow error should preserve numeric parse context: {overflow}",
    );
}
