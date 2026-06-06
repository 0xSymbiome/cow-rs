//! Contract suite pinning the typed `AppDataValidated` return shape for
//! `app_data_info`, including the `Deref<Target = AppDataInfo>`
//! ergonomics, the near-limit `ApproachingSizeLimit` warning, the
//! unchanged hard `AppDataError::TooLarge` path, and the property that
//! `bytes_used` matches the stringified deterministic payload length.

#![allow(
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic lint group acceptable inside integration test code"
)]

use cow_sdk_app_data::{
    APP_DATA_APPROACHING_LIMIT_RATIO, APP_DATA_MAX_BYTES, AppDataError, AppDataWarning,
    app_data_info,
};

const DOC_OVERHEAD: &str =
    r#"{"appCode":"","environment":"production","metadata":{},"version":"1.3.0"}"#;

fn document_of_size(bytes: usize) -> String {
    let overhead = DOC_OVERHEAD.len();
    assert!(
        bytes >= overhead,
        "requested byte size {bytes} must leave room for the wrapping document"
    );
    let filler_size = bytes - overhead;
    let filler = "a".repeat(filler_size);
    format!(
        r#"{{"appCode":"{filler}","environment":"production","metadata":{{}},"version":"1.3.0"}}"#
    )
}

#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "threshold byte size fits back inside usize with the floor the warning contract requires"
)]
fn approaching_threshold() -> usize {
    (APP_DATA_MAX_BYTES as f64 * APP_DATA_APPROACHING_LIMIT_RATIO) as usize
}

#[test]
fn payload_just_below_threshold_emits_no_warning() {
    let threshold = approaching_threshold();
    let doc = document_of_size(threshold - 1);
    let validated = app_data_info(doc).expect("sub-threshold payload must succeed");
    assert_eq!(validated.validation.bytes_used, threshold - 1);
    assert!(
        validated.validation.warnings.is_empty(),
        "sub-threshold payload must carry no soft warnings, got {:?}",
        validated.validation.warnings,
    );
}

#[test]
fn payload_at_threshold_emits_approaching_size_limit_warning() {
    let threshold = approaching_threshold();
    let doc = document_of_size(threshold);
    let validated = app_data_info(doc).expect("at-threshold payload must succeed");
    assert_eq!(validated.validation.bytes_used, threshold);
    assert_eq!(
        validated.validation.warnings.len(),
        1,
        "at-threshold payload must emit exactly one soft warning",
    );
    assert!(matches!(
        validated.validation.warnings[0],
        AppDataWarning::ApproachingSizeLimit {
            bytes_used,
            max_bytes,
        } if bytes_used == threshold && max_bytes == APP_DATA_MAX_BYTES,
    ));
}

#[test]
fn payload_at_ceiling_still_emits_approaching_size_limit_warning() {
    let doc = document_of_size(APP_DATA_MAX_BYTES);
    let validated = app_data_info(doc).expect("at-ceiling payload must still succeed");
    assert_eq!(validated.validation.bytes_used, APP_DATA_MAX_BYTES);
    assert!(matches!(
        validated.validation.warnings.first(),
        Some(AppDataWarning::ApproachingSizeLimit {
            bytes_used,
            max_bytes,
        }) if *bytes_used == APP_DATA_MAX_BYTES && *max_bytes == APP_DATA_MAX_BYTES,
    ));
}

#[test]
fn payload_above_ceiling_fails_with_too_large_and_never_constructs_the_validated_wrapper() {
    let doc = document_of_size(APP_DATA_MAX_BYTES + 1);
    let error = app_data_info(doc).expect_err("oversized payload must fail");
    match error {
        AppDataError::TooLarge {
            actual_bytes,
            max_bytes,
        } => {
            assert_eq!(max_bytes, APP_DATA_MAX_BYTES);
            assert_eq!(actual_bytes, APP_DATA_MAX_BYTES + 1);
        }
        other => panic!("expected AppDataError::TooLarge, got {other:?}"),
    }
}

#[test]
fn deref_preserves_field_access_for_existing_callers() {
    let doc = document_of_size(256);
    let validated = app_data_info(doc).expect("small payload must succeed");
    // Field read through Deref auto-deref must match the explicit inner access.
    assert_eq!(validated.app_data_hex, validated.info.app_data_hex);
    assert_eq!(validated.cid, validated.info.cid);
    assert_eq!(validated.app_data_content, validated.info.app_data_content);
    // Methods on the inner AppDataInfo fields remain callable via Deref.
    assert!(validated.app_data_hex.starts_with("0x"));
    assert!(!validated.app_data_content.is_empty());
}

#[test]
fn bytes_used_equals_the_stringified_deterministic_payload_length() {
    for size in [256usize, 1024, 2048, approaching_threshold() - 1] {
        let doc = document_of_size(size);
        let expected_bytes = doc.len();
        let validated = app_data_info(doc)
            .unwrap_or_else(|error| panic!("payload of {size} bytes must succeed, got {error:?}"));
        assert_eq!(
            validated.validation.bytes_used, expected_bytes,
            "bytes_used must equal the stringified deterministic payload length",
        );
        assert_eq!(
            validated.info.app_data_content.len(),
            expected_bytes,
            "the stored app_data_content must match the reported bytes_used",
        );
    }
}
