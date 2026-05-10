#![cfg(target_arch = "wasm32")]

use cow_sdk_wasm::exports::JsCallbackHttpTransport;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn js_callback_http_transport_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<JsCallbackHttpTransport>();
}
