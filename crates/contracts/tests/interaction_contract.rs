#![allow(
    clippy::missing_const_for_fn,
    clippy::redundant_clone,
    reason = "nursery and perf lints acceptable in test helper code"
)]

use cow_sdk_contracts::{InteractionLike, normalize_interaction, normalize_interactions};
use cow_sdk_core::{Address, Amount};

mod common;
use common::bytes_from_hex_literal;

#[test]
fn interaction_normalization_preserves_explicit_value_and_calldata() {
    let target = Address::new("0x9008D19f58AAbD9eD0D60971565AA8510560ab41").unwrap();

    let explicit = normalize_interaction(&InteractionLike::new(
        target,
        Some(Amount::new("42").unwrap()),
        Some(bytes_from_hex_literal("0x12345678")),
    ));
    assert_eq!(explicit.value.to_string(), "42");
    assert_eq!(
        explicit.call_data.as_ref(),
        &[0x12, 0x34, 0x56, 0x78][..],
        "explicit calldata must round-trip byte-equal through normalization"
    );
}

#[test]
fn batch_interaction_normalization_preserves_order() {
    let interactions = vec![
        InteractionLike::new(
            Address::new("0x1111111111111111111111111111111111111111").unwrap(),
            None,
            None,
        ),
        InteractionLike::new(
            Address::new("0x2222222222222222222222222222222222222222").unwrap(),
            Some(Amount::new("7").unwrap()),
            Some(bytes_from_hex_literal("0x01020304")),
        ),
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
    let interaction = normalize_interaction(&InteractionLike::new(
        Address::new("0x3333333333333333333333333333333333333333").unwrap(),
        None,
        Some(bytes_from_hex_literal("0xdeadbeefcafef00d")),
    ));

    let cloned = interaction.call_data.clone();
    assert_eq!(
        cloned, interaction.call_data,
        "alloy_primitives::Bytes clone must preserve the original byte sequence"
    );
    assert_eq!(
        cloned.as_ptr(),
        interaction.call_data.as_ptr(),
        "alloy_primitives::Bytes clone must reference the same backing allocation"
    );
}
