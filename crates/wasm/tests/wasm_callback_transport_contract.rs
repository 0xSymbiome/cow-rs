#![cfg(target_arch = "wasm32")]

use std::time::Duration;

use cow_sdk_core::{HttpTransport, TransportError, TransportErrorClass};
use cow_sdk_wasm::exports::{
    JsCallbackHttpTransport,
    registry::{FetchCallbackHandleId, register_fetch_callback},
};
use js_sys::Function;
use serde_json::Value;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

fn callback(args: &str, body: &str) -> Function {
    Function::new_with_args(args, body)
}

fn value_from_global(name: &str) -> Value {
    serde_wasm_bindgen::from_value(
        js_sys::eval(&format!("globalThis.{name}")).expect("global value should exist"),
    )
    .expect("global value should decode to JSON")
}

fn transport(
    function: Function,
) -> (
    cow_sdk_wasm::exports::registry::FetchCallbackHandle,
    JsCallbackHttpTransport,
) {
    let handle = register_fetch_callback(function).expect("callback should register");
    let id = FetchCallbackHandleId::new(handle.id()).expect("handle id should be valid");
    let transport = JsCallbackHttpTransport::new(
        "https://api.example.test".to_owned(),
        id,
        Some(Duration::from_millis(500)),
    )
    .expect("transport should build");
    (handle, transport)
}

#[wasm_bindgen_test]
fn registered_callbacks_receive_unique_nonzero_handles() {
    let first = register_fetch_callback(callback(
        "request",
        "return { status: 200, headers: {}, body: 'a' };",
    ))
    .unwrap();
    let second = register_fetch_callback(callback(
        "request",
        "return { status: 200, headers: {}, body: 'b' };",
    ))
    .unwrap();

    assert_ne!(first.id(), 0);
    assert_ne!(second.id(), 0);
    assert_ne!(first.id(), second.id());
}

#[wasm_bindgen_test]
fn handle_id_is_within_js_safe_integer_range() {
    let handle = register_fetch_callback(callback(
        "request",
        "return { status: 200, headers: {}, body: 'ok' };",
    ))
    .unwrap();

    assert!(u64::from(handle.id()) < 9_007_199_254_740_991);
}

#[wasm_bindgen_test]
fn zero_handle_id_is_reserved() {
    let error = FetchCallbackHandleId::new(0).expect_err("zero handle id must fail");
    let value: Value = serde_wasm_bindgen::from_value(error).unwrap();

    assert_eq!(value["kind"], "invalidInput");
    assert_eq!(value["field"], "fetchCallbackId");
}

#[wasm_bindgen_test]
fn dispose_is_idempotent() {
    let handle = register_fetch_callback(callback(
        "request",
        "return { status: 200, headers: {}, body: 'ok' };",
    ))
    .unwrap();

    handle.dispose();
    handle.dispose();
}

#[wasm_bindgen_test]
async fn disposed_handle_returns_configuration_error() {
    let handle = register_fetch_callback(callback(
        "request",
        "return { status: 200, headers: {}, body: 'ok' };",
    ))
    .unwrap();
    let id = FetchCallbackHandleId::new(handle.id()).unwrap();
    let transport =
        JsCallbackHttpTransport::new("https://api.example.test".to_owned(), id, None).unwrap();
    handle.dispose();

    let error = transport.get("/orders", &[], None).await.unwrap_err();

    assert!(matches!(error, TransportError::Configuration { .. }));
}

#[wasm_bindgen_test]
async fn callback_receives_request_dto_with_signal() {
    let (_handle, transport) = transport(callback(
        "request",
        "globalThis.__cowTransportRequest = request; return { status: 200, headers: { ok: 'yes' }, body: 'posted' };",
    ));
    let body = transport
        .post(
            "/orders",
            "{\"hello\":true}",
            &[("x-test".to_owned(), "yes".to_owned())],
            Some(Duration::from_millis(25)),
        )
        .await
        .unwrap();
    let request = value_from_global("__cowTransportRequest");
    let signal_is_abort_signal =
        js_sys::eval("globalThis.__cowTransportRequest.signal instanceof AbortSignal")
            .unwrap()
            .as_bool()
            .unwrap();

    assert_eq!(body, "posted");
    assert_eq!(request["method"], "POST");
    assert_eq!(request["url"], "https://api.example.test/orders");
    assert_eq!(request["headers"]["x-test"], "yes");
    assert_eq!(request["body"], "{\"hello\":true}");
    assert_eq!(request["timeoutMs"], 25);
    assert!(signal_is_abort_signal);
}

#[wasm_bindgen_test]
async fn callback_response_2xx_passes_body_through() {
    let (_handle, transport) = transport(callback(
        "request",
        "return Promise.resolve({ status: 204, headers: {}, body: 'accepted' });",
    ));

    assert_eq!(
        transport.get("/health", &[], None).await.unwrap(),
        "accepted"
    );
}

#[wasm_bindgen_test]
async fn callback_response_non_2xx_maps_to_http_status() {
    let (_handle, transport) = transport(callback(
        "request",
        "return { status: 404, headers: { 'x-trace': 'secret' }, body: 'missing' };",
    ));
    let error = transport.get("/missing", &[], None).await.unwrap_err();

    assert!(matches!(
        error,
        TransportError::HttpStatus { status: 404, .. }
    ));
}

#[wasm_bindgen_test]
async fn callback_throw_maps_to_connect_error() {
    let (_handle, transport) = transport(callback(
        "request",
        "throw new Error('transport unavailable');",
    ));
    let error = transport.get("/orders", &[], None).await.unwrap_err();

    assert!(matches!(
        error,
        TransportError::Transport {
            class: TransportErrorClass::Connect,
            ..
        }
    ));
}

#[wasm_bindgen_test]
async fn callback_reject_maps_to_connect_error() {
    let (_handle, transport) = transport(callback(
        "request",
        "return Promise.reject(new Error('async unavailable'));",
    ));
    let error = transport.get("/orders", &[], None).await.unwrap_err();

    assert!(matches!(
        error,
        TransportError::Transport {
            class: TransportErrorClass::Connect,
            ..
        }
    ));
}

#[wasm_bindgen_test]
async fn callback_abort_error_maps_to_timeout() {
    let (_handle, transport) = transport(callback(
        "request",
        "return Promise.reject(Object.assign(new Error('timed out'), { name: 'AbortError' }));",
    ));
    let error = transport.get("/orders", &[], None).await.unwrap_err();

    assert!(matches!(
        error,
        TransportError::Transport {
            class: TransportErrorClass::Timeout,
            ..
        }
    ));
}

#[wasm_bindgen_test]
async fn malformed_response_maps_to_decode_error() {
    let (_handle, transport) = transport(callback("request", "return { status: 200 };"));
    let error = transport.get("/orders", &[], None).await.unwrap_err();

    assert!(matches!(
        error,
        TransportError::Transport {
            class: TransportErrorClass::Decode,
            ..
        }
    ));
}

#[wasm_bindgen_test]
fn transport_storage_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<JsCallbackHttpTransport>();
}

#[wasm_bindgen_test]
async fn all_http_methods_round_trip_through_callback() {
    let (_handle, transport) = transport(callback(
        "request",
        "globalThis.__cowMethods = (globalThis.__cowMethods || []).concat(request.method); return { status: 200, headers: {}, body: request.method };",
    ));

    assert_eq!(transport.get("/a", &[], None).await.unwrap(), "GET");
    assert_eq!(transport.post("/b", "b", &[], None).await.unwrap(), "POST");
    assert_eq!(transport.put("/c", "c", &[], None).await.unwrap(), "PUT");
    assert_eq!(
        transport.delete("/d", "d", &[], None).await.unwrap(),
        "DELETE"
    );
    assert_eq!(
        value_from_global("__cowMethods"),
        serde_json::json!(["GET", "POST", "PUT", "DELETE"])
    );
}

#[wasm_bindgen_test]
async fn absolute_url_is_not_prefixed_by_base_url() {
    let (_handle, transport) = transport(callback(
        "request",
        "globalThis.__cowAbsoluteUrl = request.url; return { status: 200, headers: {}, body: 'ok' };",
    ));

    transport
        .get("https://orders.example.test/v1", &[], None)
        .await
        .unwrap();
    let url = js_sys::eval("globalThis.__cowAbsoluteUrl").unwrap();

    assert_eq!(url.as_string().unwrap(), "https://orders.example.test/v1");
}

#[wasm_bindgen_test]
async fn timeout_overflow_fails_before_dispatch() {
    let (_handle, transport) = transport(callback(
        "request",
        "globalThis.__cowOverflowDispatched = true; return { status: 200, headers: {}, body: 'ok' };",
    ));
    let error = transport
        .get(
            "/orders",
            &[],
            Some(Duration::from_millis(i32::MAX as u64 + 1)),
        )
        .await
        .unwrap_err();
    let dispatched = js_sys::eval("Boolean(globalThis.__cowOverflowDispatched)")
        .unwrap()
        .as_bool()
        .unwrap();

    assert!(matches!(error, TransportError::Configuration { .. }));
    assert!(!dispatched);
}

#[wasm_bindgen_test]
async fn numeric_timer_handle_is_cleared() {
    js_sys::eval(
        "globalThis.__setTimeoutOriginal = globalThis.setTimeout;
         globalThis.__clearTimeoutOriginal = globalThis.clearTimeout;
         globalThis.__cowClearedTimer = undefined;
         globalThis.setTimeout = function () { return 77; };
         globalThis.clearTimeout = function (handle) { globalThis.__cowClearedTimer = handle; };",
    )
    .unwrap();
    let (_handle, transport) = transport(callback(
        "request",
        "return { status: 200, headers: {}, body: 'ok' };",
    ));

    transport
        .get("/orders", &[], Some(Duration::from_millis(20)))
        .await
        .unwrap();
    let cleared = js_sys::eval(
        "globalThis.setTimeout = globalThis.__setTimeoutOriginal;
         globalThis.clearTimeout = globalThis.__clearTimeoutOriginal;
         globalThis.__cowClearedTimer",
    )
    .unwrap();

    assert_eq!(cleared.as_f64().unwrap(), 77.0);
}

#[wasm_bindgen_test]
async fn object_timer_handle_is_cleared() {
    js_sys::eval(
        "globalThis.__setTimeoutOriginalObject = globalThis.setTimeout;
         globalThis.__clearTimeoutOriginalObject = globalThis.clearTimeout;
         globalThis.__cowObjectTimer = { id: 'object-handle' };
         globalThis.__cowClearedObjectTimer = undefined;
         globalThis.setTimeout = function () { return globalThis.__cowObjectTimer; };
         globalThis.clearTimeout = function (handle) { globalThis.__cowClearedObjectTimer = handle; };",
    )
    .unwrap();
    let (_handle, transport) = transport(callback(
        "request",
        "return { status: 200, headers: {}, body: 'ok' };",
    ));

    transport
        .get("/orders", &[], Some(Duration::from_millis(20)))
        .await
        .unwrap();
    let same_object = js_sys::eval(
        "globalThis.setTimeout = globalThis.__setTimeoutOriginalObject;
         globalThis.clearTimeout = globalThis.__clearTimeoutOriginalObject;
         globalThis.__cowClearedObjectTimer === globalThis.__cowObjectTimer",
    )
    .unwrap();

    assert_eq!(same_object.as_bool(), Some(true));
}
