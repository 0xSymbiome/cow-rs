//! Contract suite pinning the public partner-fee surface against the
//! reviewed wire parity, the published basis-point bounds, the typed
//! construction error surface, the legacy `{ bps, recipient }` shape,
//! and the compile-fail witness that proves the narrowed `u16` field
//! rejects wider integer literals at the compiler.
//!
//! The fixture JSON inputs in this module mirror the reviewed services
//! test matrix for `PartnerFees` so drift in either the wire shape or
//! the parse-side discrimination is caught before it reaches release.

#![allow(
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic lint group acceptable inside integration test code"
)]

use cow_sdk_app_data::{AppDataError, PartnerFee, PartnerFeePolicy};
use cow_sdk_core::{Address, ValidationReason};
use serde_json::{Value, json};

const RECIPIENT_A: &str = "0x0101010101010101010101010101010101010101";
const RECIPIENT_B: &str = "0x0202020202020202020202020202020202020202";
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

fn address(hex: &str) -> Address {
    Address::new(hex).expect("fixture address must be valid")
}

#[test]
fn volume_single_shape_roundtrips_against_the_reviewed_wire_fixture() {
    let wire = json!({
        "volumeBps": 1000,
        "recipient": RECIPIENT_A,
    });
    let fee = PartnerFee::from_value(wire.clone()).expect("volume single must parse");
    assert_eq!(fee.to_value(), wire, "volume single must roundtrip");
    assert_eq!(fee.volume_bps(), Some(1000));
}

#[test]
fn surplus_single_shape_roundtrips_against_the_reviewed_wire_fixture() {
    let wire = json!({
        "surplusBps": 100,
        "maxVolumeBps": 100,
        "recipient": RECIPIENT_A,
    });
    let fee = PartnerFee::from_value(wire.clone()).expect("surplus single must parse");
    assert_eq!(fee.to_value(), wire, "surplus single must roundtrip");
    assert_eq!(fee.volume_bps(), None);
    fee.validate().expect("surplus single must validate");
}

#[test]
fn price_improvement_single_shape_roundtrips_against_the_reviewed_wire_fixture() {
    let wire = json!({
        "priceImprovementBps": 100,
        "maxVolumeBps": 100,
        "recipient": RECIPIENT_A,
    });
    let fee = PartnerFee::from_value(wire.clone()).expect("price-improvement single must parse");
    assert_eq!(
        fee.to_value(),
        wire,
        "price-improvement single must roundtrip"
    );
    assert_eq!(fee.volume_bps(), None);
    fee.validate()
        .expect("price-improvement single must validate");
}

#[test]
fn array_mixed_shape_roundtrips_against_the_reviewed_wire_fixture() {
    let wire = json!([
        {
            "bps": 100,
            "recipient": RECIPIENT_B,
        },
        {
            "volumeBps": 1000,
            "recipient": RECIPIENT_A,
        },
        {
            "surplusBps": 100,
            "maxVolumeBps": 100,
            "recipient": RECIPIENT_A,
        },
        {
            "priceImprovementBps": 100,
            "maxVolumeBps": 100,
            "recipient": RECIPIENT_A,
        },
    ]);
    let fee = PartnerFee::from_value(wire).expect("array shape must parse");
    let normalized = json!([
        {
            "volumeBps": 100,
            "recipient": RECIPIENT_B,
        },
        {
            "volumeBps": 1000,
            "recipient": RECIPIENT_A,
        },
        {
            "surplusBps": 100,
            "maxVolumeBps": 100,
            "recipient": RECIPIENT_A,
        },
        {
            "priceImprovementBps": 100,
            "maxVolumeBps": 100,
            "recipient": RECIPIENT_A,
        },
    ]);
    assert_eq!(
        fee.to_value(),
        normalized,
        "array must roundtrip with the legacy `bps` key promoted to `volumeBps`",
    );
    assert_eq!(
        fee.volume_bps(),
        Some(100),
        "array volume_bps must return the first volume-shaped fee",
    );
}

#[test]
fn legacy_bps_object_parses_as_volume_for_services_parity() {
    let wire = json!({
        "bps": 100,
        "recipient": RECIPIENT_B,
    });
    let fee = PartnerFee::from_value(wire).expect("legacy shape must parse");
    match fee {
        PartnerFee::Single(PartnerFeePolicy::Volume {
            volume_bps,
            recipient,
        }) => {
            assert_eq!(volume_bps, 100);
            assert_eq!(recipient.to_hex_string(), RECIPIENT_B);
        }
        other => panic!("legacy shape must map to Volume, got {other:?}"),
    }
}

#[test]
fn legacy_bps_object_renders_as_volume_on_output() {
    let wire = json!({
        "bps": 100,
        "recipient": RECIPIENT_B,
    });
    let fee = PartnerFee::from_value(wire).expect("legacy shape must parse");
    assert_eq!(
        fee.to_value(),
        json!({
            "volumeBps": 100,
            "recipient": RECIPIENT_B,
        }),
        "legacy input must emit on the wire as the modern `volumeBps` shape",
    );
}

#[test]
fn ambiguous_mixed_keys_fail_with_bad_shape() {
    let ambiguous = json!({
        "volumeBps": 1000,
        "surplusBps": 100,
        "recipient": RECIPIENT_A,
    });
    let error = PartnerFee::from_value(ambiguous).expect_err("mixed-key shape must be rejected");
    assert!(
        matches!(error, AppDataError::Json(_)),
        "ambiguous mixed-key shape must surface as a parse-level error, got {error:?}",
    );
}

#[test]
fn validate_rejects_out_of_range_volume_bps() {
    let policy = PartnerFeePolicy::Volume {
        volume_bps: 200,
        recipient: address(RECIPIENT_A),
    };
    let error = policy
        .validate()
        .expect_err("volume bps above the published cap must be rejected");
    assert!(matches!(
        error,
        AppDataError::InvalidPartnerFee {
            field: "partnerFee.volumeBps",
            reason: ValidationReason::OutOfRange { .. },
        }
    ));
}

#[test]
fn validate_rejects_zero_volume_bps() {
    let policy = PartnerFeePolicy::Volume {
        volume_bps: 0,
        recipient: address(RECIPIENT_A),
    };
    let error = policy
        .validate()
        .expect_err("zero volume bps must be rejected");
    assert!(matches!(
        error,
        AppDataError::InvalidPartnerFee {
            field: "partnerFee.volumeBps",
            reason: ValidationReason::OutOfRange { .. },
        }
    ));
}

#[test]
fn validate_rejects_out_of_range_surplus_bps() {
    let policy = PartnerFeePolicy::Surplus {
        surplus_bps: 10_000,
        max_volume_bps: 50,
        recipient: address(RECIPIENT_A),
    };
    let error = policy
        .validate()
        .expect_err("surplus bps above the published cap must be rejected");
    assert!(matches!(
        error,
        AppDataError::InvalidPartnerFee {
            field: "partnerFee.surplusBps",
            reason: ValidationReason::OutOfRange { .. },
        }
    ));
}

#[test]
fn validate_rejects_out_of_range_price_improvement_bps() {
    let policy = PartnerFeePolicy::PriceImprovement {
        price_improvement_bps: 10_000,
        max_volume_bps: 50,
        recipient: address(RECIPIENT_A),
    };
    let error = policy
        .validate()
        .expect_err("price-improvement bps above the published cap must be rejected");
    assert!(matches!(
        error,
        AppDataError::InvalidPartnerFee {
            field: "partnerFee.priceImprovementBps",
            reason: ValidationReason::OutOfRange { .. },
        }
    ));
}

#[test]
fn validate_rejects_max_volume_bps_above_cap() {
    let policy = PartnerFeePolicy::Surplus {
        surplus_bps: 500,
        max_volume_bps: 101,
        recipient: address(RECIPIENT_A),
    };
    let error = policy
        .validate()
        .expect_err("max volume bps above the published cap must be rejected");
    assert!(matches!(
        error,
        AppDataError::InvalidPartnerFee {
            field: "partnerFee.maxVolumeBps",
            reason: ValidationReason::OutOfRange { .. },
        }
    ));
}

#[test]
fn validate_accepts_boundaries_of_the_published_ranges() {
    for policy in [
        PartnerFeePolicy::Volume {
            volume_bps: 1,
            recipient: address(RECIPIENT_A),
        },
        PartnerFeePolicy::Volume {
            volume_bps: 100,
            recipient: address(RECIPIENT_A),
        },
        PartnerFeePolicy::Surplus {
            surplus_bps: 1,
            max_volume_bps: 1,
            recipient: address(RECIPIENT_A),
        },
        PartnerFeePolicy::Surplus {
            surplus_bps: 9_999,
            max_volume_bps: 100,
            recipient: address(RECIPIENT_A),
        },
        PartnerFeePolicy::PriceImprovement {
            price_improvement_bps: 1,
            max_volume_bps: 1,
            recipient: address(RECIPIENT_A),
        },
        PartnerFeePolicy::PriceImprovement {
            price_improvement_bps: 9_999,
            max_volume_bps: 100,
            recipient: address(RECIPIENT_A),
        },
    ] {
        policy
            .validate()
            .expect("boundary values of the published ranges must validate");
    }
}

#[test]
fn bounds_coverage_across_the_full_u16_range_matches_published_bounds() {
    for candidate in 0u16..=200 {
        let policy = PartnerFeePolicy::Volume {
            volume_bps: candidate,
            recipient: address(RECIPIENT_A),
        };
        let expected_ok = (1..=100).contains(&candidate);
        let outcome = policy.validate().is_ok();
        assert_eq!(
            outcome, expected_ok,
            "volume_bps={candidate} must respect the published [1, 100] range",
        );
    }

    for candidate in [0u16, 1, 100, 500, 9_998, 9_999, 10_000, 12_345, u16::MAX] {
        let policy = PartnerFeePolicy::Surplus {
            surplus_bps: candidate,
            max_volume_bps: 50,
            recipient: address(RECIPIENT_A),
        };
        let expected_ok = (1..=9_999).contains(&candidate);
        let outcome = policy.validate().is_ok();
        assert_eq!(
            outcome, expected_ok,
            "surplus_bps={candidate} must respect the published [1, 9999] range",
        );
    }
}

#[test]
fn constructors_reject_zero_address_recipient() {
    let volume_error = PartnerFeePolicy::volume(50, address(ZERO_ADDRESS))
        .expect_err("volume constructor must reject zero-address recipient");
    assert!(matches!(
        volume_error,
        AppDataError::InvalidPartnerFee {
            field: "partnerFee.recipient",
            reason: ValidationReason::Precondition { .. },
        }
    ));

    let surplus_error = PartnerFeePolicy::surplus(500, 50, address(ZERO_ADDRESS))
        .expect_err("surplus constructor must reject zero-address recipient");
    assert!(matches!(
        surplus_error,
        AppDataError::InvalidPartnerFee {
            field: "partnerFee.recipient",
            reason: ValidationReason::Precondition { .. },
        }
    ));

    let price_error = PartnerFeePolicy::price_improvement(500, 50, address(ZERO_ADDRESS))
        .expect_err("price-improvement constructor must reject zero-address recipient");
    assert!(matches!(
        price_error,
        AppDataError::InvalidPartnerFee {
            field: "partnerFee.recipient",
            reason: ValidationReason::Precondition { .. },
        }
    ));
}

#[test]
fn from_value_returns_typed_appdata_error_on_bad_input() {
    let bad: Value = json!({ "volumeBps": "not-a-number", "recipient": RECIPIENT_A });
    let error = PartnerFee::from_value(bad).expect_err("bad shape must fail closed");
    assert!(
        matches!(error, AppDataError::Json(_)),
        "bad input must surface as the typed AppDataError::Json variant, got {error:?}",
    );
}

/// Live `trybuild` harness that re-proves the narrowed-bps compile
/// failure on every `cargo test` run. A regression that silently widens
/// `volume_bps` back to `u32` fails the test rather than passing a
/// stale snapshot.
#[test]
fn partner_fee_bps_width_rejects_wider_integer_literal_at_compile_time() {
    let cases = trybuild::TestCases::new();
    cases.compile_fail("tests/ui/partner_fee_bps_width_witness.rs");
}
