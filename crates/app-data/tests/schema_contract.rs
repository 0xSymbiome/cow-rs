mod common;

use cow_sdk_app_data::{
    AppDataParams, FlashloanHints, LATEST_APP_DATA_VERSION, generate_app_data_doc,
    validate_app_data_doc,
};
use cow_sdk_core::{Address, Amount, AppCode};
use serde_json::{Value, json};

use crate::common::app_data_doc_custom;

#[test]
fn generation_follows_pinned_contract() {
    let generated = generate_app_data_doc(AppDataParams::default());
    assert_eq!(generated["version"], json!(LATEST_APP_DATA_VERSION));
    assert_eq!(generated["appCode"], json!("CoW Swap"));
    assert_eq!(generated["metadata"], json!({}));
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
fn validation_accepts_modelled_docs_and_rejects_out_of_range_quote() {
    let valid = validate_app_data_doc(&app_data_doc_custom());
    assert!(valid.is_ok(), "{valid:?}");

    let error = validate_app_data_doc(&json!({
        "version": LATEST_APP_DATA_VERSION,
        "appCode": "cow-rs",
        "metadata": { "quote": { "slippageBips": 20_000 } }
    }))
    .expect_err("an out-of-range quote slippage must be rejected");
    assert!(error.to_string().contains("slippageBips"), "{error:?}");
}

#[test]
fn modelled_and_unmodelled_metadata_families_validate_through_the_document_surface() {
    // An unmodelled family passes through untouched.
    let bridging = json!({
        "version": "1.5.0",
        "metadata": {
            "bridging": {
                "destinationTokenAddress": "0x00E989b87700514118Fa55326CD1cCE82faebEF6",
                "destinationChainId": "42161"
            }
        }
    });
    assert!(validate_app_data_doc(&bridging).is_ok());

    // A modelled flashloan hint in its current shape validates.
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
    assert!(validate_app_data_doc(&flashloan).is_ok());

    // The typed `FlashloanHints` surface produces a document that validates
    // through the same typed bound checks.
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
    let generated = generate_app_data_doc(params);
    assert!(validate_app_data_doc(&generated).is_ok());
}

#[test]
fn unmodelled_wrappers_doc_passes_through_and_preserves_typed_fields() {
    let wrappers = json!({
        "version": "1.13.0",
        "appCode": "euler",
        "metadata": {
            "wrappers": [
                {
                    "address": "0x74399a40D9FE2478e82058480F426D7e5783167c",
                    "data": "0x000000000000000000000000f39fd6e51aad88f6f4ce6ab8827279cfffb92266",
                    "isOmittable": false
                }
            ]
        }
    });

    let validation = validate_app_data_doc(&wrappers);
    assert!(
        validation.is_ok(),
        "an unmodelled wrappers document must pass through, got {validation:?}",
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
}

#[test]
fn non_semver_version_is_rejected_without_leaking_the_value() {
    let secret_bearing_doc = json!({
        "version": "Bearer eyJleHAiOiAibGVha19jaGVjayJ9",
        "metadata": {}
    });
    let error = validate_app_data_doc(&secret_bearing_doc)
        .expect_err("a non-semver version must be rejected");
    let rendered = error.to_string();
    assert!(
        !rendered.contains("Bearer"),
        "rendered error must not include the caller-supplied version value; got {rendered:?}",
    );
    assert!(
        !rendered.contains("eyJleHAiOiAibGVha19jaGVjayJ9"),
        "rendered error must not include the caller-supplied version value; got {rendered:?}",
    );
}
