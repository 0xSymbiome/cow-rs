mod common;

use cow_sdk_app_data::{
    AppDataParams, FlashloanHints, LATEST_APP_DATA_VERSION, generate_app_data_doc,
    get_app_data_schema, validate_app_data_doc,
};
use cow_sdk_core::{Address, Amount, AppCode};
use serde_json::{Value, json};

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
    let unknown = get_app_data_schema("0.0.0").unwrap_err();
    assert_eq!(unknown.to_string(), "AppData version 0.0.0 doesn't exist");
    let invalid = get_app_data_schema("not semver").unwrap_err();
    assert_eq!(
        invalid.to_string(),
        "AppData version [redacted] is not a valid version"
    );
}

#[test]
fn app_data_params_builders_preserve_top_level_wire_fields() {
    let params = AppDataParams::new(
        AppCode::new("solver-integration").expect("fixture appCode must validate"),
    )
    .with_environment("staging-canary");

    assert_eq!(
        params.app_code.as_ref().map(AppCode::as_str),
        Some("solver-integration")
    );
    assert_eq!(params.environment.as_deref(), Some("staging-canary"));

    let generated = generate_app_data_doc(params);
    assert_eq!(generated["appCode"], json!("solver-integration"));
    assert_eq!(generated["environment"], json!("staging-canary"));
    assert_eq!(generated["metadata"], json!({}));
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

    // Assert the typed `FlashloanHints` surface produces a document that
    // still validates against the bundled `flashloan/v0.2.0` sub-schema.
    let hints = FlashloanHints::new(
        Address::new("0xb50201558B00496A145fE76f7424749556E326D8").unwrap(),
        Address::new("0x1186B5ad42E3e6d6c6901FC53b4A367540E6EcFE").unwrap(),
        Address::new("0x1186B5ad42E3e6d6c6901FC53b4A367540E6EcFE").unwrap(),
        Address::new("0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d").unwrap(),
        Amount::new("2000000000000000000").unwrap(),
    )
    .expect("typed flashloan hints must validate");
    let params = AppDataParams::new(
        AppCode::new("aave-v3-flashloan").expect("fixture appCode must validate"),
    )
    .with_flashloan(hints);
    let mut generated = generate_app_data_doc(params);
    if let Value::Object(map) = &mut generated {
        map.insert("version".to_owned(), Value::String("1.7.0".to_owned()));
    }
    assert!(validate_app_data_doc(&generated).success);

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

#[test]
fn wrappers_v1_13_0_minimal_doc_validates_and_extracts_typed_fields() {
    let wrappers = json!({
        "version": "1.13.0",
        "appCode": "euler",
        "metadata": {
            "wrappers": [
                {
                    "address": "0x74399a40D9FE2478e82058480F426D7e5783167c",
                    "data": "0x000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb9226600000000000000000000000000000000000000000000000000000000ff123456000000000000000000000d8b27cf359b7da5be299af6e7bf904984c2000000000000000000000797dd80692c3b2dadabce8e30c07fde5307d48a90000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000000000005f5e100",
                    "isOmittable": false
                }
            ]
        }
    });

    let validation = validate_app_data_doc(&wrappers);
    assert!(
        validation.success,
        "v1.13.0 wrappers fixture must validate, got {:?}",
        validation.errors,
    );

    let wrapper = wrappers
        .get("metadata")
        .and_then(Value::as_object)
        .and_then(|metadata| metadata.get("wrappers"))
        .and_then(Value::as_array)
        .and_then(|wrappers| wrappers.first())
        .and_then(Value::as_object)
        .expect("fixture carries one wrappers entry");
    assert_eq!(
        wrapper.get("address").and_then(Value::as_str),
        Some("0x74399a40D9FE2478e82058480F426D7e5783167c")
    );
    assert_eq!(
        wrapper.get("isOmittable").and_then(Value::as_bool),
        Some(false)
    );
    assert!(
        wrapper
            .get("data")
            .and_then(Value::as_str)
            .is_some_and(|data| data.starts_with("0x")),
        "wrapper calldata must remain hex-encoded",
    );
}

#[test]
fn schema_error_message_masks_failing_instance_values() {
    let secret_bearing_doc = json!({
        "version": "Bearer eyJleHAiOiAibGVha19jaGVjayJ9",
        "metadata": {}
    });
    let result = validate_app_data_doc(&secret_bearing_doc);
    assert!(!result.success);
    let errors = result
        .errors
        .as_deref()
        .expect("schema validation failure must carry a rendered error");
    assert!(
        !errors.contains("Bearer"),
        "masked validator output must not include caller-supplied instance values; got {errors:?}",
    );
    assert!(
        !errors.contains("eyJleHAiOiAibGVha19jaGVjayJ9"),
        "masked validator output must not include caller-supplied instance values; got {errors:?}",
    );
}

#[test]
fn schema_error_message_does_not_leak_unexpected_property_names() {
    let extra_prop_doc = json!({
        "version": "1.14.0",
        "appCode": "cow-rs",
        "metadata": {
            "flashloan": {
                "leak_check_secret_property_name": "value",
                "liquidityProvider": "0xb50201558B00496A145fE76f7424749556E326D8",
                "protocolAdapter": "0x1186B5ad42E3e6d6c6901FC53b4A367540E6EcFE",
                "receiver": "0x1186B5ad42E3e6d6c6901FC53b4A367540E6EcFE",
                "token": "0xe91D153E0b41518A2Ce8Dd3D7944Fa863463a97d",
                "amount": "1"
            }
        }
    });
    let result = validate_app_data_doc(&extra_prop_doc);
    if !result.success {
        let errors = result
            .errors
            .as_deref()
            .expect("schema validation failure must carry a rendered error");
        assert!(
            !errors.contains("leak_check_secret_property_name"),
            "additional-properties failure must not include caller-supplied property names; got {errors:?}",
        );
    }
}
