#![cfg(target_arch = "wasm32")]

use cow_sdk_wasm::exports::WasmError;
use serde_json::{Value, json};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn round_trip(value: Value) -> Value {
    let js_value = serde_wasm_bindgen::to_value(&value).unwrap();
    let error: WasmError = serde_wasm_bindgen::from_value(js_value).unwrap();
    serde_wasm_bindgen::from_value(serde_wasm_bindgen::to_value(&error).unwrap()).unwrap()
}

#[wasm_bindgen_test]
fn invalid_input_variant_round_trips() {
    let value = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "invalidInput",
        "message": "invalid address",
        "field": "owner"
    }));

    assert_eq!(value["schemaVersion"], "v1");
    assert_eq!(value["kind"], "invalidInput");
    assert_eq!(value["field"], "owner");
}

#[wasm_bindgen_test]
fn unknown_enum_variant_round_trips() {
    let value = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "unknownEnumValue",
        "field": "kind",
        "value": "swap"
    }));

    assert_eq!(value["kind"], "unknownEnumValue");
    assert_eq!(value["value"], "swap");
}

#[wasm_bindgen_test]
fn unsupported_chain_variant_round_trips() {
    let value = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "unsupportedChain",
        "chainId": 13337
    }));

    assert_eq!(value["kind"], "unsupportedChain");
    assert_eq!(value["chainId"], 13337);
}

#[wasm_bindgen_test]
fn wallet_request_variant_round_trips() {
    let value = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "walletRequest",
        "method": "eth_signTypedData_v4",
        "code": 4001,
        "message": "user rejected",
        "data": { "reason": "denied" }
    }));

    assert_eq!(value["kind"], "walletRequest");
    assert_eq!(value["code"], 4001);
    assert_eq!(value["data"]["reason"], "denied");
}

#[wasm_bindgen_test]
fn transport_variant_round_trips() {
    let value = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "transport",
        "class": "status",
        "message": "HTTP 500",
        "status": 500,
        "headers": [["x-request-id", "[redacted]"]],
        "body": "[redacted]"
    }));

    assert_eq!(value["kind"], "transport");
    assert_eq!(value["status"], 500);
    assert_eq!(value["body"], "[redacted]");
}

#[wasm_bindgen_test]
fn orderbook_subgraph_signing_and_app_data_variants_round_trip() {
    let orderbook = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "orderbook",
        "code": "422",
        "message": "order rejected"
    }));
    let subgraph = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "subgraph",
        "message": "query failed"
    }));
    let signing = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "signing",
        "message": "signature invalid"
    }));
    let app_data = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "appData",
        "class": "decode",
        "message": "document invalid"
    }));

    assert_eq!(orderbook["kind"], "orderbook");
    assert_eq!(subgraph["kind"], "subgraph");
    assert_eq!(signing["kind"], "signing");
    assert_eq!(app_data["kind"], "appData");
}

#[wasm_bindgen_test]
fn forbidden_interaction_variant_round_trips() {
    let value = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "forbiddenInteraction",
        "target": "0x1111111111111111111111111111111111111111",
        "reason": "forbidden settlement interaction target"
    }));

    assert_eq!(value["kind"], "forbiddenInteraction");
    assert_eq!(
        value["target"],
        "0x1111111111111111111111111111111111111111"
    );
}

#[wasm_bindgen_test]
fn contracts_forbidden_interaction_maps_to_typed_error() {
    let target = cow_sdk_core::Address::new("0x1111111111111111111111111111111111111111").unwrap();
    let error =
        WasmError::from(cow_sdk_contracts::ContractsError::ForbiddenInteractionTarget { target });
    let value: Value = serde_wasm_bindgen::from_value(error.into_js()).unwrap();

    assert_eq!(value["schemaVersion"], "v1");
    assert_eq!(value["kind"], "forbiddenInteraction");
    assert_eq!(
        value["target"],
        "0x1111111111111111111111111111111111111111"
    );
}

#[wasm_bindgen_test]
fn cancelled_variant_has_schema_version_only() {
    let value = round_trip(json!({ "schemaVersion": "v1", "kind": "cancelled" }));

    assert_eq!(value, json!({ "schemaVersion": "v1", "kind": "cancelled" }));
}

#[wasm_bindgen_test]
fn unknown_sentinel_round_trips_raw_payload() {
    let value = round_trip(json!({
        "schemaVersion": "__unknown",
        "kind": "__unknown",
        "raw": { "kind": "futureVariant", "detail": "unknown" }
    }));

    assert_eq!(value["schemaVersion"], "__unknown");
    assert_eq!(value["kind"], "__unknown");
    assert_eq!(value["raw"]["kind"], "futureVariant");
}

#[wasm_bindgen_test]
fn internal_variant_carries_opaque_message() {
    let value = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "internal",
        "message": "serialization failed"
    }));

    assert_eq!(value["kind"], "internal");
    assert_eq!(value["message"], "serialization failed");
}

#[wasm_bindgen_test]
fn malformed_kind_is_rejected_without_panic() {
    let js_value = serde_wasm_bindgen::to_value(&json!({
        "schemaVersion": "v1",
        "kind": "futureVariant",
        "message": "unknown"
    }))
    .unwrap();
    let decoded = serde_wasm_bindgen::from_value::<WasmError>(js_value);

    assert!(decoded.is_err());
}

#[wasm_bindgen_test]
fn optional_fields_are_omitted_when_absent() {
    let value = round_trip(json!({
        "schemaVersion": "v1",
        "kind": "invalidInput",
        "message": "invalid input"
    }));

    assert_eq!(value["kind"], "invalidInput");
    assert!(value.get("field").is_none());
}
