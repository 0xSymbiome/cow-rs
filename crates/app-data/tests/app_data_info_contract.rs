mod common;

use cow_sdk_app_data::{
    APP_DATA_MAX_BYTES, AppDataError, app_data_hex_to_cid_legacy, get_app_data_info,
    get_app_data_info_legacy,
};
use serde_json::json;

use crate::common::{
    APP_DATA_HEX, APP_DATA_HEX_2, APP_DATA_STRING, APP_DATA_STRING_2, APP_DATA_STRING_LEGACY, CID,
    CID_2, app_data_doc, parity_fixture,
};

#[test]
fn get_app_data_info_matches_pinned_upstream_samples() {
    let fixture = parity_fixture();
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "app-data-get-app-data-info-deterministic")
    );

    let from_string = get_app_data_info(APP_DATA_STRING).unwrap();
    assert_eq!(from_string.cid, CID);
    assert_eq!(from_string.app_data_hex, APP_DATA_HEX);
    assert_eq!(from_string.app_data_content, APP_DATA_STRING);

    let from_doc = get_app_data_info(app_data_doc()).unwrap();
    assert_eq!(from_doc, from_string);

    let secondary = get_app_data_info(APP_DATA_STRING_2).unwrap();
    assert_eq!(secondary.cid, CID_2);
    assert_eq!(secondary.app_data_hex, APP_DATA_HEX_2);
    assert_eq!(secondary.app_data_content, APP_DATA_STRING_2);
}

#[test]
fn legacy_info_flow_remains_explicit_and_compatible() {
    let legacy = get_app_data_info_legacy(APP_DATA_STRING_LEGACY).unwrap();
    assert!(legacy.cid.starts_with("Qm"));
    assert_eq!(
        app_data_hex_to_cid_legacy(&legacy.app_data_hex).unwrap(),
        legacy.cid
    );
    assert_eq!(legacy.app_data_content, APP_DATA_STRING_LEGACY);
}

#[test]
fn invalid_documents_fail_through_typed_error_surface() {
    let invalid = json!({
        "version": "0.4.0",
        "metadata": {
            "referrer": {
                "version": "312313",
                "address": "0xssss"
            }
        }
    });

    let error = get_app_data_info(invalid).unwrap_err();
    assert!(matches!(error, AppDataError::InvalidAppDataProvided(_)));
}

#[test]
fn app_data_size_guard_accepts_exactly_the_configured_maximum() {
    assert_eq!(APP_DATA_MAX_BYTES, 8192);

    let overhead_with_empty_filler =
        r#"{"appCode":"","environment":"production","metadata":{},"version":"1.3.0"}"#.len();
    let filler_size = APP_DATA_MAX_BYTES - overhead_with_empty_filler;
    let filler = "a".repeat(filler_size);
    let at_limit_doc = format!(
        r#"{{"appCode":"{filler}","environment":"production","metadata":{{}},"version":"1.3.0"}}"#
    );
    assert_eq!(
        at_limit_doc.len(),
        APP_DATA_MAX_BYTES,
        "constructed document must match the configured ceiling exactly"
    );

    let accepted = get_app_data_info(at_limit_doc);
    assert!(
        accepted.is_ok(),
        "documents at exactly the configured ceiling must pass the size guard: {accepted:?}"
    );
}

#[test]
fn app_data_size_guard_rejects_documents_above_the_configured_maximum() {
    let overhead_with_empty_filler =
        r#"{"appCode":"","environment":"production","metadata":{},"version":"1.3.0"}"#.len();
    let filler_size = APP_DATA_MAX_BYTES - overhead_with_empty_filler + 1;
    let filler = "a".repeat(filler_size);
    let oversized_doc = format!(
        r#"{{"appCode":"{filler}","environment":"production","metadata":{{}},"version":"1.3.0"}}"#
    );
    assert_eq!(
        oversized_doc.len(),
        APP_DATA_MAX_BYTES + 1,
        "constructed document must sit one byte past the configured ceiling"
    );

    let rejected = get_app_data_info(oversized_doc).unwrap_err();
    match rejected {
        AppDataError::TooLarge {
            actual_bytes,
            max_bytes,
        } => {
            assert_eq!(max_bytes, APP_DATA_MAX_BYTES);
            assert_eq!(
                actual_bytes,
                APP_DATA_MAX_BYTES + 1,
                "TooLarge must surface the exact oversized byte count"
            );
        }
        other => panic!("expected AppDataError::TooLarge, got {other:?}"),
    }
}
