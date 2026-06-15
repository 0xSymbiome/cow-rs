#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_wasm::exports::{IpfsClient, SdkClientOptions};
use js_sys::{Function, Object, Reflect};
use serde_json::Value;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

use crate::common::{APP_DATA_CONTENT, CID_APP_DATA, ipfs_config};

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

fn tracked_signal(pre_abort: bool) -> JsValue {
    callback(
        "preAbort",
        r#"
        const controller = new AbortController();
        const signal = controller.signal;
        const add = signal.addEventListener.bind(signal);
        const remove = signal.removeEventListener.bind(signal);
        const stats = { adds: 0, removes: 0 };
        signal.addEventListener = (type, listener, options) => {
          if (type === "abort") {
            stats.adds += 1;
          }
          return add(type, listener, options);
        };
        signal.removeEventListener = (type, listener, options) => {
          if (type === "abort") {
            stats.removes += 1;
          }
          return remove(type, listener, options);
        };
        globalThis.__cowAbortController = controller;
        globalThis.__cowAbortSignal = signal;
        globalThis.__cowAbortStats = stats;
        if (preAbort) {
          controller.abort();
        }
        return signal;
        "#,
    )
    .call1(&JsValue::NULL, &JsValue::from_bool(pre_abort))
    .unwrap()
}

fn options_with_signal(signal: &JsValue, timeout_ms: Option<f64>) -> SdkClientOptions {
    let options = Object::new();
    set_js(&options, "signal", signal);
    if let Some(timeout_ms) = timeout_ms {
        set_js(&options, "timeoutMs", &JsValue::from_f64(timeout_ms));
    }
    JsValue::from(options).unchecked_into()
}

fn listener_counts() -> (u32, u32) {
    let stats = js_sys::eval("globalThis.__cowAbortStats").unwrap();
    let adds = Reflect::get(&stats, &JsValue::from_str("adds"))
        .unwrap()
        .as_f64()
        .unwrap() as u32;
    let removes = Reflect::get(&stats, &JsValue::from_str("removes"))
        .unwrap()
        .as_f64()
        .unwrap() as u32;
    (adds, removes)
}

fn assert_listener_counts(adds: u32, removes: u32) {
    assert_eq!(listener_counts(), (adds, removes));
}

fn success_callback() -> Function {
    callback(
        "request",
        &format!(
            "return {{ status: 200, headers: {{}}, body: '{}' }};",
            APP_DATA_CONTENT.replace('\'', "\\'")
        ),
    )
}

fn ipfs_client(fetch: &Function) -> IpfsClient {
    IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        None,
        fetch,
    ))
    .unwrap()
}

#[wasm_bindgen_test]
async fn abort_bridge_removes_listener_after_success() {
    let signal = tracked_signal(false);
    let client = ipfs_client(&success_callback());
    client
        .fetch_app_data_from_cid(
            CID_APP_DATA.to_owned(),
            Some(options_with_signal(&signal, None)),
        )
        .await
        .unwrap();

    assert_listener_counts(1, 1);
}

#[wasm_bindgen_test]
async fn abort_bridge_removes_listener_after_callback_throw() {
    let signal = tracked_signal(false);
    let client = ipfs_client(&callback("request", "throw new Error('sync failure');"));
    let error = client
        .fetch_app_data_from_cid(
            CID_APP_DATA.to_owned(),
            Some(options_with_signal(&signal, None)),
        )
        .await
        .expect_err("throwing callback must fail");
    let value = json(error);

    assert_eq!(value["kind"], "appData");
    assert_listener_counts(1, 1);
}

#[wasm_bindgen_test]
async fn abort_bridge_removes_listener_after_callback_reject() {
    let signal = tracked_signal(false);
    let client = ipfs_client(&callback(
        "request",
        "return Promise.reject(new Error('async failure'));",
    ));
    let error = client
        .fetch_app_data_from_cid(
            CID_APP_DATA.to_owned(),
            Some(options_with_signal(&signal, None)),
        )
        .await
        .expect_err("rejecting callback must fail");
    let value = json(error);

    assert_eq!(value["kind"], "appData");
    assert_listener_counts(1, 1);
}

#[wasm_bindgen_test]
async fn abort_bridge_removes_listener_after_parse_error() {
    let signal = tracked_signal(false);
    let client = ipfs_client(&callback(
        "request",
        "globalThis.__cowUnexpectedCancellationFetch = true; return { status: 200, headers: {}, body: '{}' };",
    ));
    let error = client
        .fetch_app_data_from_hex(
            "not-a-hex".to_owned(),
            Some(options_with_signal(&signal, None)),
        )
        .await
        .expect_err("parse error must fail");
    let dispatched = js_sys::eval("Boolean(globalThis.__cowUnexpectedCancellationFetch)")
        .unwrap()
        .as_bool()
        .unwrap();
    let value = json(error);

    assert_eq!(value["kind"], "appData");
    assert!(!dispatched);
    assert_listener_counts(1, 1);
}

#[wasm_bindgen_test]
async fn abort_bridge_removes_listener_after_timeout_overflow() {
    let signal = tracked_signal(false);
    let client = ipfs_client(&success_callback());
    let error = client
        .fetch_app_data_from_cid(
            CID_APP_DATA.to_owned(),
            Some(options_with_signal(
                &signal,
                Some(f64::from(i32::MAX) + 1.0),
            )),
        )
        .await
        .expect_err("oversized timeout must fail");
    let value = json(error);

    assert_eq!(value["kind"], "invalidInput");
    assert_eq!(value["field"], "timeoutMs");
    assert_listener_counts(1, 1);
}

#[wasm_bindgen_test]
async fn pre_aborted_signal_cancels_without_registering_listener() {
    let signal = tracked_signal(true);
    let client = ipfs_client(&success_callback());
    let error = client
        .fetch_app_data_from_cid(
            CID_APP_DATA.to_owned(),
            Some(options_with_signal(&signal, None)),
        )
        .await
        .expect_err("pre-aborted signal must cancel the call");
    let value = json(error);

    assert_eq!(value["kind"], "cancelled");
    assert_listener_counts(0, 0);
}

#[wasm_bindgen_test]
async fn abort_signal_cancels_pending_client_call_and_cleans_up_listener() {
    let signal = tracked_signal(false);
    let client = ipfs_client(&callback(
        "request",
        "setTimeout(() => globalThis.__cowAbortController.abort(), 0); return new Promise(() => {});",
    ));
    let error = client
        .fetch_app_data_from_cid(
            CID_APP_DATA.to_owned(),
            Some(options_with_signal(&signal, None)),
        )
        .await
        .expect_err("aborted signal must cancel the pending call");
    let value = json(error);

    assert_eq!(value["kind"], "cancelled");
    assert_listener_counts(1, 1);
}
