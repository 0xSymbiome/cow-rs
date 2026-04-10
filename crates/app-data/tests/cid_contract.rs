mod common;

use cid::Cid;
use cow_sdk_app_data::{
    CidMode, app_data_hex_to_cid, app_data_hex_to_cid_legacy, app_data_hex_to_cid_with_mode,
    cid_to_app_data_hex,
};
use multihash::Multihash;

use crate::common::{
    APP_DATA_HEX, APP_DATA_HEX_2, APP_DATA_HEX_LEGACY, CID, CID_2, CID_LEGACY, parity_fixture,
};

const RAW_CODEC: u64 = 0x55;
const DAG_PB_CODEC: u64 = 0x70;
const KECCAK_256_CODE: u64 = 0x1b;
const SHA2_256_CODE: u64 = 0x12;

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
fn invalid_app_data_hex_inputs_fail_closed() {
    for invalid in [
        "invalidHash",
        APP_DATA_HEX.trim_start_matches("0x"),
        "0x1234",
    ] {
        assert!(
            app_data_hex_to_cid(invalid).is_err(),
            "latest conversion should reject {invalid}"
        );
        assert!(
            app_data_hex_to_cid_legacy(invalid).is_err(),
            "legacy conversion should reject {invalid}"
        );
    }
}

#[test]
fn unsupported_and_malformed_cids_are_rejected() {
    let invalid_cases = [
        ("malformed", "invalidCid".to_owned()),
        (
            "wrong digest length",
            Cid::new_v1(
                RAW_CODEC,
                Multihash::<64>::wrap(KECCAK_256_CODE, &[0x11; 31]).unwrap(),
            )
            .to_string(),
        ),
        (
            "unsupported multicodec",
            Cid::new_v1(
                DAG_PB_CODEC,
                Multihash::<64>::wrap(KECCAK_256_CODE, &[0x22; 32]).unwrap(),
            )
            .to_string(),
        ),
        (
            "unsupported multihash",
            Cid::new_v1(
                RAW_CODEC,
                Multihash::<64>::wrap(SHA2_256_CODE, &[0x33; 32]).unwrap(),
            )
            .to_string(),
        ),
    ];

    for (label, cid) in invalid_cases {
        assert!(
            cid_to_app_data_hex(&cid).is_err(),
            "cid_to_app_data_hex should reject {label}: {cid}"
        );
    }
}
