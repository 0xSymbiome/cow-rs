#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_wasm::exports::{
    HttpToIpfsAdapter, IpfsClient, IpfsClientWithFetch, registry::register_fetch_callback,
};
use js_sys::Function;
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use crate::common::{APP_DATA_CONTENT, CID_APP_DATA, HASH_APP_DATA};

wasm_bindgen_test_configure!(run_in_browser);

fn callback(args: &str, body: &str) -> Function {
    Function::new_with_args(args, body)
}

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

fn app_data_fetch_callback() -> Function {
    callback(
        "request",
        &format!(
            "globalThis.__cowIpfsRequests = (globalThis.__cowIpfsRequests || []).concat(request.url);
             return {{ status: 200, headers: {{}}, body: '{}' }};",
            APP_DATA_CONTENT.replace('\'', "\\'")
        ),
    )
}

#[wasm_bindgen_test]
async fn adapter_fetches_app_data_from_cid() {
    let adapter = HttpToIpfsAdapter::new(app_data_fetch_callback(), Some(500)).unwrap();
    let value = json(
        adapter
            .fetch_app_data_from_cid(
                CID_APP_DATA.to_owned(),
                Some("https://ipfs.example.test/ipfs".to_owned()),
            )
            .await
            .unwrap(),
    );

    assert_eq!(value["schemaVersion"], "v1");
    assert_eq!(value["document"]["appCode"], "CoW Swap");
}

#[wasm_bindgen_test]
async fn adapter_fetches_app_data_from_hex() {
    let adapter = HttpToIpfsAdapter::new(app_data_fetch_callback(), None).unwrap();
    let value = json(
        adapter
            .fetch_app_data_from_hex(
                HASH_APP_DATA.to_owned(),
                Some("https://ipfs.example.test/ipfs".to_owned()),
            )
            .await
            .unwrap(),
    );

    assert_eq!(value["document"]["version"], "0.7.0");
}

#[wasm_bindgen_test]
async fn ipfs_client_with_fetch_uses_custom_gateway_url() {
    let client = IpfsClientWithFetch::new(
        Some("https://ipfs.example.test/ipfs".to_owned()),
        Some(500),
        app_data_fetch_callback(),
    )
    .unwrap();
    client
        .fetch_app_data_from_cid(CID_APP_DATA.to_owned())
        .await
        .unwrap();
    let requests: Vec<String> =
        serde_wasm_bindgen::from_value(js_sys::eval("globalThis.__cowIpfsRequests").unwrap())
            .unwrap();

    assert!(
        requests
            .iter()
            .any(|url| url == &format!("https://ipfs.example.test/ipfs/{CID_APP_DATA}"))
    );
}

#[wasm_bindgen_test]
async fn ipfs_client_from_handle_keeps_shared_callback_alive() {
    let handle = register_fetch_callback(app_data_fetch_callback()).unwrap();
    let client = IpfsClientWithFetch::from_handle(
        Some("https://ipfs.example.test/ipfs".to_owned()),
        None,
        handle.id(),
    )
    .unwrap();
    let value = json(
        client
            .fetch_app_data_from_cid(CID_APP_DATA.to_owned())
            .await
            .unwrap(),
    );

    assert_eq!(value["document"]["appCode"], "CoW Swap");
}

#[wasm_bindgen_test]
async fn http_404_maps_to_app_data_error() {
    let adapter = HttpToIpfsAdapter::new(
        callback(
            "request",
            "return { status: 404, headers: {}, body: 'not found' };",
        ),
        None,
    )
    .unwrap();
    let error = adapter
        .fetch_app_data_from_cid(
            CID_APP_DATA.to_owned(),
            Some("https://ipfs.example.test/ipfs".to_owned()),
        )
        .await
        .expect_err("404 must fail");
    let value = json(error);

    assert_eq!(value["kind"], "appData");
    assert_eq!(value["class"], "other");
}

#[wasm_bindgen_test]
async fn invalid_hex_rejects_before_fetch_callback() {
    let adapter = HttpToIpfsAdapter::new(
        callback(
            "request",
            "globalThis.__cowUnexpectedIpfsDispatch = true; return { status: 200, headers: {}, body: '{}' };",
        ),
        None,
    )
    .unwrap();
    let error = adapter
        .fetch_app_data_from_hex(
            "not-a-hex".to_owned(),
            Some("https://ipfs.example.test/ipfs".to_owned()),
        )
        .await
        .expect_err("malformed app-data hash must fail");
    let dispatched = js_sys::eval("Boolean(globalThis.__cowUnexpectedIpfsDispatch)")
        .unwrap()
        .as_bool()
        .unwrap();
    let value = json(error);

    assert_eq!(value["kind"], "appData");
    assert!(!dispatched);
}

#[wasm_bindgen_test]
fn invalid_timeout_is_rejected_by_constructor() {
    let error = match HttpToIpfsAdapter::new(app_data_fetch_callback(), Some(i32::MAX as u32 + 1)) {
        Ok(_) => panic!("oversized timeout must fail"),
        Err(error) => error,
    };
    let value = json(error);

    assert_eq!(value["kind"], "invalidInput");
    assert_eq!(value["field"], "timeoutMs");
}

#[wasm_bindgen_test]
async fn default_ipfs_client_rejects_malformed_hex_without_network() {
    let client = IpfsClient::new(None, None).unwrap();
    let error = client
        .fetch_app_data_from_hex("not-a-hex".to_owned())
        .await
        .expect_err("malformed app-data hash must fail");
    let value = json(error);

    assert_eq!(value["kind"], "appData");
}
