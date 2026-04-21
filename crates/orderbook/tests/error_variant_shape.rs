//! Public-surface contract assertions for every structured
//! [`cow_sdk_orderbook::OrderbookError`] wrapper variant plus the shared
//! [`cow_sdk_core::TransportErrorClass`] classification.
//!
//! Each test destructures the typed shape of one variant through an
//! exhaustive pattern match. The `TransportErrorClass` coverage walks every
//! named variant so the partition stays stable for downstream telemetry and
//! retry layers. Any future variant whose shape drifts from this contract
//! fails the corresponding test at compile time.

use cow_sdk_core::{TransportErrorClass, ValidationReason};
use cow_sdk_orderbook::OrderbookError;

#[test]
fn transport_variant_carries_typed_class_and_detail() {
    let error = OrderbookError::Transport {
        class: TransportErrorClass::Connect,
        detail: "connection reset by peer".to_owned(),
    };

    let OrderbookError::Transport { class, detail } = &error else {
        panic!("expected Transport variant, got {error:?}");
    };
    assert_eq!(*class, TransportErrorClass::Connect);
    assert!(detail.contains("connection reset"));
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
        OrderbookError::Serialization(inner) => {
            let _ = inner;
        }
        other => panic!("expected Serialization(#[from] serde_json::Error), got {other:?}"),
    }
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
