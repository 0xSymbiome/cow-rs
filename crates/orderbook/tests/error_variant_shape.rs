//! Public-surface contract assertions for every structured
//! [`cow_sdk_orderbook::OrderbookError`] wrapper variant plus the shared
//! [`cow_sdk_core::TransportErrorClass`] classification.
//!
//! Each test destructures the typed shape of one variant through an
//! exhaustive pattern match. The `TransportErrorClass` coverage walks every
//! named variant so the partition stays stable for downstream telemetry and
//! retry layers. Any future variant whose shape drifts from this contract
//! fails the corresponding test at compile time.

use cow_sdk_core::{AppDataHash, TransportErrorClass, ValidationReason};
use cow_sdk_orderbook::{HashMismatchStage, OrderbookError};

#[test]
fn transport_variant_carries_typed_class_and_detail() {
    let error = OrderbookError::Transport {
        class: TransportErrorClass::Connect,
        detail: "connection reset by peer".to_owned().into(),
    };

    let OrderbookError::Transport { class, detail } = &error else {
        panic!("expected Transport variant, got {error:?}");
    };
    assert_eq!(*class, TransportErrorClass::Connect);
    assert!(detail.as_inner().contains("connection reset"));
    assert!(
        error.to_string().contains("connect"),
        "transport Display must include the typed class label",
    );
}

#[test]
fn serialization_variant_wraps_serde_json_error_via_from_conversion() {
    let source = serde_json::from_str::<serde_json::Value>("{ malformed").unwrap_err();
    let error: OrderbookError = source.into();

    match &error {
        OrderbookError::Serialization {
            category,
            line,
            column,
        } => {
            assert_eq!(*category, "syntax");
            assert!(*line >= 1 && *column >= 1);
        }
        other => panic!("expected Serialization variant, got {other:?}"),
    }
    assert!(
        error.to_string().contains("serialization error (syntax)"),
        "Display must surface the structural category, not the serde message: {error}",
    );
}

#[test]
fn invalid_trades_query_carries_structured_field_and_reason() {
    let error = OrderbookError::InvalidTradesQuery {
        field: "filter",
        reason: ValidationReason::Precondition {
            details: "exactly one of owner or orderUid must be set",
        },
    };

    let OrderbookError::InvalidTradesQuery { field, reason } = &error else {
        panic!("expected InvalidTradesQuery variant, got {error:?}");
    };
    assert_eq!(*field, "filter");
    assert!(matches!(reason, ValidationReason::Precondition { .. }));
}

#[test]
fn invalid_quote_request_carries_structured_field_and_reason() {
    let error = OrderbookError::InvalidQuoteRequest {
        field: "side",
        reason: ValidationReason::Precondition {
            details: "exactly one of sellAmountBeforeFee or buyAmountAfterFee must be set",
        },
    };

    let OrderbookError::InvalidQuoteRequest { field, reason } = &error else {
        panic!("expected InvalidQuoteRequest variant, got {error:?}");
    };
    assert_eq!(*field, "side");
    assert!(matches!(reason, ValidationReason::Precondition { .. }));
}

#[test]
fn invalid_transform_carries_structured_field_and_reason() {
    let error = OrderbookError::InvalidTransform {
        field: "executedFee",
        reason: ValidationReason::BadShape {
            details: "expected unsigned decimal string",
        },
    };

    let OrderbookError::InvalidTransform { field, reason } = &error else {
        panic!("expected InvalidTransform variant, got {error:?}");
    };
    assert_eq!(*field, "executedFee");
    assert!(matches!(reason, ValidationReason::BadShape { .. }));
}

#[test]
fn app_data_hash_mismatch_carries_typed_hashes_and_stage_discriminator() {
    let expected = AppDataHash::from_full_app_data("{}");
    let observed = AppDataHash::ZERO;

    let client_precheck = OrderbookError::AppDataHashMismatch {
        expected,
        observed,
        stage: HashMismatchStage::ClientPrecheck,
    };
    let OrderbookError::AppDataHashMismatch {
        expected: e_pre,
        observed: o_pre,
        stage: s_pre,
    } = &client_precheck
    else {
        panic!("expected AppDataHashMismatch variant, got {client_precheck:?}");
    };
    assert_eq!(*e_pre, expected);
    assert_eq!(*o_pre, observed);
    assert_eq!(*s_pre, HashMismatchStage::ClientPrecheck);
    let pre_display = client_precheck.to_string();
    assert!(
        pre_display.contains("client precheck"),
        "Display must surface the stage discriminator label: {pre_display}",
    );
    assert!(
        pre_display.contains(&expected.to_hex_string()),
        "Display must surface the typed expected hash: {pre_display}",
    );

    let server_echo = OrderbookError::AppDataHashMismatch {
        expected,
        observed,
        stage: HashMismatchStage::ServerEcho,
    };
    assert!(
        server_echo.to_string().contains("server echo"),
        "ServerEcho Display must surface the stage discriminator label",
    );
}

#[test]
fn transport_error_class_labels_are_stable_across_all_named_variants() {
    for (class, expected) in [
        (TransportErrorClass::Timeout, "timeout"),
        (TransportErrorClass::Connect, "connect"),
        (TransportErrorClass::Redirect, "redirect"),
        (TransportErrorClass::Decode, "decode"),
        (TransportErrorClass::Body, "body"),
        (TransportErrorClass::Builder, "builder"),
        (TransportErrorClass::Request, "request"),
        (TransportErrorClass::Status, "status"),
        (TransportErrorClass::Other, "other"),
    ] {
        assert_eq!(class.as_str(), expected);
        assert_eq!(class.to_string(), expected);
    }
}
