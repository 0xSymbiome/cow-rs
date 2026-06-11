//! Contract suite pinning the public typed hooks metadata surface.
//!
//! The fixture input mirrors the reviewed `metadata.hooks` wire shape with
//! decimal-string gas limits. These tests pin typed serde, the
//! [`AppDataParams`] lift, compatibility access through `metadata["hooks"]`,
//! and typed validation for generated hook-bearing documents.

#![allow(
    clippy::doc_markdown,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    reason = "pedantic lint group acceptable inside integration test code"
)]

use cow_sdk_app_data::{
    AppDataParams, Hook, HookList, generate_app_data_doc, validate_app_data_doc,
};
use cow_sdk_core::{AppCode, HexData};
use cow_sdk_test_utils::builders::address;
use serde_json::{Value, json};

const FIXTURE: &str = include_str!("../../../parity/fixtures/app_data/hooks_v1.14.0.json");
const HOOKS_VERSION: &str = "0.2.0";
const PRE_TARGET: &str = "0x1234567890abcdef1234567890abcdef12345678";
const POST_TARGET: &str = "0xabcdef1234567890abcdef1234567890abcdef12";
const PRE_CALL_DATA: &str = "0x01020304";
const POST_CALL_DATA: &str = "0x05060708";
const PRE_GAS_LIMIT: u64 = 100_000;
const POST_GAS_LIMIT: u64 = 125_000;
const DAPP_ID: &str = "cow-rs-hooks";

fn call_data(value: &str) -> HexData {
    HexData::new(value).expect("fixture call data must be valid")
}

fn sample_hooks() -> HookList {
    HookList::new(
        vec![
            Hook::new(address(PRE_TARGET), call_data(PRE_CALL_DATA), PRE_GAS_LIMIT)
                .with_dapp_id(DAPP_ID),
        ],
        vec![Hook::new(
            address(POST_TARGET),
            call_data(POST_CALL_DATA),
            POST_GAS_LIMIT,
        )],
    )
    .with_version(HOOKS_VERSION)
}

fn fixture_doc() -> Value {
    serde_json::from_str(FIXTURE).expect("hooks fixture must be valid JSON")
}

#[test]
fn typed_hooklist_serializes_gas_limits_as_decimal_strings() {
    let value = serde_json::to_value(sample_hooks()).expect("HookList must serialize");

    assert_eq!(value["version"], json!(HOOKS_VERSION));
    assert_eq!(value["pre"][0]["target"], json!(PRE_TARGET));
    assert_eq!(value["pre"][0]["callData"], json!(PRE_CALL_DATA));
    assert_eq!(
        value["pre"][0]["gasLimit"],
        json!(PRE_GAS_LIMIT.to_string())
    );
    assert_eq!(value["pre"][0]["dappId"], json!(DAPP_ID));
    assert_eq!(value["post"][0]["target"], json!(POST_TARGET));
    assert_eq!(value["post"][0]["callData"], json!(POST_CALL_DATA));
    assert_eq!(
        value["post"][0]["gasLimit"],
        json!(POST_GAS_LIMIT.to_string())
    );
}

#[test]
fn fixture_hook_document_roundtrips_byte_identically() {
    let doc = fixture_doc();
    let hooks_value = doc["metadata"]["hooks"].clone();
    let parsed: HookList = serde_json::from_value(hooks_value.clone())
        .expect("fixture hooks must parse into HookList");
    let reserialized = serde_json::to_value(&parsed).expect("HookList must reserialize");

    assert_eq!(
        serde_json::to_string(&reserialized).expect("reserialized hooks stringify"),
        serde_json::to_string(&hooks_value).expect("fixture hooks stringify"),
        "HookList must re-emit the fixture hooks envelope byte-identically",
    );
    assert_eq!(parsed.pre.len(), 1);
    assert_eq!(parsed.post.len(), 1);
    assert_eq!(parsed.pre[0].gas_limit, PRE_GAS_LIMIT);
    assert_eq!(parsed.post[0].gas_limit, POST_GAS_LIMIT);
}

#[test]
fn app_data_params_lifts_hooks_and_preserves_open_ended_access() {
    let doc = fixture_doc();
    let hooks_value = doc["metadata"]["hooks"].clone();

    let params: AppDataParams =
        serde_json::from_value(doc).expect("fixture document must parse through AppDataParams");

    assert_eq!(
        params.hooks.as_ref().map(|hooks| hooks.pre.len()),
        Some(1),
        "AppDataParams must expose metadata.hooks through the typed slot",
    );
    assert_eq!(
        params.metadata.get("hooks"),
        Some(&hooks_value),
        "metadata[\"hooks\"] must remain available for open-ended consumers",
    );

    let reserialized = serde_json::to_value(&params).expect("AppDataParams must serialize");
    assert_eq!(
        reserialized["metadata"]["hooks"], hooks_value,
        "typed hooks must preserve the same wire envelope on re-serialization",
    );
}

#[test]
fn generated_document_with_typed_hooks_validates() {
    let doc = generate_app_data_doc(
        AppDataParams::new(AppCode::new("hooks-contract").expect("fixture appCode must validate"))
            .with_hooks(sample_hooks()),
    );
    let validation = validate_app_data_doc(&doc);

    assert!(
        validation.is_ok(),
        "generated hook-bearing app-data document must validate, got {validation:?}",
    );
    assert_eq!(
        doc["metadata"]["hooks"]["pre"][0]["target"],
        json!(PRE_TARGET)
    );
}

#[test]
fn malformed_hook_gas_limit_rejects_at_typed_boundary() {
    let malformed = json!({
        "pre": [{
            "target": PRE_TARGET,
            "callData": PRE_CALL_DATA,
            "gasLimit": "not-a-number",
        }]
    });

    let error = serde_json::from_value::<HookList>(malformed)
        .expect_err("non-decimal gasLimit must fail typed parsing");

    let rendered = error.to_string();
    assert!(
        rendered.contains("invalid digit"),
        "typed parse error must surface the underlying decimal-string parse failure, got {error:?}",
    );
}
