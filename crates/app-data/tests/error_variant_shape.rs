//! Public-surface contract assertions for every structured
//! [`cow_sdk_app_data::AppDataError`] wrapper variant.
//!
//! Each test destructures the typed shape of one variant through an
//! exhaustive pattern match. The `Json` variant wraps [`serde_json::Error`]
//! through a `#[from]` converter; `InvalidAppDataProvided` carries
//! `{ field, reason: ValidationReason }`; `Calculation` carries a typed
//! `Box<dyn Error>` source so the underlying cid or multihash failure stays
//! addressable; and `Transport` carries `{ class: TransportErrorClass, detail }`.
//! Any future variant whose shape drifts from this contract fails the
//! corresponding test at compile time.

use cow_sdk_app_data::AppDataError;
use cow_sdk_core::{TransportErrorClass, ValidationReason};

#[test]
fn json_variant_wraps_serde_json_error_via_from_conversion() {
    let source = serde_json::from_str::<serde_json::Value>("{ malformed").unwrap_err();
    let error: AppDataError = source.into();

    match &error {
        AppDataError::Json(inner) => {
            let _ = inner;
        }
        other => panic!("expected Json(#[from] serde_json::Error), got {other:?}"),
    }
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
