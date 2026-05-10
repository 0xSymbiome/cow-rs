#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_wasm::exports::{IpfsClient, JsCallbackHttpTransport};
use js_sys::Function;
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use crate::common::{APP_DATA_CONTENT, CID_APP_DATA, ipfs_config};

wasm_bindgen_test_configure!(run_in_browser);

fn callback(args: &str, body: &str) -> Function {
    Function::new_with_args(args, body)
}

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

fn app_data_callback(body: &str) -> Function {
    callback(
        "request",
        &format!(
            "globalThis.__cowTransportRequest = request;
             return Promise.resolve({{ status: 200, statusText: 'OK', headers: {{ ok: 'yes' }}, body: '{}' }});",
            body.replace('\'', "\\'")
        ),
    )
}

#[wasm_bindgen_test]
async fn callback_transport_receives_request_dto_with_signal() {
    let fetch = app_data_callback(APP_DATA_CONTENT);
    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        Some(25),
        &fetch,
    ))
    .unwrap();

    let value = json(
        client
            .fetch_app_data_from_cid(CID_APP_DATA.to_owned())
            .await
            .unwrap(),
    );
    let request = json(js_sys::eval("globalThis.__cowTransportRequest").unwrap());
    let signal_is_abort_signal =
        js_sys::eval("globalThis.__cowTransportRequest.signal instanceof AbortSignal")
            .unwrap()
            .as_bool()
            .unwrap();

    assert_eq!(value["document"]["appCode"], "CoW Swap");
    assert_eq!(request["method"], "GET");
    assert_eq!(
        request["url"],
        format!("https://ipfs.example.test/ipfs/{CID_APP_DATA}")
    );
    assert_eq!(request["timeoutMs"], 25);
    assert!(signal_is_abort_signal);
}

#[wasm_bindgen_test]
async fn callback_transport_maps_non_2xx_to_typed_error() {
    let fetch = callback(
        "request",
        "return { status: 404, statusText: 'Not Found', headers: {}, body: 'missing' };",
    );
    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        None,
        &fetch,
    ))
    .unwrap();
    let error = client
        .fetch_app_data_from_cid(CID_APP_DATA.to_owned())
        .await
        .expect_err("non-success callback response must fail");
    let value = json(error);

    assert_eq!(value["kind"], "appData");
    assert_eq!(value["class"], "other");
}

#[wasm_bindgen_test]
async fn callback_transport_reject_maps_to_typed_error() {
    let fetch = callback(
        "request",
        "return Promise.reject(new Error('async unavailable'));",
    );
    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        None,
        &fetch,
    ))
    .unwrap();
    let error = client
        .fetch_app_data_from_cid(CID_APP_DATA.to_owned())
        .await
        .expect_err("callback rejection must fail");
    let value = json(error);

    assert_eq!(value["kind"], "appData");
}

#[wasm_bindgen_test]
async fn timeout_overflow_fails_before_dispatch() {
    let fetch = callback(
        "request",
        "globalThis.__cowOverflowDispatched = true; return { status: 200, headers: {}, body: '{}' };",
    );
    let error = match IpfsClient::new(ipfs_config(None, Some(i32::MAX as u32 + 1), &fetch)) {
        Ok(_) => panic!("oversized timeout must fail"),
        Err(error) => error,
    };
    let dispatched = js_sys::eval("Boolean(globalThis.__cowOverflowDispatched)")
        .unwrap()
        .as_bool()
        .unwrap();
    let value = json(error);

    assert_eq!(value["kind"], "invalidInput");
    assert!(!dispatched);
}

#[wasm_bindgen_test]
fn transport_storage_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<JsCallbackHttpTransport>();
}
