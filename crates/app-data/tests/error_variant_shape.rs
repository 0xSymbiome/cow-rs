//! Public-surface regressions for the typed shape of every structured
//! [`cow_sdk_app_data::AppDataError`] wrapper variant.
//!
//! The six wrappers that previously held arbitrary external error strings
//! (`Json`, `Schema`, `InvalidAppDataProvided`, `Calculation`, `Transport`,
//! `Pinning`) now carry either a typed `#[from]` converter or a structured
//! validation field set. Each test destructures the current shape through an
//! exhaustive pattern match; any regression to a `(String)` payload fails
//! this file at compile time.

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
fn schema_variant_carries_structured_message_field() {
    let error = AppDataError::Schema {
        message: "draft-07 reference missing `$id`".to_owned(),
    };

    let AppDataError::Schema { message } = &error else {
        panic!("expected Schema variant, got {error:?}");
    };
    assert!(message.contains("reference missing"));
}

#[test]
fn invalid_app_data_provided_carries_structured_field_and_reason() {
    let error = AppDataError::InvalidAppDataProvided {
        field: "document",
        reason: ValidationReason::BadShape {
            details: "document failed the embedded JSON schema validation",
        },
    };

    let AppDataError::InvalidAppDataProvided { field, reason } = &error else {
        panic!("expected InvalidAppDataProvided variant, got {error:?}");
    };
    assert_eq!(*field, "document");
    assert!(matches!(reason, ValidationReason::BadShape { .. }));
}

#[test]
fn calculation_variant_carries_structured_message_field() {
    let error = AppDataError::Calculation {
        message: "multihash length overflow".to_owned(),
    };

    let AppDataError::Calculation { message } = &error else {
        panic!("expected Calculation variant, got {error:?}");
    };
    assert!(message.contains("multihash"));
}

#[test]
fn transport_variant_carries_typed_class_and_detail() {
    let error = AppDataError::Transport {
        class: TransportErrorClass::Timeout,
        detail: "ipfs gateway did not respond within the configured timeout".to_owned(),
    };

    let AppDataError::Transport { class, detail } = &error else {
        panic!("expected Transport variant, got {error:?}");
    };
    assert_eq!(*class, TransportErrorClass::Timeout);
    assert!(detail.contains("ipfs gateway"));
    assert!(
        error.to_string().contains("timeout"),
        "transport Display must include the typed class label",
    );
}

#[test]
fn pinning_variant_carries_optional_status_and_message() {
    let error = AppDataError::Pinning {
        status: Some(401),
        message: "unauthorized".to_owned(),
    };

    let AppDataError::Pinning { status, message } = &error else {
        panic!("expected Pinning variant, got {error:?}");
    };
    assert_eq!(*status, Some(401));
    assert_eq!(message, "unauthorized");

    let error_without_status = AppDataError::Pinning {
        status: None,
        message: "pinning backend unreachable".to_owned(),
    };
    let AppDataError::Pinning { status, .. } = &error_without_status else {
        panic!("expected Pinning variant, got {error_without_status:?}");
    };
    assert!(status.is_none());
}
