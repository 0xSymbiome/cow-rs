mod common;

use cow_sdk_app_data::{AppDataError, app_data_info};
use serde_json::json;

use crate::common::{
    APP_DATA_HEX, APP_DATA_HEX_2, APP_DATA_STRING, APP_DATA_STRING_2, CID, CID_2, app_data_doc,
};

#[test]
fn get_app_data_info_matches_pinned_upstream_samples() {
    let from_string = app_data_info(APP_DATA_STRING).unwrap();
    assert_eq!(from_string.cid, CID);
    assert_eq!(from_string.app_data_hex, APP_DATA_HEX);
    assert_eq!(from_string.app_data_content, APP_DATA_STRING);

    let from_doc = app_data_info(app_data_doc()).unwrap();
    assert_eq!(from_doc, from_string);

    let secondary = app_data_info(APP_DATA_STRING_2).unwrap();
    assert_eq!(secondary.cid, CID_2);
    assert_eq!(secondary.app_data_hex, APP_DATA_HEX_2);
    assert_eq!(secondary.app_data_content, APP_DATA_STRING_2);
}

#[test]
fn invalid_documents_fail_through_typed_error_surface() {
    let invalid = json!({
        "version": "1.14.0",
        "metadata": {
            "quote": {
                "slippageBips": 20_000
            }
        }
    });

    let error = app_data_info(invalid).unwrap_err();
    assert!(matches!(error, AppDataError::InvalidAppDataProvided { .. }));
}

// The at-ceiling acceptance (+ the `APP_DATA_MAX_BYTES == 8192` pin and the
// approaching-limit warning) and the one-byte-over `TooLarge` rejection are
// owned by `validated_shape_contract.rs`, which additionally asserts the
// validated wrapper is never constructed past the ceiling.
