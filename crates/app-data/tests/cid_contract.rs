mod common;

use cid::Cid;
use cow_sdk_app_data::{app_data_hex_to_cid, cid_to_app_data_hex};
use multihash::Multihash;

use crate::common::{APP_DATA_HEX, APP_DATA_HEX_2, CID, CID_2, parity_fixture};

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
}

#[test]
fn latest_cid_conversion_matches_upstream_samples() {
    assert_eq!(app_data_hex_to_cid(APP_DATA_HEX).unwrap(), CID);
}

#[test]
fn cid_digest_extraction_supports_the_supported_cid_shape() {
    assert_eq!(cid_to_app_data_hex(CID).unwrap(), APP_DATA_HEX);
    assert_eq!(cid_to_app_data_hex(CID_2).unwrap(), APP_DATA_HEX_2);
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
            "conversion should reject {invalid}"
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
