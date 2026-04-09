mod common;

use cow_sdk_app_data::{
    CidMode, app_data_hex_to_cid, app_data_hex_to_cid_legacy, app_data_hex_to_cid_with_mode,
    cid_to_app_data_hex,
};

use crate::common::{
    APP_DATA_HEX, APP_DATA_HEX_2, APP_DATA_HEX_LEGACY, CID, CID_2, CID_LEGACY, parity_fixture,
};

#[test]
fn cid_surface_matches_fixture_contract() {
    let fixture = parity_fixture();
    assert_eq!(fixture["surface"].as_str().unwrap(), "app-data");
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "app-data-cid-v1-conversion")
    );
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "app-data-legacy-cid-compatibility")
    );
}

#[test]
fn latest_and_legacy_cid_conversion_match_upstream_samples() {
    assert_eq!(app_data_hex_to_cid(APP_DATA_HEX).unwrap(), CID);
    assert_eq!(
        app_data_hex_to_cid_legacy(APP_DATA_HEX_LEGACY).unwrap(),
        CID_LEGACY
    );
    assert_eq!(
        app_data_hex_to_cid_with_mode(APP_DATA_HEX, CidMode::Latest).unwrap(),
        CID
    );
    assert_eq!(
        app_data_hex_to_cid_with_mode(APP_DATA_HEX_LEGACY, CidMode::Legacy).unwrap(),
        CID_LEGACY
    );
}

#[test]
fn cid_digest_extraction_supports_latest_and_legacy_inputs() {
    assert_eq!(cid_to_app_data_hex(CID).unwrap(), APP_DATA_HEX);
    assert_eq!(cid_to_app_data_hex(CID_2).unwrap(), APP_DATA_HEX_2);
    assert_eq!(
        cid_to_app_data_hex(CID_LEGACY).unwrap(),
        APP_DATA_HEX_LEGACY
    );
}

#[test]
fn malformed_hex_and_cid_are_rejected() {
    assert!(app_data_hex_to_cid("invalidHash").is_err());
    assert!(app_data_hex_to_cid_legacy("0x1234").is_err());
    assert!(cid_to_app_data_hex("invalidCid").is_err());
}
