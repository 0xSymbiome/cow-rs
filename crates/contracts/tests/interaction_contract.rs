mod common;

use cow_sdk_contracts::{InteractionLike, normalize_interaction, normalize_interactions};
use cow_sdk_core::Address;

use common::fixture_case;

#[test]
fn interaction_normalization_applies_zero_value_call_defaults() {
    let fixture = fixture_case("contracts-interaction-defaults");
    let target = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();

    let normalized = normalize_interaction(&InteractionLike {
        target: target.clone(),
        value: None,
        call_data: None,
    });
    assert_eq!(normalized.target, target);
    assert_eq!(
        normalized.value,
        fixture["expected"]["value"].as_str().unwrap()
    );
    assert_eq!(
        normalized.call_data,
        fixture["expected"]["call_data"].as_str().unwrap()
    );

    let explicit = normalize_interaction(&InteractionLike {
        target: normalized.target.clone(),
        value: Some("42".to_owned()),
        call_data: Some("0x12345678".to_owned()),
    });
    assert_eq!(explicit.value, "42");
    assert_eq!(explicit.call_data, "0x12345678");
}

#[test]
fn batch_interaction_normalization_preserves_order() {
    let interactions = vec![
        InteractionLike {
            target: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            value: None,
            call_data: None,
        },
        InteractionLike {
            target: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            value: Some("7".to_owned()),
            call_data: Some("0x01020304".to_owned()),
        },
    ];

    let normalized = normalize_interactions(&interactions);
    assert_eq!(normalized.len(), 2);
    assert_eq!(normalized[0].value, "0");
    assert_eq!(normalized[0].call_data, "0x");
    assert_eq!(normalized[1].value, "7");
    assert_eq!(normalized[1].call_data, "0x01020304");
    assert_eq!(normalized[1].target, interactions[1].target);
}
