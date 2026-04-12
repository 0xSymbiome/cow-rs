use cow_sdk_subgraph::{
    LastDaysVolumeResponse, LastHoursVolumeResponse, SubgraphQueryRequest, TotalsResponse,
};
use serde_json::{Map, Value, json};

const CASE_COUNT: u64 = 128;

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
        let operation_name = rng
            .next_bool()
            .then(|| format!("GeneratedOp{}", seed));
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
