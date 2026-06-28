#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_core::OrderData;
use cow_sdk_js::exports::{
    compute_order_uid, sign_cancellation_with_typed_data_signer, sign_order_with_typed_data_signer,
};
use js_sys::{Function, Reflect};
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use crate::common::{ADDR_OWNER, CHAIN_MAINNET, CHAIN_UNSUPPORTED, wasm_order_input};

wasm_bindgen_test_configure!(run_in_browser);

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

fn callback(args: &str, body: &str) -> Function {
    Function::new_with_args(args, body)
}

#[wasm_bindgen_test]
fn unknown_order_kind_fails_closed_during_input_decode() {
    let input = serde_json::json!({
        "sellToken": crate::common::ADDR_SELL,
        "buyToken": crate::common::ADDR_BUY,
        "receiver": crate::common::ADDR_ZERO,
        "sellAmount": "1",
        "buyAmount": "2",
        "validTo": crate::common::VALID_TO,
        "appData": crate::common::HASH_APP_DATA,
        "feeAmount": "0",
        "kind": "swap",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20"
    });
    let value = serde_wasm_bindgen::to_value(&input).unwrap();
    let decoded = serde_wasm_bindgen::from_value::<OrderData>(value);

    assert!(decoded.is_err());
}

#[wasm_bindgen_test]
fn unknown_buy_balance_fails_closed_during_input_decode() {
    let mut input = wasm_order_input();
    input.buy_token_balance = cow_sdk_core::BuyTokenDestination::Internal;
    assert!(compute_order_uid(input, CHAIN_MAINNET, ADDR_OWNER.to_owned()).is_ok());

    let invalid = serde_json::json!({
        "sellToken": crate::common::ADDR_SELL,
        "buyToken": crate::common::ADDR_BUY,
        "receiver": crate::common::ADDR_ZERO,
        "sellAmount": "1",
        "buyAmount": "2",
        "validTo": crate::common::VALID_TO,
        "appData": crate::common::HASH_APP_DATA,
        "feeAmount": "0",
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "external"
    });
    let decoded = serde_wasm_bindgen::from_value::<OrderData>(
        serde_wasm_bindgen::to_value(&invalid).unwrap(),
    );

    assert!(decoded.is_err());
}

#[wasm_bindgen_test]
fn missing_required_order_field_fails_closed() {
    let input = serde_json::json!({
        "buyToken": crate::common::ADDR_BUY,
        "sellAmount": "1",
        "buyAmount": "2",
        "validTo": crate::common::VALID_TO,
        "appData": crate::common::HASH_APP_DATA,
        "feeAmount": "0",
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20"
    });
    let decoded =
        serde_wasm_bindgen::from_value::<OrderData>(serde_wasm_bindgen::to_value(&input).unwrap());

    assert!(decoded.is_err());
}

#[wasm_bindgen_test]
fn malformed_app_data_hash_fails_closed_during_input_decode() {
    let input = serde_json::json!({
        "sellToken": crate::common::ADDR_SELL,
        "buyToken": crate::common::ADDR_BUY,
        "receiver": crate::common::ADDR_ZERO,
        "sellAmount": "1",
        "buyAmount": "2",
        "validTo": crate::common::VALID_TO,
        "appData": "0x1234",
        "feeAmount": "0",
        "kind": "sell",
        "partiallyFillable": false,
        "sellTokenBalance": "erc20",
        "buyTokenBalance": "erc20"
    });
    let decoded =
        serde_wasm_bindgen::from_value::<OrderData>(serde_wasm_bindgen::to_value(&input).unwrap());

    assert!(decoded.is_err());
}

#[wasm_bindgen_test]
async fn unsupported_chain_rejects_before_wallet_callback() {
    let signer = callback(
        "envelope",
        "globalThis.__cowUnsupportedChainDispatch = true; return '0x00';",
    );
    let error = sign_order_with_typed_data_signer(
        wasm_order_input(),
        CHAIN_UNSUPPORTED,
        ADDR_OWNER.to_owned(),
        signer,
        None,
    )
    .await
    .expect_err("unsupported chain must fail");
    let dispatched = js_sys::eval("Boolean(globalThis.__cowUnsupportedChainDispatch)")
        .unwrap()
        .as_bool()
        .unwrap();
    let value = json(error);

    assert_eq!(value["kind"], "unsupportedChain");
    assert!(!dispatched);
}

#[wasm_bindgen_test]
async fn empty_cancellation_rejects_before_wallet_callback() {
    let signer = callback(
        "envelope",
        "globalThis.__cowEmptyCancelDispatch = true; return '0x00';",
    );
    let error = sign_cancellation_with_typed_data_signer(Vec::new(), CHAIN_MAINNET, signer, None)
        .await
        .expect_err("empty cancellation list must fail");
    let dispatched = js_sys::eval("Boolean(globalThis.__cowEmptyCancelDispatch)")
        .unwrap()
        .as_bool()
        .unwrap();
    let value = json(error);

    assert_eq!(value["kind"], "invalidInput");
    assert!(!dispatched);
}

#[wasm_bindgen_test]
fn flavour_descriptor_exposes_web_and_module_subpaths() {
    let descriptor: Value = serde_json::from_str(include_str!("../npm/flavours.json")).unwrap();
    let flavours = descriptor["flavours"].as_array().unwrap();

    // Every flavour ships the same target coverage so a browser, Node, edge, or
    // source-phase host resolves a working build for any feature set: the bundler
    // build backs browser dApps, nodejs backs Node hosts, the web build backs edge
    // runtimes (Cloudflare Workers, Deno, Vercel Edge) through the explicit web
    // subpath, and the source-phase module build is the opt-in standards-track
    // entry. Consistency is the contract — no flavour is browser-portable while
    // another is bundler-only.
    for flavour in flavours {
        let name = flavour["name"].as_str().unwrap();
        let targets = flavour["targets"].as_array().unwrap();
        for required in ["bundler", "nodejs", "web"] {
            assert!(
                targets
                    .iter()
                    .any(|target| target.as_str() == Some(required)),
                "{name} must ship the {required} target"
            );
        }
        assert!(
            flavour["webSubpath"].is_string(),
            "{name} must declare a webSubpath for edge runtimes"
        );
        assert!(
            flavour["rawWasmSubpath"].is_string(),
            "{name} must declare a rawWasmSubpath for the Worker module asset"
        );
        assert!(
            flavour["moduleSubpath"].is_string(),
            "{name} must declare a moduleSubpath for the source-phase build"
        );
    }

    // The order-lifecycle flavour anchors the concrete subpath shape.
    let trading = flavours
        .iter()
        .find(|flavour| flavour["name"] == "trading")
        .unwrap();
    assert_eq!(trading["webSubpath"], "./trading/edge");
    assert_eq!(trading["rawWasmSubpath"], "./trading/edge/wasm");
    assert_eq!(trading["moduleSubpath"], "./trading/module");
}

#[wasm_bindgen_test]
fn worker_source_avoids_dynamic_wasm_compilation_entrypoints() {
    let source = include_str!("../../../e2e/wasm-typescript-cf/src/worker.ts");
    let patterns = [
        "WebAssembly.compile",
        "WebAssembly.compileStreaming",
        "WebAssembly.instantiateStreaming",
    ];

    for pattern in patterns {
        assert!(!source.contains(pattern), "{pattern} must not appear");
    }
    assert!(
        !source.contains("WebAssembly.instantiate("),
        "runtime instantiation must not be hand-coded"
    );
}

#[wasm_bindgen_test]
fn abort_controller_is_available_for_callback_transport() {
    let controller = js_sys::eval("new AbortController()").unwrap();
    let signal = Reflect::get(&controller, &JsValue::from_str("signal")).unwrap();
    let is_abort_signal = js_sys::eval("(new AbortController()).signal instanceof AbortSignal")
        .unwrap()
        .as_bool()
        .unwrap();

    assert!(!signal.is_undefined());
    assert!(is_abort_signal);
}
