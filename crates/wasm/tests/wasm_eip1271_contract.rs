#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_pure_helpers as pure;
use cow_sdk_wasm::exports::{
    ResolvedEip1271Provider, compute_order_uid, eip1271_signature_payload_export,
    sign_order_with_custom_eip1271, sign_order_with_eip1271,
};
use js_sys::Function;
use serde::Deserialize;
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use crate::common::{
    ADDR_OWNER, CHAIN_MAINNET, ECDSA_SIGNATURE, EIP1271_SIGNATURE, wasm_order_input,
};

wasm_bindgen_test_configure!(run_in_browser);

#[derive(Debug, Deserialize)]
struct UpstreamVector {
    ecdsa_signature: String,
    eip1271_signature: String,
}

fn callback(args: &str, body: &str) -> Function {
    Function::new_with_args(args, body)
}

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

fn expected_uid() -> String {
    json(compute_order_uid(wasm_order_input(), CHAIN_MAINNET, ADDR_OWNER.to_owned()).unwrap())
        ["value"]["orderUid"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn envelope_string(value: JsValue) -> String {
    json(value)["value"].as_str().unwrap().to_owned()
}

#[wasm_bindgen_test]
fn eip1271_payload_matches_native_rust() {
    let order = pure::dto::OrderInput::from(wasm_order_input())
        .to_unsigned_order()
        .unwrap();
    let native = cow_sdk_signing::eip1271_signature_payload(&order, ECDSA_SIGNATURE).unwrap();
    let exported = envelope_string(
        eip1271_signature_payload_export(wasm_order_input(), ECDSA_SIGNATURE.to_owned()).unwrap(),
    );

    assert_eq!(exported, native);
    assert_eq!(exported, EIP1271_SIGNATURE);
}

#[wasm_bindgen_test]
fn eip1271_payload_matches_recorded_typescript_sdk_vector() {
    let vector: UpstreamVector =
        serde_json::from_str(include_str!("fixtures/eip1271_upstream_vector.json")).unwrap();
    let exported = envelope_string(
        eip1271_signature_payload_export(wasm_order_input(), vector.ecdsa_signature).unwrap(),
    );

    assert_eq!(exported, vector.eip1271_signature);
}

#[wasm_bindgen_test]
async fn sign_order_with_eip1271_returns_eip1271_scheme() {
    let signer = callback("envelope", &format!("return '{}';", ECDSA_SIGNATURE));
    let signed = json(
        sign_order_with_eip1271(
            wasm_order_input(),
            CHAIN_MAINNET,
            ADDR_OWNER.to_owned(),
            signer,
            None,
        )
        .await
        .unwrap(),
    );

    assert_eq!(signed["value"]["signingScheme"], "eip1271");
    assert_eq!(signed["value"]["signature"], EIP1271_SIGNATURE);
}

#[wasm_bindgen_test]
async fn sign_order_with_eip1271_uid_equals_generated_order_id_as_str() {
    let signer = callback("envelope", &format!("return '{}';", ECDSA_SIGNATURE));
    let signed = json(
        sign_order_with_eip1271(
            wasm_order_input(),
            CHAIN_MAINNET,
            ADDR_OWNER.to_owned(),
            signer,
            None,
        )
        .await
        .unwrap(),
    );

    assert_eq!(signed["value"]["orderUid"], expected_uid());
}

#[wasm_bindgen_test]
async fn sign_order_with_eip1271_from_field_is_owner() {
    let signer = callback("envelope", &format!("return '{}';", ECDSA_SIGNATURE));
    let signed = json(
        sign_order_with_eip1271(
            wasm_order_input(),
            CHAIN_MAINNET,
            ADDR_OWNER.to_owned(),
            signer,
            None,
        )
        .await
        .unwrap(),
    );

    assert_eq!(signed["value"]["from"], ADDR_OWNER);
}

#[wasm_bindgen_test]
async fn typed_data_callback_receives_order_primary_type() {
    let signer = callback(
        "envelope",
        &format!(
            "globalThis.__cowEip1271Envelope = envelope; return '{}';",
            ECDSA_SIGNATURE
        ),
    );
    sign_order_with_eip1271(
        wasm_order_input(),
        CHAIN_MAINNET,
        ADDR_OWNER.to_owned(),
        signer,
        None,
    )
    .await
    .unwrap();
    let envelope = json(js_sys::eval("globalThis.__cowEip1271Envelope").unwrap());

    assert_eq!(envelope["primaryType"], "Order");
}

#[wasm_bindgen_test]
async fn custom_eip1271_callback_signature_is_used_verbatim() {
    let custom = callback(
        "request",
        "globalThis.__cowCustomEip1271 = request; return '0x1234';",
    );
    let signed = json(
        sign_order_with_custom_eip1271(
            wasm_order_input(),
            CHAIN_MAINNET,
            ADDR_OWNER.to_owned(),
            custom,
            None,
        )
        .await
        .unwrap(),
    );
    let request = json(js_sys::eval("globalThis.__cowCustomEip1271").unwrap());

    assert_eq!(request["owner"], ADDR_OWNER);
    assert_eq!(request["chainId"], CHAIN_MAINNET);
    assert_eq!(signed["value"]["signature"], "0x1234");
}

#[wasm_bindgen_test]
async fn missing_owner_is_rejected_before_dispatch() {
    let signer = callback(
        "envelope",
        "globalThis.__cowEip1271UnexpectedDispatch = true; return '0x00';",
    );
    let error = sign_order_with_eip1271(
        wasm_order_input(),
        CHAIN_MAINNET,
        "0x1234".to_owned(),
        signer,
        None,
    )
    .await
    .expect_err("malformed owner must fail");
    let dispatched = js_sys::eval("Boolean(globalThis.__cowEip1271UnexpectedDispatch)")
        .unwrap()
        .as_bool()
        .unwrap();
    let value = json(error);

    assert_eq!(value["kind"], "invalidInput");
    assert_eq!(value["field"], "owner");
    assert!(!dispatched);
}

#[wasm_bindgen_test]
async fn callback_throw_maps_to_typed_wallet_error() {
    let signer = callback("envelope", "throw new Error('cannot sign');");
    let error = sign_order_with_eip1271(
        wasm_order_input(),
        CHAIN_MAINNET,
        ADDR_OWNER.to_owned(),
        signer,
        None,
    )
    .await
    .expect_err("throw must fail");
    let value = json(error);

    assert_eq!(value["kind"], "walletRequest");
    assert_eq!(value["method"], "signTypedData");
}

#[wasm_bindgen_test]
async fn custom_callback_non_string_is_rejected() {
    let custom = callback("request", "return { signature: '0x1234' };");
    let error = sign_order_with_custom_eip1271(
        wasm_order_input(),
        CHAIN_MAINNET,
        ADDR_OWNER.to_owned(),
        custom,
        None,
    )
    .await
    .expect_err("non-string custom callback must fail");
    let value = json(error);

    assert_eq!(value["kind"], "walletRequest");
}

#[wasm_bindgen_test]
fn resolved_eip1271_provider_is_send_sync_without_jsvalue() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<ResolvedEip1271Provider>();
}

#[wasm_bindgen_test]
fn eip1271_rejects_malformed_ecdsa_signature() {
    let error = eip1271_signature_payload_export(wasm_order_input(), "0x1234".to_owned())
        .expect_err("malformed ECDSA signature must fail");
    let value = json(error);

    assert_eq!(value["kind"], "signing");
}
