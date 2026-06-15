//! Fixture-driven wire parity for the split `SellTokenSource` and
//! `BuyTokenDestination` contract types.
//!
//! Services models the sell-side allowance path and the buy-side payout
//! path as two distinct enums:
//!
//! - `SellTokenSource { Erc20, External, Internal }` is published by the
//!   upstream services model crate.
//! - `BuyTokenDestination { Erc20, Internal }` is published alongside it
//!   in the same services model crate.
//!
//! Both emit their variant names on the wire as lowercase strings
//! (`"erc20"`, `"external"`, `"internal"`), and both reject any other
//! spelling on deserialization with `deny_unknown_fields`-style strict
//! matching. The test below pins both properties on the cow-rs split:
//!
//! - Every variant round-trips byte-identically through
//!   `serde_json::to_string` / `serde_json::from_str`.
//! - The closed-variant contract on `BuyTokenDestination` rejects the
//!   sell-only `"external"` spelling at the deserialization boundary —
//!   this is the parity counterpart to the upstream rule that
//!   `BuyTokenDestination` does not admit an external-vault payout
//!   path.

use cow_sdk_core::{BuyTokenDestination, SellTokenSource};

const SELL_TOKEN_SOURCE_WIRE: &[(SellTokenSource, &str)] = &[
    (SellTokenSource::Erc20, "\"erc20\""),
    (SellTokenSource::External, "\"external\""),
    (SellTokenSource::Internal, "\"internal\""),
];

const BUY_TOKEN_DESTINATION_WIRE: &[(BuyTokenDestination, &str)] = &[
    (BuyTokenDestination::Erc20, "\"erc20\""),
    (BuyTokenDestination::Internal, "\"internal\""),
];

#[test]
fn sell_token_source_wire_round_trip_matches_services_byte_identically() {
    for (variant, expected) in SELL_TOKEN_SOURCE_WIRE {
        let encoded =
            serde_json::to_string(variant).expect("SellTokenSource must serialize to JSON");
        assert_eq!(
            &encoded, expected,
            "SellTokenSource::{variant:?} must serialize to {expected}",
        );

        let decoded: SellTokenSource = serde_json::from_str(expected)
            .expect("services wire string must deserialize back into SellTokenSource");
        assert_eq!(
            decoded, *variant,
            "SellTokenSource round-trip must preserve the original variant",
        );
    }
}

#[test]
fn buy_token_destination_wire_round_trip_matches_services_byte_identically() {
    for (variant, expected) in BUY_TOKEN_DESTINATION_WIRE {
        let encoded =
            serde_json::to_string(variant).expect("BuyTokenDestination must serialize to JSON");
        assert_eq!(
            &encoded, expected,
            "BuyTokenDestination::{variant:?} must serialize to {expected}",
        );

        let decoded: BuyTokenDestination = serde_json::from_str(expected)
            .expect("services wire string must deserialize back into BuyTokenDestination");
        assert_eq!(
            decoded, *variant,
            "BuyTokenDestination round-trip must preserve the original variant",
        );
    }
}

#[test]
fn buy_token_destination_rejects_sell_only_external_variant_on_the_wire() {
    let rejection = serde_json::from_str::<BuyTokenDestination>("\"external\"");
    assert!(
        rejection.is_err(),
        "BuyTokenDestination must reject the sell-only `external` spelling so the buy-side payout path stays closed to {{erc20, internal}}",
    );
}

#[test]
fn default_variants_match_services_reference() {
    assert_eq!(SellTokenSource::default(), SellTokenSource::Erc20);
    assert_eq!(BuyTokenDestination::default(), BuyTokenDestination::Erc20);
}
