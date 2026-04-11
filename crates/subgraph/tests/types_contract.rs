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
                "timestamp": 1651017600,
                "volumeUsd": 34693.62007717298
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
                "timestamp": 1651183200,
                "volumeUsd": 529.9946238000562
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
        SubgraphGraphQlError {
            message: "Type `Query` has no field `invalidQuery`".to_owned(),
            locations: vec![SubgraphGraphQlErrorLocation { line: 2, column: 9 }],
        }
    );
}

#[test]
fn graphql_error_payload_allows_missing_locations() {
    let error: SubgraphGraphQlError = serde_json::from_value(json!({
        "message": "Something went wrong"
    }))
    .unwrap();

    assert_eq!(error.message, "Something went wrong");
    assert!(error.locations.is_empty());
}
