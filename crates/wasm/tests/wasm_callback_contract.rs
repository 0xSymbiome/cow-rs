#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_wasm::exports::{
    SigningOptions, compute_order_uid, sign_cancellation_eth_sign_digest,
    sign_cancellation_with_eip1193, sign_cancellation_with_typed_data_signer,
    sign_order_eth_sign_digest, sign_order_with_eip1193, sign_order_with_typed_data_signer,
};
use js_sys::{Function, Object, Reflect};
use serde_json::Value;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

use crate::common::{
    ADDR_OWNER, CHAIN_MAINNET, ECDSA_SIGNATURE, ECDSA_SIGNATURE_MODERN_V,
    ECDSA_SIGNATURE_MODERN_V_ONE, ECDSA_SIGNATURE_RECOVERY_28, wasm_order_input,
};

wasm_bindgen_test_configure!(run_in_browser);

fn callback(args: &str, body: &str) -> Function {
    Function::new_with_args(args, body)
}

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

fn set_js(target: &Object, key: &str, value: &JsValue) {
    Reflect::set(target, &JsValue::from_str(key), value).expect("test option should be set");
}

fn signing_options_with_wallet_timeout(timeout_ms: u32) -> SigningOptions {
    let wallet_config = Object::new();
    set_js(
        &wallet_config,
        "timeoutMs",
        &JsValue::from_f64(f64::from(timeout_ms)),
    );
    let options = Object::new();
    set_js(&options, "walletConfig", wallet_config.as_ref());
    JsValue::from(options).unchecked_into()
}

fn generated_order_uid() -> String {
    let value =
        json(compute_order_uid(wasm_order_input(), CHAIN_MAINNET, ADDR_OWNER.to_owned()).unwrap());
    value["value"]["orderUid"].as_str().unwrap().to_owned()
}

#[wasm_bindgen_test]
async fn typed_data_signer_receives_order_envelope() {
    let signer = callback(
        "envelope",
        &format!(
            "globalThis.__cowEnvelope = envelope; return '{}';",
            ECDSA_SIGNATURE
        ),
    );
    let signed = json(
        sign_order_with_typed_data_signer(
            wasm_order_input(),
            CHAIN_MAINNET,
            ADDR_OWNER.to_owned(),
            signer,
            None,
        )
        .await
        .unwrap(),
    );
    let envelope = json(js_sys::eval("globalThis.__cowEnvelope").unwrap());

    assert_eq!(envelope["primaryType"], "Order");
    assert_eq!(envelope["domain"]["chainId"], CHAIN_MAINNET);
    assert_eq!(signed["value"]["signingScheme"], "eip712");
}

#[wasm_bindgen_test]
async fn typed_data_signer_normalizes_modern_v_signatures() {
    let signer = callback(
        "envelope",
        &format!("return '{}';", ECDSA_SIGNATURE_MODERN_V),
    );
    let signed = json(
        sign_order_with_typed_data_signer(
            wasm_order_input(),
            CHAIN_MAINNET,
            ADDR_OWNER.to_owned(),
            signer,
            None,
        )
        .await
        .unwrap(),
    );

    assert_eq!(signed["value"]["signature"], ECDSA_SIGNATURE);

    let signer = callback(
        "envelope",
        &format!("return '{}';", ECDSA_SIGNATURE_MODERN_V_ONE),
    );
    let signed = json(
        sign_order_with_typed_data_signer(
            wasm_order_input(),
            CHAIN_MAINNET,
            ADDR_OWNER.to_owned(),
            signer,
            None,
        )
        .await
        .unwrap(),
    );

    assert_eq!(signed["value"]["signature"], ECDSA_SIGNATURE_RECOVERY_28);
}

#[wasm_bindgen_test]
async fn eip1193_request_uses_eth_sign_typed_data_v4() {
    let provider = callback(
        "request",
        &format!(
            "globalThis.__cowRequest = request; return Promise.resolve('{}');",
            ECDSA_SIGNATURE
        ),
    );
    let signed = json(
        sign_order_with_eip1193(
            wasm_order_input(),
            CHAIN_MAINNET,
            ADDR_OWNER.to_owned(),
            provider,
            None,
        )
        .await
        .unwrap(),
    );
    let request = json(js_sys::eval("globalThis.__cowRequest").unwrap());

    assert_eq!(request["method"], "eth_signTypedData_v4");
    assert_eq!(request["params"][0], ADDR_OWNER);
    assert_eq!(signed["value"]["signingScheme"], "eip712");
}

#[wasm_bindgen_test]
async fn eip1193_throw_maps_to_wallet_error() {
    let provider = callback(
        "request",
        "const err = new Error('provider denied request'); err.code = 4001; throw err;",
    );
    let error = sign_order_with_eip1193(
        wasm_order_input(),
        CHAIN_MAINNET,
        ADDR_OWNER.to_owned(),
        provider,
        None,
    )
    .await
    .expect_err("provider throw must fail");
    let value = json(error);

    assert_eq!(value["kind"], "walletRequest");
    assert_eq!(value["method"], "eth_signTypedData_v4");
    assert_eq!(value["code"], 4001);
}

#[wasm_bindgen_test]
async fn eip1193_rejection_maps_to_wallet_error() {
    let provider = callback(
        "request",
        "return Promise.reject(Object.assign(new Error('async denial'), { code: 4900 }));",
    );
    let error = sign_order_with_eip1193(
        wasm_order_input(),
        CHAIN_MAINNET,
        ADDR_OWNER.to_owned(),
        provider,
        None,
    )
    .await
    .expect_err("provider rejection must fail");
    let value = json(error);

    assert_eq!(value["kind"], "walletRequest");
    // The structured provider code survives as the safe machine signal, while
    // the provider-authored message is redacted to SDK-authored guidance
    // (ADR 0053), so the raw "async denial" reason never crosses the boundary.
    assert_eq!(value["code"], 4900);
    assert!(!value["message"].as_str().unwrap().contains("async denial"));
}

#[wasm_bindgen_test]
async fn typed_data_callback_non_string_return_is_rejected() {
    let signer = callback("envelope", "return { signature: 'not a string' };");
    let error = sign_order_with_typed_data_signer(
        wasm_order_input(),
        CHAIN_MAINNET,
        ADDR_OWNER.to_owned(),
        signer,
        None,
    )
    .await
    .expect_err("non-string callback return must fail");
    let value = json(error);

    assert_eq!(value["kind"], "walletRequest");
    assert!(
        value["message"]
            .as_str()
            .unwrap()
            .contains("callback did not return a string")
    );
}

#[wasm_bindgen_test]
async fn wallet_config_timeout_rejects_pending_signer_callback() {
    let signer = callback("envelope", "return new Promise(() => {});");
    let error = sign_order_with_typed_data_signer(
        wasm_order_input(),
        CHAIN_MAINNET,
        ADDR_OWNER.to_owned(),
        signer,
        Some(signing_options_with_wallet_timeout(1)),
    )
    .await
    .expect_err("wallet timeout must reject a pending signer callback");
    let value = json(error);

    assert_eq!(value["kind"], "walletTimeout");
    assert_eq!(value["timeoutMs"], 1);
}

#[wasm_bindgen_test]
async fn eth_sign_digest_timeout_preserves_wallet_timeout_kind() {
    let digest_signer = callback("digest", "return new Promise(() => {});");
    let error = sign_order_eth_sign_digest(
        wasm_order_input(),
        CHAIN_MAINNET,
        ADDR_OWNER.to_owned(),
        digest_signer,
        Some(signing_options_with_wallet_timeout(1)),
    )
    .await
    .expect_err("eth_sign digest wallet timeout must reject a pending callback");
    let value = json(error);

    // The digest path routes through the `DigestSigner` adapter; a wallet
    // timeout must keep the `walletTimeout` kind (and `timeoutMs`) rather than
    // collapsing into `walletRequest`, matching the typed-data path above.
    assert_eq!(value["kind"], "walletTimeout");
    assert_eq!(value["timeoutMs"], 1);
}

#[wasm_bindgen_test]
async fn eth_sign_cancellation_timeout_preserves_wallet_timeout_kind() {
    let digest_signer = callback("digest", "return new Promise(() => {});");
    let error = sign_cancellation_eth_sign_digest(
        vec![generated_order_uid()],
        CHAIN_MAINNET,
        digest_signer,
        Some(signing_options_with_wallet_timeout(1)),
    )
    .await
    .expect_err("eth_sign cancellation wallet timeout must reject a pending callback");
    let value = json(error);

    assert_eq!(value["kind"], "walletTimeout");
    assert_eq!(value["timeoutMs"], 1);
}

#[wasm_bindgen_test]
async fn eth_sign_digest_callback_receives_digest() {
    let signer = callback(
        "digest",
        &format!(
            "globalThis.__cowDigest = digest; return Promise.resolve('{}');",
            ECDSA_SIGNATURE
        ),
    );
    let signed = json(
        sign_order_eth_sign_digest(
            wasm_order_input(),
            CHAIN_MAINNET,
            ADDR_OWNER.to_owned(),
            signer,
            None,
        )
        .await
        .unwrap(),
    );
    let digest = js_sys::eval("globalThis.__cowDigest").unwrap();

    assert_eq!(digest.as_string().unwrap().len(), 66);
    assert_eq!(signed["value"]["signingScheme"], "ethsign");
}

#[wasm_bindgen_test]
async fn typed_cancellation_signer_returns_order_uids() {
    let order_uid = generated_order_uid();
    let signer = callback(
        "envelope",
        &format!(
            "globalThis.__cowCancel = envelope; return '{}';",
            ECDSA_SIGNATURE
        ),
    );
    let signed = json(
        sign_cancellation_with_typed_data_signer(
            vec![order_uid.clone()],
            CHAIN_MAINNET,
            signer,
            None,
        )
        .await
        .unwrap(),
    );
    let envelope = json(js_sys::eval("globalThis.__cowCancel").unwrap());

    assert_eq!(envelope["primaryType"], "OrderCancellations");
    assert_eq!(signed["value"]["orderUids"][0], order_uid);
    assert_eq!(signed["value"]["signingScheme"], "eip712");
}

#[wasm_bindgen_test]
async fn eip1193_cancellation_callback_shape_is_stable() {
    let order_uid = generated_order_uid();
    let provider = callback(
        "request",
        &format!(
            "globalThis.__cowCancelRequest = request; return '{}';",
            ECDSA_SIGNATURE
        ),
    );
    let signed = json(
        sign_cancellation_with_eip1193(
            vec![order_uid],
            CHAIN_MAINNET,
            ADDR_OWNER.to_owned(),
            provider,
            None,
        )
        .await
        .unwrap(),
    );
    let request = json(js_sys::eval("globalThis.__cowCancelRequest").unwrap());

    assert_eq!(request["method"], "eth_signTypedData_v4");
    assert_eq!(request["params"][0], ADDR_OWNER);
    assert_eq!(signed["value"]["signingScheme"], "eip712");
}

#[wasm_bindgen_test]
async fn eth_sign_cancellation_callback_receives_digest() {
    let order_uid = generated_order_uid();
    let signer = callback(
        "digest",
        &format!(
            "globalThis.__cowCancelDigest = digest; return '{}';",
            ECDSA_SIGNATURE
        ),
    );
    let signed = json(
        sign_cancellation_eth_sign_digest(vec![order_uid], CHAIN_MAINNET, signer, None)
            .await
            .unwrap(),
    );
    let digest = js_sys::eval("globalThis.__cowCancelDigest").unwrap();

    assert_eq!(digest.as_string().unwrap().len(), 66);
    assert_eq!(signed["value"]["signingScheme"], "ethsign");
}

#[wasm_bindgen_test]
async fn empty_cancellation_list_fails_before_callback_dispatch() {
    let signer = callback(
        "envelope",
        "globalThis.__cowUnexpectedCancelDispatch = true; return '0x00';",
    );
    let error = sign_cancellation_with_typed_data_signer(Vec::new(), CHAIN_MAINNET, signer, None)
        .await
        .expect_err("empty cancellation list must fail");
    let value = json(error);
    let dispatched = js_sys::eval("Boolean(globalThis.__cowUnexpectedCancelDispatch)")
        .unwrap()
        .as_bool()
        .unwrap();

    assert_eq!(value["kind"], "invalidInput");
    assert_eq!(value["field"], "orderUids");
    assert!(!dispatched);
}
