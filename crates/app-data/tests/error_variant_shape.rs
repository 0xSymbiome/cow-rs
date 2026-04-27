//! Public-surface contract assertions for every structured
//! [`cow_sdk_app_data::AppDataError`] wrapper variant.
//!
//! Each test destructures the typed shape of one variant through an
//! exhaustive pattern match. The `Json` variant wraps [`serde_json::Error`]
//! through a `#[from]` converter; `Schema` carries a path-prefixed display
//! message paired with a typed [`jsonschema::ValidationError`] source;
//! `InvalidAppDataProvided` carries `{ field, reason: ValidationReason }`;
//! `Calculation` carries a typed `Box<dyn Error>` source so the underlying
//! cid or multihash failure stays addressable; `Transport` carries
//! `{ class: TransportErrorClass, detail }`; and `Pinning` carries
//! `{ status: Option<u16>, message }`. Any future variant whose shape drifts
//! from this contract fails the corresponding test at compile time.

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
fn schema_variant_wraps_jsonschema_validation_error_through_typed_source() {
    let schema = serde_json::json!({"type": "object", "required": ["x"]});
    let candidate = serde_json::json!({});
    let validator = jsonschema::validator_for(&schema).expect("schema fixture must compile");
    let validation_error = validator
        .iter_errors(&candidate)
        .next()
        .expect("missing-required-property must surface a validation error")
        .to_owned();
    let error = AppDataError::Schema {
        message: format!("data {validation_error}"),
        source: Box::new(validation_error),
    };

    let AppDataError::Schema { message, source } = &error else {
        panic!("expected Schema variant, got {error:?}");
    };
    assert!(message.contains("required"));
    assert!(format!("{source}").contains("required"));
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
        message: "unauthorized".to_owned().into(),
    };

    let AppDataError::Pinning { status, message } = &error else {
        panic!("expected Pinning variant, got {error:?}");
    };
    assert_eq!(*status, Some(401));
    assert_eq!(message.as_inner(), "unauthorized");

    let error_without_status = AppDataError::Pinning {
        status: None,
        message: "pinning backend unreachable".to_owned().into(),
    };
    let AppDataError::Pinning { status, .. } = &error_without_status else {
        panic!("expected Pinning variant, got {error_without_status:?}");
    };
    assert!(status.is_none());
}
