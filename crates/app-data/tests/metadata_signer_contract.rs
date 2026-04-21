//! Contract suite pinning the typed `metadata.signer` surface.
//!
//! The reviewed services authority carries an optional signer address inside
//! the `metadata` envelope. This suite pins the wire shape, the round-trip
//! through [`AppDataParams`], the generated-document placement, and the
//! typed parse-time failure mode for malformed signer strings.

#![allow(
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic lint group acceptable inside integration test code"
)]

use cow_sdk_app_data::{AppDataParams, generate_app_data_doc};
use cow_sdk_core::Address;
use serde_json::{Value, json};

const SIGNER_ADDRESS: &str = "0x1111111111111111111111111111111111111111";
const SIGNER_KEY: &str = "signer";
const QUOTE_KEY: &str = "quote";

fn address(hex: &str) -> Address {
    Address::new(hex).expect("fixture address must be valid")
}

#[test]
fn typed_signer_field_emits_into_metadata_of_generated_document() {
    let params = AppDataParams {
        signer: Some(address(SIGNER_ADDRESS)),
        ..Default::default()
    };

    let doc = generate_app_data_doc(params);
    let reparsed: AppDataParams =
        serde_json::from_value(doc).expect("generated doc must round-trip through AppDataParams");
    assert_eq!(
        reparsed.signer.as_ref().map(Address::as_str),
        Some(SIGNER_ADDRESS),
        "generated document must carry the typed metadata signer",
    );
}

#[test]
fn typed_signer_field_survives_appdataparams_roundtrip() {
    let input = json!({
        "appCode": "cow-sdk",
        "metadata": {
            SIGNER_KEY: SIGNER_ADDRESS,
        }
    });

    let params: AppDataParams = serde_json::from_value(input)
        .expect("AppDataParams must parse typed signer through metadata");
    assert_eq!(
        params.signer.as_ref().map(Address::as_str),
        Some(SIGNER_ADDRESS),
    );
    assert!(
        !params.metadata.contains_key(SIGNER_KEY),
        "typed signer sub-field must leave the open-ended metadata map on deserialization",
    );

    let reserialized = serde_json::to_value(&params).expect("AppDataParams must reserialize");
    let reparsed: AppDataParams = serde_json::from_value(reserialized)
        .expect("re-serialized AppDataParams must round-trip back through deserialize");
    assert_eq!(
        reparsed.signer.as_ref().map(Address::as_str),
        Some(SIGNER_ADDRESS),
        "AppDataParams must emit metadata.signer on the wire so a re-parse recovers it",
    );
}

#[test]
fn malformed_signer_surfaces_as_typed_parse_error() {
    let input = json!({
        "metadata": {
            SIGNER_KEY: "not-an-address",
        }
    });
    let error = serde_json::from_value::<AppDataParams>(input)
        .expect_err("malformed signer must surface as a typed parse error");
    let rendered = error.to_string();
    assert!(
        rendered.contains(SIGNER_KEY) || rendered.contains("address"),
        "typed parse error must identify the failing field or typed shape, got {rendered:?}",
    );
}

#[test]
fn open_ended_metadata_keys_other_than_signer_and_flashloan_survive_roundtrip() {
    let input = json!({
        "appCode": "cow-sdk",
        "metadata": {
            SIGNER_KEY: SIGNER_ADDRESS,
            QUOTE_KEY: {
                "slippageBips": "50",
            },
        }
    });
    let params: AppDataParams = serde_json::from_value(input)
        .expect("AppDataParams must parse typed signer alongside open-ended metadata");
    assert_eq!(
        params.signer.as_ref().map(Address::as_str),
        Some(SIGNER_ADDRESS),
    );
    assert_eq!(
        params
            .metadata
            .get(QUOTE_KEY)
            .and_then(Value::as_object)
            .and_then(|quote| quote.get("slippageBips"))
            .and_then(Value::as_str),
        Some("50"),
        "open-ended metadata keys must remain inside the AppDataParams.metadata slot",
    );

    let reserialized = serde_json::to_value(&params).expect("AppDataParams must reserialize");
    let reparsed: AppDataParams = serde_json::from_value(reserialized)
        .expect("re-serialized AppDataParams must round-trip back through deserialize");
    assert_eq!(
        reparsed
            .metadata
            .get(QUOTE_KEY)
            .and_then(Value::as_object)
            .and_then(|quote| quote.get("slippageBips"))
            .and_then(Value::as_str),
        Some("50"),
        "open-ended metadata keys must survive a roundtrip through AppDataParams",
    );
}
