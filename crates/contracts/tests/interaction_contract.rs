#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, and perf lints acceptable in test helper code"
)]

mod common;

use bytes::Bytes;
use cow_sdk_contracts::{InteractionLike, normalize_interaction, normalize_interactions};
use cow_sdk_core::{Address, Amount};

use common::fixture_case;

fn bytes_from_hex_literal(literal: &str) -> Bytes {
    let stripped = literal
        .strip_prefix("0x")
        .expect("hex literal must start with 0x");
    Bytes::from(hex::decode(stripped).expect("hex literal must decode"))
}

fn hex_prefixed(bytes: &Bytes) -> String {
    format!("0x{}", hex::encode(bytes))
}

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
        normalized.value.to_string(),
        fixture["expected"]["value"].as_str().unwrap()
    );
    assert_eq!(
        hex_prefixed(&normalized.call_data),
        fixture["expected"]["call_data"].as_str().unwrap()
    );
    assert!(
        normalized.call_data.is_empty(),
        "default calldata must be an empty byte buffer"
    );

    let explicit = normalize_interaction(&InteractionLike {
        target: normalized.target.clone(),
        value: Some(Amount::new("42").unwrap()),
        call_data: Some(bytes_from_hex_literal("0x12345678")),
    });
    assert_eq!(explicit.value.to_string(), "42");
    assert_eq!(
        explicit.call_data.as_ref(),
        &[0x12, 0x34, 0x56, 0x78][..],
        "explicit calldata must round-trip byte-equal through the encoder"
    );
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
            value: Some(Amount::new("7").unwrap()),
            call_data: Some(bytes_from_hex_literal("0x01020304")),
        },
    ];

    let normalized = normalize_interactions(&interactions);
    assert_eq!(normalized.len(), 2);
    assert_eq!(normalized[0].value.to_string(), "0");
    assert!(
        normalized[0].call_data.is_empty(),
        "missing calldata must normalize to an empty byte buffer"
    );
    assert_eq!(normalized[1].value.to_string(), "7");
    assert_eq!(
        normalized[1].call_data.as_ref(),
        &[0x01, 0x02, 0x03, 0x04][..],
        "explicit calldata must preserve the input bytes through normalization"
    );
    assert_eq!(normalized[1].target, interactions[1].target);
}

#[test]
fn interaction_calldata_clone_shares_backing_allocation() {
    let interaction = normalize_interaction(&InteractionLike {
        target: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        value: None,
        call_data: Some(bytes_from_hex_literal("0xdeadbeefcafef00d")),
    });

    let cloned = interaction.call_data.clone();
    assert_eq!(
        cloned, interaction.call_data,
        "bytes::Bytes clone must preserve the original byte sequence"
    );
    assert_eq!(
        cloned.as_ptr(),
        interaction.call_data.as_ptr(),
        "bytes::Bytes clone must reference the same backing allocation"
    );
}
