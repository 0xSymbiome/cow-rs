//! Public-surface contract assertions for every structured
//! [`cow_sdk_app_data::AppDataError`] wrapper variant.
//!
//! Each test destructures the typed shape of one variant through an
//! exhaustive pattern match. The `Json` variant captures only the serde
//! failure `{ category, line, column }` through a manual `From` converter that
//! drops the raw `serde_json::Error` text (ADR 0025); `InvalidAppDataProvided`
//! carries
//! `{ field, reason: ValidationReason }`; `Calculation` carries a typed
//! `Box<dyn Error>` source so the underlying cid or multihash failure stays
//! addressable; and `Transport` carries `{ class: TransportErrorClass, detail }`.
//! Any future variant whose shape drifts from this contract fails the
//! corresponding test at compile time.

use cow_sdk_app_data::AppDataError;
use cow_sdk_core::{TransportErrorClass, ValidationReason};

#[test]
fn json_variant_drops_raw_serde_error_for_structured_position() {
    let source = serde_json::from_str::<serde_json::Value>("{ malformed").unwrap_err();
    let error: AppDataError = source.into();

    let AppDataError::Json {
        category,
        line,
        column,
    } = &error
    else {
        panic!("expected Json {{ category, line, column }}, got {error:?}");
    };
    assert_eq!(*category, "syntax");
    assert!(*line >= 1 && *column >= 1);
    // The structured diagnostic never renders the raw serde error text that
    // could echo decoded document bytes (ADR 0025).
    assert_eq!(
        error.to_string(),
        format!("json error ({category}) at line {line} column {column}"),
    );
}

#[test]
fn invalid_app_data_provided_carries_structured_field_and_reason() {
    let error = AppDataError::InvalidAppDataProvided {
        field: "document",
        reason: ValidationReason::BadShape {
            details: "document failed typed metadata validation",
        },
    };

    let AppDataError::InvalidAppDataProvided { field, reason } = &error else {
        panic!("expected InvalidAppDataProvided variant, got {error:?}");
    };
    assert_eq!(*field, "document");
    assert!(matches!(reason, ValidationReason::BadShape { .. }));
}

#[test]
fn calculation_variant_carries_typed_source_through_box_dyn_error() {
    #[derive(Debug, thiserror::Error)]
    #[error("synthetic multihash failure: {0}")]
    struct StubMultihashFailure(&'static str);

    let error = AppDataError::Calculation {
        source: Box::new(StubMultihashFailure("multihash length overflow")),
    };

    let AppDataError::Calculation { source } = &error else {
        panic!("expected Calculation variant, got {error:?}");
    };
    assert!(format!("{source}").contains("multihash"));
}

#[test]
fn transport_variant_carries_typed_class_and_detail() {
    let error = AppDataError::Transport {
        class: TransportErrorClass::Timeout,
        detail: "ipfs gateway did not respond within the configured timeout"
            .to_owned()
            .into(),
    };

    let AppDataError::Transport { class, detail } = &error else {
        panic!("expected Transport variant, got {error:?}");
    };
    assert_eq!(*class, TransportErrorClass::Timeout);
    assert!(detail.as_inner().contains("ipfs gateway"));
    assert!(
        error.to_string().contains("timeout"),
        "transport Display must include the typed class label",
    );
}
