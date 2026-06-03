//! Contract suite pinning the public typed flash-loan hint surface.
//!
//! The fixture input mirrors the reviewed golden sample for the
//! `flashloan/v0.2.0.json` sub-schema so drift in either the wire shape or
//! the construction-time validation rules surfaces before it reaches
//! release.

#![allow(
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic lint group acceptable inside integration test code"
)]

use cow_sdk_app_data::{
    AppDataError, AppDataParams, FlashloanHints, generate_app_data_doc, validate_app_data_doc,
};
use cow_sdk_core::{Address, Amount, AppCode, ValidationReason};

fn test_app_code() -> AppCode {
    AppCode::new("aave-v3-flashloan").expect("fixture appCode must validate")
}
use serde_json::{Value, json};

const FIXTURE_PATH: &str = "../../parity/fixtures/app_data/flashloan_v1.7.0.json";
// The reviewed golden sample is now stored in the canonical lowercase
// 0x-prefixed wire form per PROP-WB-004; the cow Address newtype canonicalizes
// every input to lowercase at construction (ADR 0052) so the fixture and the
// constants below stay aligned with the runtime serialization shape.
const LIQUIDITY_PROVIDER: &str = "0xb50201558b00496a145fe76f7424749556e326d8";
const PROTOCOL_ADAPTER: &str = "0x1186b5ad42e3e6d6c6901fc53b4a367540e6ecfe";
const RECEIVER: &str = "0x1186b5ad42e3e6d6c6901fc53b4a367540e6ecfe";
const TOKEN: &str = "0xe91d153e0b41518a2ce8dd3d7944fa863463a97d";
const AMOUNT: &str = "2000000000000000000";
const ZERO_ADDRESS: &str = "0x0000000000000000000000000000000000000000";

fn address(hex: &str) -> Address {
    Address::new(hex).expect("fixture address must be valid")
}

fn amount(value: &str) -> Amount {
    Amount::new(value).expect("fixture amount must be valid")
}

fn sample_hints() -> FlashloanHints {
    FlashloanHints::new(
        address(LIQUIDITY_PROVIDER),
        address(PROTOCOL_ADAPTER),
        address(RECEIVER),
        address(TOKEN),
        amount(AMOUNT),
    )
    .expect("sample hints must validate")
}

#[test]
fn fixture_golden_sample_roundtrips_byte_identically() {
    let fixture_text = std::fs::read_to_string(FIXTURE_PATH)
        .expect("flash-loan fixture sample must remain pinned in the tree");
    let wire: Value =
        serde_json::from_str(&fixture_text).expect("fixture sample must be valid JSON");

    let parsed: FlashloanHints = serde_json::from_value(wire.clone())
        .expect("fixture sample must parse into FlashloanHints");
    let reserialized = serde_json::to_value(&parsed)
        .expect("parsed FlashloanHints must reserialize through serde");

    assert_eq!(
        reserialized, wire,
        "FlashloanHints must roundtrip the reviewed fixture byte-identically",
    );
    assert_eq!(parsed.amount.to_string(), AMOUNT);
    assert_eq!(
        parsed.liquidity_provider.to_hex_string(),
        LIQUIDITY_PROVIDER
    );
    assert_eq!(parsed.protocol_adapter.to_hex_string(), PROTOCOL_ADAPTER);
    assert_eq!(parsed.receiver.to_hex_string(), RECEIVER);
    assert_eq!(parsed.token.to_hex_string(), TOKEN);
}

#[test]
fn valid_hints_validate_and_match_the_golden_sample_shape() {
    let hints = sample_hints();
    hints.validate().expect("valid hints must validate");
    assert_eq!(hints.amount.to_string(), AMOUNT);
    assert_eq!(hints.liquidity_provider.to_hex_string(), LIQUIDITY_PROVIDER);
}

#[test]
fn zero_liquidity_provider_is_rejected_as_bad_shape() {
    let error = FlashloanHints::new(
        address(ZERO_ADDRESS),
        address(PROTOCOL_ADAPTER),
        address(RECEIVER),
        address(TOKEN),
        amount(AMOUNT),
    )
    .expect_err("zero liquidityProvider must fail validation");
    assert!(matches!(
        error,
        AppDataError::InvalidFlashloanHints {
            field: "liquidityProvider",
            reason: ValidationReason::BadShape { .. },
        }
    ));
}

#[test]
fn zero_protocol_adapter_is_rejected_as_bad_shape() {
    let error = FlashloanHints::new(
        address(LIQUIDITY_PROVIDER),
        address(ZERO_ADDRESS),
        address(RECEIVER),
        address(TOKEN),
        amount(AMOUNT),
    )
    .expect_err("zero protocolAdapter must fail validation");
    assert!(matches!(
        error,
        AppDataError::InvalidFlashloanHints {
            field: "protocolAdapter",
            reason: ValidationReason::BadShape { .. },
        }
    ));
}

#[test]
fn zero_receiver_is_rejected_as_bad_shape() {
    let error = FlashloanHints::new(
        address(LIQUIDITY_PROVIDER),
        address(PROTOCOL_ADAPTER),
        address(ZERO_ADDRESS),
        address(TOKEN),
        amount(AMOUNT),
    )
    .expect_err("zero receiver must fail validation");
    assert!(matches!(
        error,
        AppDataError::InvalidFlashloanHints {
            field: "receiver",
            reason: ValidationReason::BadShape { .. },
        }
    ));
}

#[test]
fn zero_token_is_rejected_as_bad_shape() {
    let error = FlashloanHints::new(
        address(LIQUIDITY_PROVIDER),
        address(PROTOCOL_ADAPTER),
        address(RECEIVER),
        address(ZERO_ADDRESS),
        amount(AMOUNT),
    )
    .expect_err("zero token must fail validation");
    assert!(matches!(
        error,
        AppDataError::InvalidFlashloanHints {
            field: "token",
            reason: ValidationReason::BadShape { .. },
        }
    ));
}

#[test]
fn flashloan_hint_appears_inside_metadata_of_generated_document() {
    let params = AppDataParams::new(test_app_code()).with_flashloan(sample_hints());

    let doc = generate_app_data_doc(params);
    let flashloan = doc
        .get("metadata")
        .and_then(Value::as_object)
        .and_then(|metadata| metadata.get("flashloan"))
        .expect("generated document must carry metadata.flashloan");

    assert_eq!(flashloan["amount"], json!(AMOUNT));
    assert_eq!(flashloan["liquidityProvider"], json!(LIQUIDITY_PROVIDER));
    assert_eq!(flashloan["protocolAdapter"], json!(PROTOCOL_ADAPTER));
    assert_eq!(flashloan["receiver"], json!(RECEIVER));
    assert_eq!(flashloan["token"], json!(TOKEN));
}

#[test]
fn validation_accepts_documents_carrying_the_typed_flashloan_hint() {
    let params = AppDataParams::new(test_app_code()).with_flashloan(sample_hints());

    let mut doc = generate_app_data_doc(params);
    if let Value::Object(map) = &mut doc {
        map.insert("version".to_owned(), Value::String("1.7.0".to_owned()));
    }

    let validation = validate_app_data_doc(&doc);
    assert!(
        validation.success,
        "generated document carrying typed FlashloanHints must validate, got {:?}",
        validation.errors,
    );
}

#[test]
fn typed_flashloan_field_survives_appdataparams_roundtrip() {
    let params = AppDataParams::default().with_flashloan(sample_hints());

    let json_value =
        serde_json::to_value(&params).expect("AppDataParams with typed flashloan must serialize");
    assert_eq!(
        json_value
            .get("metadata")
            .and_then(Value::as_object)
            .and_then(|metadata| metadata.get("flashloan"))
            .and_then(|flashloan| flashloan.get("amount"))
            .and_then(Value::as_str),
        Some(AMOUNT),
        "AppDataParams must emit `metadata.flashloan.amount` on the wire",
    );

    let reparsed: AppDataParams = serde_json::from_value(json_value)
        .expect("serialized AppDataParams must reparse through the custom deserializer");
    assert_eq!(
        reparsed
            .flashloan
            .as_ref()
            .map(|hints| hints.amount.to_string()),
        Some(AMOUNT.to_owned()),
        "typed flashloan field must survive a roundtrip through AppDataParams",
    );
    assert!(
        !reparsed.metadata.contains_key("flashloan"),
        "typed flashloan sub-field must leave the open-ended metadata map on deserialization",
    );
}

fn flashloan_v1_7_0_document() -> Value {
    json!({
        "version": "1.7.0",
        "appCode": "aave-v3-flashloan",
        "metadata": {
            "flashloan": {
                "amount": AMOUNT,
                "liquidityProvider": LIQUIDITY_PROVIDER,
                "protocolAdapter": PROTOCOL_ADAPTER,
                "receiver": RECEIVER,
                "token": TOKEN
            }
        }
    })
}

#[test]
fn flashloan_v1_7_0_rejects_invalid_address() {
    let mut doc = flashloan_v1_7_0_document();
    doc["metadata"]["flashloan"]["liquidityProvider"] = json!("not-an-address");

    let validation = validate_app_data_doc(&doc);
    assert!(!validation.success);
    assert!(
        validation
            .errors
            .as_deref()
            .is_some_and(|errors| errors.contains("flashloan")),
        "validation error must identify the flashloan family, got {:?}",
        validation.errors,
    );
}

#[test]
fn flashloan_v1_7_0_rejects_zero_amount() {
    let error = FlashloanHints::new(
        address(LIQUIDITY_PROVIDER),
        address(PROTOCOL_ADAPTER),
        address(RECEIVER),
        address(TOKEN),
        Amount::ZERO,
    )
    .expect_err("zero amount must fail validation");
    assert!(matches!(
        error,
        AppDataError::InvalidFlashloanHints {
            field: "amount",
            reason: ValidationReason::OutOfRange { .. },
        }
    ));
}

#[test]
fn flashloan_v1_7_0_rejects_missing_field() {
    let mut doc = flashloan_v1_7_0_document();
    doc["metadata"]["flashloan"]
        .as_object_mut()
        .expect("flashloan fixture is an object")
        .remove("receiver");

    let validation = validate_app_data_doc(&doc);
    assert!(!validation.success);
    assert!(
        validation
            .errors
            .as_deref()
            .is_some_and(|errors| errors.contains("flashloan")),
        "validation error must identify the flashloan family, got {:?}",
        validation.errors,
    );
}
