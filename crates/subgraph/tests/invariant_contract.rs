#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::derive_partial_eq_without_eq,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, style, and perf lints acceptable in test helper code"
)]

use cow_sdk_subgraph::{
    LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphQueryRequest, TotalsResponse,
};
use serde_json::{Map, Value, json};

const CASE_COUNT: u64 = 128;
const SEARCH_CASE_COUNT: u64 = 512;

#[derive(Clone)]
struct CaseRng {
    state: u64,
}

impl CaseRng {
    fn new(seed: u64) -> Self {
        Self {
            state: seed ^ 0xE703_7ED1_A0B4_28DB,
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut value = self.state;
        value ^= value << 7;
        value ^= value >> 9;
        value ^= value << 8;
        self.state = value;
        value
    }

    fn next_u32(&mut self) -> u32 {
        (self.next_u64() >> 16) as u32
    }

    fn next_bool(&mut self) -> bool {
        (self.next_u64() & 1) == 1
    }
}

fn generated_variables(rng: &mut CaseRng) -> Value {
    let mut object = Map::new();
    object.insert("limit".to_owned(), json!(1 + (rng.next_u32() % 50)));
    object.insert(
        "label".to_owned(),
        json!(format!("token-{}", rng.next_u32() % 10_000)),
    );
    object.insert("enabled".to_owned(), json!(rng.next_bool()));
    Value::Object(object)
}

fn generated_nested_variables(rng: &mut CaseRng) -> Value {
    let mut filters = Map::new();
    filters.insert(
        "owners".to_owned(),
        json!([
            format!("0x{:040x}", rng.next_u64()),
            format!("0x{:040x}", rng.next_u64()),
        ]),
    );
    filters.insert(
        "minVolume".to_owned(),
        json!(1 + (rng.next_u64() % 1_000_000)),
    );
    filters.insert("includeInactive".to_owned(), json!(rng.next_bool()));

    let mut variables = Map::new();
    variables.insert("limit".to_owned(), json!(1 + (rng.next_u32() % 50)));
    variables.insert("offset".to_owned(), json!(rng.next_u32() % 500));
    variables.insert("filters".to_owned(), Value::Object(filters));
    variables.insert(
        "windows".to_owned(),
        json!([
            { "kind": "daily", "size": 1 + (rng.next_u32() % 30) },
            { "kind": "hourly", "size": 1 + (rng.next_u32() % 48) }
        ]),
    );
    Value::Object(variables)
}

fn generated_numeric_string(rng: &mut CaseRng) -> String {
    (1 + (rng.next_u64() % 1_000_000_000)).to_string()
}

fn generated_optional_numeric_string(rng: &mut CaseRng) -> Option<String> {
    rng.next_bool().then(|| generated_numeric_string(rng))
}

fn generated_search_profile_variables(rng: &mut CaseRng, depth: usize) -> Value {
    if depth == 0 {
        return match rng.next_u64() % 5 {
            0 => json!(1 + (rng.next_u32() % 10_000)),
            1 => json!(rng.next_bool()),
            2 => json!(format!("value-{}", rng.next_u32())),
            3 => json!(format!("0x{:040x}", rng.next_u64())),
            _ => Value::Null,
        };
    }

    match rng.next_u64() % 4 {
        0 => generated_search_profile_variables(rng, 0),
        1 => {
            let len = 1 + (rng.next_u64() % 4) as usize;
            Value::Array(
                (0..len)
                    .map(|_| generated_search_profile_variables(rng, depth.saturating_sub(1)))
                    .collect(),
            )
        }
        _ => {
            let len = 1 + (rng.next_u64() % 4) as usize;
            let mut object = Map::new();
            for index in 0..len {
                object.insert(
                    format!("node-{}-{}", index, rng.next_u32()),
                    generated_search_profile_variables(rng, depth.saturating_sub(1)),
                );
            }
            Value::Object(object)
        }
    }
}

fn generated_search_profile_document(case: u64) -> (String, Option<String>) {
    if case.is_multiple_of(2) {
        (
            format!(
                "query TotalsPrimary{case}($input: TotalsInput!, $cursor: CursorInput) {{ totals(input: $input, cursor: $cursor) {{ orders }} }} query TotalsSecondary{case}($owner: ID!) {{ orders(where: {{ owner: $owner }}) {{ uid }} }}"
            ),
            Some(format!("TotalsSecondary{case}")),
        )
    } else {
        (
            "query($input: TotalsInput!, $cursor: CursorInput) { totals(input: $input, cursor: $cursor) { orders } }".to_owned(),
            None,
        )
    }
}

fn numeric_or_string_value(case: u64, value: &str) -> Value {
    if case.is_multiple_of(2) {
        json!(value)
    } else {
        json!(value.parse::<u64>().unwrap())
    }
}

#[test]
fn subgraph_request_shape_roundtrips_without_losing_explicit_fields() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 1);
        let document = if rng.next_bool() {
            format!(
                "query GeneratedOp{}($limit: Int!) {{ tokens(first: $limit) {{ symbol }} }}",
                seed
            )
        } else {
            "{ totals { orders } }".to_owned()
        };
        let variables = rng.next_bool().then(|| generated_variables(&mut rng));
        let operation_name = rng.next_bool().then(|| format!("GeneratedOp{}", seed));
        let mut request = SubgraphQueryRequest::new(document.clone());

        if let Some(variables) = variables.clone() {
            request = request.with_variables(variables);
        } else if rng.next_bool() {
            request = request.with_optional_variables(None);
        }

        if let Some(operation_name) = operation_name.clone() {
            request = request.with_operation_name(operation_name);
        }

        let value = serde_json::to_value(&request).expect("request serialization must succeed");

        assert_eq!(value["document"], json!(document));
        if let Some(variables) = variables {
            assert_eq!(value["variables"], variables);
        } else {
            assert!(value.get("variables").is_none());
        }
        if let Some(operation_name) = operation_name {
            assert_eq!(value["operation_name"], json!(operation_name));
        } else {
            assert!(value.get("operation_name").is_none());
        }

        let roundtrip: SubgraphQueryRequest =
            serde_json::from_value(value).expect("request roundtrip must remain stable");
        assert_eq!(roundtrip, request);
    }
}

#[test]
fn nested_subgraph_variables_roundtrip_without_normalizing_objects_or_arrays() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 3_001);
        let document = format!(
            "query WindowedTotals{}($limit: Int!, $offset: Int!, $filters: TotalsFilter!, $windows: [WindowInput!]!) {{ totals {{ orders }} }}",
            seed
        );
        let variables = generated_nested_variables(&mut rng);
        let request = SubgraphQueryRequest::new(document.clone())
            .with_variables(variables.clone())
            .with_operation_name(format!("WindowedTotals{}", seed));

        let value = serde_json::to_value(&request).expect("request serialization must succeed");
        let roundtrip: SubgraphQueryRequest =
            serde_json::from_value(value.clone()).expect("request roundtrip must remain stable");

        assert_eq!(value["document"], json!(document));
        assert_eq!(value["variables"], variables);
        assert_eq!(
            value["operation_name"],
            json!(format!("WindowedTotals{}", seed))
        );
        assert_eq!(roundtrip, request);
    }
}

#[test]
fn subgraph_scalar_responses_accept_equivalent_string_and_number_forms() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 9_001);
        let tokens = generated_numeric_string(&mut rng);
        let orders = generated_numeric_string(&mut rng);
        let traders = generated_numeric_string(&mut rng);
        let settlements = generated_numeric_string(&mut rng);
        let volume_usd = generated_optional_numeric_string(&mut rng);
        let volume_eth = generated_optional_numeric_string(&mut rng);
        let fees_usd = generated_optional_numeric_string(&mut rng);
        let fees_eth = generated_optional_numeric_string(&mut rng);
        let day_timestamp = 1_651_000_000u64 + (rng.next_u64() % 50_000);
        let day_volume = generated_optional_numeric_string(&mut rng);
        let hour_timestamp = 1_651_100_000u64 + (rng.next_u64() % 50_000);
        let hour_volume = generated_optional_numeric_string(&mut rng);

        let totals_from_strings: TotalsResponse = serde_json::from_value(json!({
            "totals": [{
                "tokens": tokens,
                "orders": orders,
                "traders": traders,
                "settlements": settlements,
                "volumeUsd": volume_usd,
                "volumeEth": volume_eth,
                "feesUsd": fees_usd,
                "feesEth": fees_eth,
            }]
        }))
        .expect("string-backed totals must deserialize");
        let totals_from_numbers: TotalsResponse = serde_json::from_value(json!({
            "totals": [{
                "tokens": totals_from_strings.totals[0].tokens.parse::<u64>().unwrap(),
                "orders": totals_from_strings.totals[0].orders.parse::<u64>().unwrap(),
                "traders": totals_from_strings.totals[0].traders.parse::<u64>().unwrap(),
                "settlements": totals_from_strings.totals[0].settlements.parse::<u64>().unwrap(),
                "volumeUsd": totals_from_strings.totals[0].volume_usd.as_ref().map(|value| value.parse::<u64>().unwrap()),
                "volumeEth": totals_from_strings.totals[0].volume_eth.as_ref().map(|value| value.parse::<u64>().unwrap()),
                "feesUsd": totals_from_strings.totals[0].fees_usd.as_ref().map(|value| value.parse::<u64>().unwrap()),
                "feesEth": totals_from_strings.totals[0].fees_eth.as_ref().map(|value| value.parse::<u64>().unwrap()),
            }]
        }))
        .expect("number-backed totals must deserialize");

        let days_from_strings: LastDaysVolumeResponse = serde_json::from_value(json!({
            "dailyTotals": [{
                "timestamp": day_timestamp.to_string(),
                "volumeUsd": day_volume,
            }]
        }))
        .expect("string-backed daily totals must deserialize");
        let days_from_numbers: LastDaysVolumeResponse = serde_json::from_value(json!({
            "dailyTotals": [{
                "timestamp": day_timestamp,
                "volumeUsd": days_from_strings.daily_totals[0].volume_usd.as_ref().map(|value| value.parse::<u64>().unwrap()),
            }]
        }))
        .expect("number-backed daily totals must deserialize");

        let hours_from_strings: LastHoursVolumeResponse = serde_json::from_value(json!({
            "hourlyTotals": [{
                "timestamp": hour_timestamp.to_string(),
                "volumeUsd": hour_volume,
            }]
        }))
        .expect("string-backed hourly totals must deserialize");
        let hours_from_numbers: LastHoursVolumeResponse = serde_json::from_value(json!({
            "hourlyTotals": [{
                "timestamp": hour_timestamp,
                "volumeUsd": hours_from_strings.hourly_totals[0].volume_usd.as_ref().map(|value| value.parse::<u64>().unwrap()),
            }]
        }))
        .expect("number-backed hourly totals must deserialize");

        assert_eq!(totals_from_numbers, totals_from_strings);
        assert_eq!(days_from_numbers, days_from_strings);
        assert_eq!(hours_from_numbers, hours_from_strings);
    }
}

#[test]
fn malformed_subgraph_scalars_fail_closed_during_response_decoding() {
    for seed in 0..CASE_COUNT {
        let mut rng = CaseRng::new(seed + 5_001);
        match rng.next_u32() % 4 {
            0 => {
                let error = serde_json::from_value::<TotalsResponse>(json!({
                    "totals": [
                        {
                            "tokens": true,
                            "orders": "365210",
                            "traders": "50731",
                            "settlements": "160092"
                        }
                    ]
                }))
                .expect_err("boolean totals scalars must fail closed");
                assert!(!error.to_string().is_empty());
            }
            1 => {
                let error = serde_json::from_value::<TotalsResponse>(json!({
                    "totals": [
                        {
                            "tokens": "192",
                            "orders": ["bad"],
                            "traders": "50731",
                            "settlements": "160092"
                        }
                    ]
                }))
                .expect_err("array-backed totals scalars must fail closed");
                assert!(!error.to_string().is_empty());
            }
            2 => {
                let error = serde_json::from_value::<LastDaysVolumeResponse>(json!({
                    "dailyTotals": [
                        {
                            "timestamp": "not-a-timestamp",
                            "volumeUsd": "32085.16"
                        }
                    ]
                }))
                .expect_err("invalid timestamp strings must fail closed");
                assert!(!error.to_string().is_empty());
            }
            _ => {
                let error = serde_json::from_value::<LastHoursVolumeResponse>(json!({
                    "hourlyTotals": [
                        {
                            "timestamp": "1651186800",
                            "volumeUsd": { "unexpected": true }
                        }
                    ]
                }))
                .expect_err("object-backed volume scalars must fail closed");
                assert!(!error.to_string().is_empty());
            }
        }
    }
}

#[test]
fn raw_request_narrow_search_profile_preserves_explicit_documents_and_nested_variables() {
    for case in 0..SEARCH_CASE_COUNT {
        let mut rng = CaseRng::new(case + 15_001);
        let (document, operation_name) = generated_search_profile_document(case);
        let variables = Value::Object(Map::from_iter([
            (
                "input".to_owned(),
                generated_search_profile_variables(&mut rng, 3),
            ),
            (
                "cursor".to_owned(),
                generated_search_profile_variables(&mut rng, 2),
            ),
        ]));
        let mut request =
            SubgraphQueryRequest::new(document.clone()).with_variables(variables.clone());
        if let Some(operation_name) = operation_name.clone() {
            request = request.with_operation_name(operation_name);
        }

        let value = serde_json::to_value(&request).expect("request serialization must succeed");
        let roundtrip: SubgraphQueryRequest =
            serde_json::from_value(value.clone()).expect("request roundtrip must remain stable");

        assert_eq!(value["document"], json!(document), "case {case}");
        assert_eq!(value["variables"], variables, "case {case}");
        match operation_name {
            Some(name) => assert_eq!(value["operation_name"], json!(name), "case {case}"),
            None => assert!(value.get("operation_name").is_none(), "case {case}"),
        }
        assert_eq!(roundtrip, request, "case {case}");
    }
}

#[test]
fn scalar_decode_narrow_search_profile_covers_boundary_numeric_forms() {
    for case in 0..SEARCH_CASE_COUNT {
        let mut rng = CaseRng::new(case + 27_001);
        let tokens = generated_numeric_string(&mut rng);
        let orders = generated_numeric_string(&mut rng);
        let traders = generated_numeric_string(&mut rng);
        let settlements = generated_numeric_string(&mut rng);
        let day_timestamp = if case.is_multiple_of(2) {
            u64::MAX
        } else {
            1_651_000_000u64 + (rng.next_u64() % 50_000)
        };
        let hour_timestamp = if case.is_multiple_of(3) {
            0
        } else {
            1_651_100_000u64 + (rng.next_u64() % 50_000)
        };

        let totals: TotalsResponse = serde_json::from_value(json!({
            "totals": [{
                "tokens": numeric_or_string_value(case, &tokens),
                "orders": numeric_or_string_value(case + 1, &orders),
                "traders": numeric_or_string_value(case + 2, &traders),
                "settlements": numeric_or_string_value(case + 3, &settlements),
                "volumeUsd": numeric_or_string_value(case + 4, &generated_numeric_string(&mut rng)),
                "volumeEth": numeric_or_string_value(case + 5, &generated_numeric_string(&mut rng)),
                "feesUsd": numeric_or_string_value(case + 6, &generated_numeric_string(&mut rng)),
                "feesEth": numeric_or_string_value(case + 7, &generated_numeric_string(&mut rng)),
            }]
        }))
        .expect("mixed numeric forms must deserialize");
        let days: LastDaysVolumeResponse = serde_json::from_value(json!({
            "dailyTotals": [{
                "timestamp": numeric_or_string_value(case + 8, &day_timestamp.to_string()),
                "volumeUsd": numeric_or_string_value(case + 9, &generated_numeric_string(&mut rng)),
            }]
        }))
        .expect("daily volume boundary forms must deserialize");
        let hours: LastHoursVolumeResponse = serde_json::from_value(json!({
            "hourlyTotals": [{
                "timestamp": numeric_or_string_value(case + 10, &hour_timestamp.to_string()),
                "volumeUsd": numeric_or_string_value(case + 11, &generated_numeric_string(&mut rng)),
            }]
        }))
        .expect("hourly volume boundary forms must deserialize");

        assert_eq!(totals.totals[0].tokens, tokens, "case {case}");
        assert_eq!(totals.totals[0].orders, orders, "case {case}");
        assert_eq!(totals.totals[0].traders, traders, "case {case}");
        assert_eq!(totals.totals[0].settlements, settlements, "case {case}");
        assert_eq!(days.daily_totals[0].timestamp, day_timestamp, "case {case}");
        assert_eq!(
            hours.hourly_totals[0].timestamp, hour_timestamp,
            "case {case}"
        );
    }
}
