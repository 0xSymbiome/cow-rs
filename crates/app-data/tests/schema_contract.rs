mod common;

use cow_sdk_app_data::{
    AppDataParams, LATEST_APP_DATA_VERSION, generate_app_data_doc, get_app_data_schema,
    validate_app_data_doc,
};
use serde_json::json;

use crate::common::{app_data_doc_custom, invalid_referrer_doc, parity_fixture};

#[test]
fn generation_and_schema_lookup_follow_pinned_contract() {
    let fixture = parity_fixture();
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "app-data-schema-lookup-contract")
    );

    let generated = generate_app_data_doc(AppDataParams::default());
    assert_eq!(generated["version"], json!(LATEST_APP_DATA_VERSION));
    assert_eq!(generated["appCode"], json!("CoW Swap"));
    assert_eq!(generated["metadata"], json!({}));

    let schema = get_app_data_schema("0.4.0").unwrap();
    assert!(schema["$id"].as_str().unwrap().contains("0.4.0"));
    assert_eq!(
        get_app_data_schema("0.0.0").unwrap_err().to_string(),
        "AppData version 0.0.0 doesn't exist"
    );
    assert_eq!(
        get_app_data_schema("not semver").unwrap_err().to_string(),
        "AppData version not semver is not a valid version"
    );
}

#[test]
fn validation_supports_latest_docs_and_rejects_invalid_metadata() {
    let valid = validate_app_data_doc(&app_data_doc_custom());
    assert!(valid.success, "{valid:?}");

    let invalid = validate_app_data_doc(&invalid_referrer_doc());
    assert!(!invalid.success);
    assert!(
        invalid
            .errors
            .as_deref()
            .unwrap()
            .contains("/metadata/referrer/address")
    );
}

#[test]
fn schema_regression_families_are_supported() {
    let fixture = parity_fixture();
    assert!(
        fixture["cases"]
            .as_array()
            .unwrap()
            .iter()
            .any(|case| case["id"] == "app-data-schema-regression-families")
    );

    let bridging = json!({
        "version": "1.5.0",
        "metadata": {
            "bridging": {
                "destinationTokenAddress": "0x00E989b87700514118Fa55326CD1cCE82faebEF6",
                "destinationChainId": "42161"
            }
        }
    });
    assert!(validate_app_data_doc(&bridging).success);

    let flashloan = json!({
        "version": "1.7.0",
        "appCode": "aave-v3-flashloan",
        "metadata": {
            "flashloan": {
                "amount": "2000000000000000000",
                "liquidityProvider": "0xb50201558B00496A145fE76f7424749556E326D8",
                "protocolAdapter": "0x1186B5ad42E3e6d6c6901FC53b4A367540E6EcFE",
                "receiver": "0x1186B5ad42E3e6d6c6901FC53b4A367540E6EcFE",
                "token": "0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d"
            }
        }
    });
    assert!(validate_app_data_doc(&flashloan).success);

    let wrappers = json!({
        "version": "1.13.0",
        "appCode": "euler",
        "metadata": {
            "wrappers": [
                {
                    "address": "0x74399a40D9FE2478e82058480F426D7e5783167c",
                    "data": "0x000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb9226600000000000000000000000000000000000000000000000000000000ff123456000000000000000000000000d8b27cf359b7da5be299af6e7bf904984c2000000000000000000000000797dd80692c3b2dadabce8e30c07fde5307d48a90000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000000005f5e100",
                    "isOmittable": false
                }
            ]
        }
    });
    assert!(validate_app_data_doc(&wrappers).success);
}
