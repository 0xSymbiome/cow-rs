//! Fixture-driven contract for the typed orderbook rejection parser.
//!
//! Services encodes every non-2xx response with the universal
//! `{ "errorType": "<tag>", "description": "<message>", "data": ...? }`
//! envelope. The tests below pin every services-authoritative tag from
//! a JSON envelope to the typed [`OrderbookRejection`] variant produced
//! by [`parse_rejection`], plus the permanent `Unknown { code, message }`
//! fallback with sanitized code and redacted message rendering, the `None`
//! outcome on non-envelope bodies, and the
//! `From<OrderBookApiError>` path that promotes the envelope into
//! [`OrderbookError::Rejected`] inside the SDK transport stack.
//!
//! Authoritative spellings are sourced from the handler files under the
//! services orderbook crate; the fixtures below match those spellings
//! byte-for-byte. A dedicated regression case pins that the retired
//! `DuplicateOrder` typo (which shipped in prior cow-rs tests) is
//! classified as the permanent `Unknown` fallback, so new services
//! renames fail closed rather than silently reusing the wrong typed
//! variant.

#![allow(
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "test helper code may exercise a large number of wire variants in a single table"
)]

use cow_sdk_core::{Amount, REDACTED_PLACEHOLDER};
use cow_sdk_orderbook::{
    OrderBookApiError, OrderbookError, OrderbookRejection, ResponseBody, parse_rejection,
};
use http::StatusCode;
use serde_json::json;

fn envelope_bytes(error_type: &str, description: &str) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "errorType": error_type,
        "description": description,
    }))
    .expect("envelope must serialize")
}

fn envelope_with_data(error_type: &str, description: &str, data: &serde_json::Value) -> Vec<u8> {
    serde_json::to_vec(&json!({
        "errorType": error_type,
        "description": description,
        "data": data,
    }))
    .expect("envelope with data must serialize")
}

fn assert_message_carrying_rejection_contract(
    tag: &str,
    status: StatusCode,
    description: &str,
    expected: &OrderbookRejection,
    expected_display: &str,
) {
    let body = envelope_bytes(tag, description);
    let rejection = parse_rejection(status, &body)
        .unwrap_or_else(|| panic!("tag {tag} must classify through the typed parser"));

    assert_eq!(
        &rejection, expected,
        "tag {tag} must preserve the services description behind explicit access",
    );
    assert_eq!(
        rejection.to_string(),
        expected_display,
        "tag {tag} must preserve the reviewed redacted Display contract",
    );
}

#[test]
fn every_known_services_tag_parses_to_its_typed_variant() {
    let cases: &[(&str, StatusCode, OrderbookRejection)] = &[
        (
            "DuplicatedOrder",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::DuplicatedOrder,
        ),
        (
            "OldOrderActivelyBidOn",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::OldOrderActivelyBidOn,
        ),
        (
            "QuoteNotFound",
            StatusCode::NOT_FOUND,
            OrderbookRejection::QuoteNotFound,
        ),
        (
            "QuoteNotVerified",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::QuoteNotVerified,
        ),
        (
            "MissingFrom",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::MissingFrom,
        ),
        (
            "WrongOwner",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::WrongOwner,
        ),
        (
            "InvalidEip1271Signature",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::InvalidEip1271Signature,
        ),
        (
            "InvalidSignature",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::InvalidSignature,
        ),
        (
            "IncompatibleSigningScheme",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::IncompatibleSigningScheme,
        ),
        (
            "InsufficientBalance",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::InsufficientBalance,
        ),
        (
            "InsufficientAllowance",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::InsufficientAllowance,
        ),
        (
            "ZeroAmount",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::ZeroAmount,
        ),
        (
            "NonZeroFee",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::NonZeroFee,
        ),
        (
            "SellAmountOverflow",
            StatusCode::INTERNAL_SERVER_ERROR,
            OrderbookRejection::SellAmountOverflow,
        ),
        (
            "TooMuchGas",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::TooMuchGas,
        ),
        (
            "TooManyLimitOrders",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::TooManyLimitOrders,
        ),
        (
            "TransferSimulationFailed",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::TransferSimulationFailed,
        ),
        (
            "InsufficientValidTo",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::InsufficientValidTo,
        ),
        (
            "ExcessiveValidTo",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::ExcessiveValidTo,
        ),
        (
            "InvalidNativeSellToken",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::InvalidNativeSellToken,
        ),
        (
            "SameBuyAndSellToken",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::SameBuyAndSellToken,
        ),
        (
            "UnsupportedToken",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::UnsupportedToken,
        ),
        (
            "UnsupportedBuyTokenDestination",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::UnsupportedBuyTokenDestination,
        ),
        (
            "UnsupportedSellTokenSource",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::UnsupportedSellTokenSource,
        ),
        (
            "UnsupportedOrderType",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::UnsupportedOrderType,
        ),
        (
            "AppDataInvalid",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::AppDataInvalid {
                message: "services-authoritative description".to_owned().into(),
            },
        ),
        (
            "InvalidAppData",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::InvalidAppData,
        ),
        (
            "AppDataHashMismatch",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::AppDataHashMismatch,
        ),
        (
            "AppDataMismatch",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::AppDataMismatch {
                message: "services-authoritative description".to_owned().into(),
            },
        ),
        (
            "AppdataFromMismatch",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::AppdataFromMismatch,
        ),
        (
            "MetadataSerializationFailed",
            StatusCode::INTERNAL_SERVER_ERROR,
            OrderbookRejection::MetadataSerializationFailed,
        ),
        (
            "NoLiquidity",
            StatusCode::NOT_FOUND,
            OrderbookRejection::NoLiquidity,
        ),
        (
            "TradingOutsideAllowedWindow",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::TradingOutsideAllowedWindow,
        ),
        (
            "TokenTemporarilySuspended",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::TokenTemporarilySuspended,
        ),
        (
            "InsufficientLiquidity",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::InsufficientLiquidity,
        ),
        (
            "CustomSolverError",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::CustomSolverError,
        ),
        (
            "InvalidTradeFilter",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::InvalidTradeFilter,
        ),
        (
            "InvalidLimit",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::InvalidLimit,
        ),
        (
            "LIMIT_OUT_OF_BOUNDS",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::LimitOutOfBounds,
        ),
        (
            "AlreadyCancelled",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::AlreadyCancelled,
        ),
        (
            "OrderFullyExecuted",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::OrderFullyExecuted,
        ),
        (
            "OrderExpired",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::OrderExpired,
        ),
        (
            "OrderNotFound",
            StatusCode::NOT_FOUND,
            OrderbookRejection::OrderNotFound,
        ),
        (
            "NotFound",
            StatusCode::NOT_FOUND,
            OrderbookRejection::NotFound {
                message: "services-authoritative description".to_owned().into(),
            },
        ),
        (
            "OnChainOrder",
            StatusCode::BAD_REQUEST,
            OrderbookRejection::OnChainOrder,
        ),
        (
            "Forbidden",
            StatusCode::FORBIDDEN,
            OrderbookRejection::Forbidden,
        ),
        (
            "InternalServerError",
            StatusCode::INTERNAL_SERVER_ERROR,
            OrderbookRejection::InternalServerError,
        ),
    ];

    for (tag, status, expected) in cases {
        let body = envelope_bytes(tag, "services-authoritative description");
        let actual = parse_rejection(*status, &body).unwrap_or_else(|| {
            panic!("tag {tag} must classify through the typed parser rather than returning None")
        });
        assert_eq!(
            &actual, expected,
            "tag {tag} must classify to its typed variant",
        );
    }
}

#[test]
fn app_data_invalid_tag_preserves_typed_message_and_display() {
    let description = "appData is invalid: missing protocol metadata";
    assert_message_carrying_rejection_contract(
        "AppDataInvalid",
        StatusCode::BAD_REQUEST,
        description,
        &OrderbookRejection::AppDataInvalid {
            message: description.to_owned().into(),
        },
        "AppDataInvalid: [redacted]",
    );
}

#[test]
fn app_data_mismatch_tag_preserves_typed_message_and_display() {
    let description =
        "stored appData \"{\\\"version\\\":\\\"1.0.0\\\"}\" is different than the specified data";
    assert_message_carrying_rejection_contract(
        "AppDataMismatch",
        StatusCode::BAD_REQUEST,
        description,
        &OrderbookRejection::AppDataMismatch {
            message: description.to_owned().into(),
        },
        "AppDataMismatch: [redacted]",
    );
}

#[test]
fn not_found_tag_preserves_typed_message_and_display() {
    let description = "Order was not found";
    assert_message_carrying_rejection_contract(
        "NotFound",
        StatusCode::NOT_FOUND,
        description,
        &OrderbookRejection::NotFound {
            message: description.to_owned().into(),
        },
        "NotFound: [redacted]",
    );
}

#[test]
fn sell_amount_does_not_cover_fee_parses_typed_fee_amount_from_the_data_field() {
    let body = envelope_with_data(
        "SellAmountDoesNotCoverFee",
        "sell amount does not cover fee",
        &json!({ "fee_amount": "12345" }),
    );
    let rejection = parse_rejection(StatusCode::BAD_REQUEST, &body)
        .expect("SellAmountDoesNotCoverFee envelope must classify");

    let expected = Amount::new("12345").expect("static fee amount must remain valid");
    assert_eq!(
        rejection,
        OrderbookRejection::SellAmountDoesNotCoverFee {
            fee_amount: expected,
        },
    );
}

#[test]
fn sell_amount_does_not_cover_fee_falls_back_to_unknown_when_data_shape_drifts() {
    let body = envelope_with_data(
        "SellAmountDoesNotCoverFee",
        "sell amount does not cover fee",
        &json!({ "unexpected_shape": true }),
    );
    let rejection = parse_rejection(StatusCode::BAD_REQUEST, &body)
        .expect("SellAmountDoesNotCoverFee envelope must still classify");

    match rejection {
        OrderbookRejection::Unknown { code, .. } => {
            assert_eq!(code.as_str(), "SellAmountDoesNotCoverFee");
        }
        other => panic!(
            "unknown data shape must surface as Unknown, not {:?}",
            other,
        ),
    }
}

#[test]
fn unknown_services_tag_surfaces_as_unknown_with_preserved_code_and_message() {
    let body = envelope_bytes("NotYetDefined", "services added this in a future release");
    let rejection = parse_rejection(StatusCode::BAD_REQUEST, &body)
        .expect("well-formed envelope must classify even when the tag is unknown");

    match &rejection {
        OrderbookRejection::Unknown { code, message } => {
            assert_eq!(code.as_str(), "NotYetDefined");
            assert_eq!(
                message.as_inner(),
                "services added this in a future release"
            );
            assert_eq!(
                rejection.to_string(),
                "unknown rejection code `NotYetDefined`: [redacted]"
            );
        }
        other => panic!(
            "unknown services tag must surface as Unknown, not {:?}",
            other,
        ),
    }
}

#[test]
fn duplicate_order_typo_is_classified_as_unknown_not_as_duplicated_order() {
    let body = envelope_bytes("DuplicateOrder", "typo that should never silently match");
    let rejection = parse_rejection(StatusCode::BAD_REQUEST, &body)
        .expect("well-formed envelope must classify even when the tag is unknown");

    match rejection {
        OrderbookRejection::Unknown { code, .. } => {
            assert_eq!(
                code.as_str(),
                "DuplicateOrder",
                "the DuplicateOrder typo must surface as the literal unknown tag rather than the typed DuplicatedOrder variant",
            );
        }
        other => panic!(
            "DuplicateOrder typo must surface as Unknown, not {:?}",
            other,
        ),
    }
}

#[test]
fn secret_shaped_unknown_rejection_code_is_sanitized_before_public_rendering() {
    let body = envelope_bytes(
        "https://user:pass@example.com/path?key=secret",
        "services added this in a future release",
    );
    let rejection = parse_rejection(StatusCode::BAD_REQUEST, &body)
        .expect("well-formed envelope must classify even when the tag is unsafe");

    match &rejection {
        OrderbookRejection::Unknown { code, message } => {
            assert_eq!(code.as_str(), REDACTED_PLACEHOLDER);
            assert_eq!(
                message.as_inner(),
                "services added this in a future release"
            );
            assert_eq!(
                rejection.to_string(),
                "unknown rejection code `[redacted]`: [redacted]"
            );
        }
        other => panic!(
            "unsafe unknown services tag must surface as Unknown, not {:?}",
            other,
        ),
    }
}

#[test]
fn malformed_body_returns_none_so_the_caller_surfaces_a_transport_error() {
    let body = b"this body is not JSON";
    assert!(
        parse_rejection(StatusCode::BAD_REQUEST, body).is_none(),
        "non-envelope bodies must yield None so callers surface them as transport failures",
    );
}

#[test]
fn body_missing_error_type_field_returns_none() {
    let body = serde_json::to_vec(&json!({ "description": "no errorType" }))
        .expect("partial envelope must serialize");
    assert!(
        parse_rejection(StatusCode::BAD_REQUEST, &body).is_none(),
        "envelopes without an `errorType` field must yield None",
    );
}

#[test]
fn from_api_error_promotes_recognised_body_to_rejected_variant() {
    let api_error = OrderBookApiError::new(
        400,
        "Bad Request",
        ResponseBody::Json(json!({
            "errorType": "DuplicatedOrder",
            "description": "order already exists",
        })),
    );

    match OrderbookError::from(api_error) {
        OrderbookError::Rejected {
            status,
            rejection,
            source,
        } => {
            assert_eq!(status, StatusCode::BAD_REQUEST);
            assert_eq!(rejection, OrderbookRejection::DuplicatedOrder);
            assert_eq!(source.status, 400);
        }
        other => panic!("expected Rejected, got {:?}", other),
    }
}

#[test]
fn from_api_error_falls_back_to_api_when_body_has_no_envelope() {
    let api_error = OrderBookApiError::new(500, "Internal Server Error", ResponseBody::Empty);

    match OrderbookError::from(api_error) {
        OrderbookError::Api(envelope) => {
            assert_eq!(envelope.status, 500);
        }
        other => panic!(
            "bodies without the rejection envelope must stay on Api, not {:?}",
            other,
        ),
    }
}
