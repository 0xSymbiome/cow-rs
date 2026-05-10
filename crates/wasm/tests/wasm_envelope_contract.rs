#![cfg(target_arch = "wasm32")]

use cow_sdk_wasm::exports::{SchemaVersion, WasmEnvelope};
use serde_json::{Value, json};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn json_value(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

#[wasm_bindgen_test]
fn envelope_serializes_schema_version_and_payload() {
    let envelope = WasmEnvelope::v1(json!({ "ok": true }));
    let value = json_value(serde_wasm_bindgen::to_value(&envelope).unwrap());

    assert_eq!(value["schemaVersion"], "v1");
    assert_eq!(value["value"]["ok"], true);
}

#[wasm_bindgen_test]
fn envelope_preserves_unknown_schema_sentinel() {
    let envelope = WasmEnvelope {
        schema_version: SchemaVersion::Unknown,
        value: json!({ "future": true }),
    };
    let value = json_value(serde_wasm_bindgen::to_value(&envelope).unwrap());

    assert_eq!(value["schemaVersion"], "__unknown");
    assert_eq!(value["value"]["future"], true);
}
