#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_wasm::exports::IpfsClient;
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

#[wasm_bindgen_test]
async fn client_owned_callback_survives_until_request_resolves() {
    let fetch = callback(
        "request",
        &format!(
            "globalThis.__cowCallbackLifetime = {{ seenHandle: 'id' in request, calls: (globalThis.__cowCallbackLifetime?.calls || 0) + 1 }};
             return new Promise((resolve) => setTimeout(() => resolve({{
               status: 200,
               headers: {{}},
               body: '{}'
             }}), 0));",
            APP_DATA_CONTENT.replace('\'', "\\'")
        ),
    );
    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        None,
        &fetch,
    ))
    .unwrap();

    let value = json(
        client
            .fetch_app_data_from_cid(CID_APP_DATA.to_owned())
            .await
            .unwrap(),
    );
    let lifetime = json(js_sys::eval("globalThis.__cowCallbackLifetime").unwrap());

    assert_eq!(value["document"]["appCode"], "CoW Swap");
    assert_eq!(lifetime["calls"], 1);
    assert_eq!(lifetime["seenHandle"], false);
}
