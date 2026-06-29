#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_js::exports::IpfsClient;
use js_sys::Function;
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use crate::common::{APP_DATA_CONTENT, CID_APP_DATA, HASH_APP_DATA, ipfs_config};

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
async fn ipfs_client_fetches_app_data_from_cid() {
    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        Some(500),
        &app_data_fetch_callback(),
    ))
    .unwrap();
    let value = json(
        client
            .fetch_app_data_from_cid(CID_APP_DATA.to_owned(), None)
            .await
            .unwrap(),
    );

    assert_eq!(value["schemaVersion"], "v1");
    assert_eq!(value["value"]["document"]["appCode"], "CoW Swap");
}

#[wasm_bindgen_test]
async fn ipfs_client_fetches_app_data_from_hex() {
    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        None,
        &app_data_fetch_callback(),
    ))
    .unwrap();
    let value = json(
        client
            .fetch_app_data_from_hex(HASH_APP_DATA.to_owned(), None)
            .await
            .unwrap(),
    );

    assert_eq!(value["value"]["document"]["version"], "0.7.0");
}

#[wasm_bindgen_test]
async fn ipfs_client_uses_custom_gateway_url() {
    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        Some(500),
        &app_data_fetch_callback(),
    ))
    .unwrap();
    client
        .fetch_app_data_from_cid(CID_APP_DATA.to_owned(), None)
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
async fn ipfs_client_keeps_internal_callback_registration_alive() {
    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        None,
        &app_data_fetch_callback(),
    ))
    .unwrap();
    let value = json(
        client
            .fetch_app_data_from_cid(CID_APP_DATA.to_owned(), None)
            .await
            .unwrap(),
    );

    assert_eq!(value["value"]["document"]["appCode"], "CoW Swap");
}

#[wasm_bindgen_test]
async fn http_404_maps_to_app_data_error() {
    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        None,
        &callback(
            "request",
            "return { status: 404, headers: {}, body: 'not found' };",
        ),
    ))
    .unwrap();
    let error = client
        .fetch_app_data_from_cid(CID_APP_DATA.to_owned(), None)
        .await
        .expect_err("404 must fail");
    let value = json(error);

    assert_eq!(value["kind"], "appData");
    assert_eq!(value["class"], "other");
}

#[wasm_bindgen_test]
async fn invalid_hex_rejects_before_fetch_callback() {
    let client = IpfsClient::new(ipfs_config(
        Some("https://ipfs.example.test/ipfs"),
        None,
        &callback(
            "request",
            "globalThis.__cowUnexpectedIpfsDispatch = true; return { status: 200, headers: {}, body: '{}' };",
        ),
    ))
    .unwrap();
    let error = client
        .fetch_app_data_from_hex("not-a-hex".to_owned(), None)
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
    let callback = app_data_fetch_callback();
    let error = match IpfsClient::new(ipfs_config(None, Some(i32::MAX as u32 + 1), &callback)) {
        Ok(_) => panic!("oversized timeout must fail"),
        Err(error) => error,
    };
    let value = json(error);

    assert_eq!(value["kind"], "invalidInput");
    assert_eq!(value["field"], "timeoutMs");
}

#[wasm_bindgen_test]
async fn ipfs_client_rejects_malformed_hex_without_network() {
    let client = IpfsClient::new(ipfs_config(None, None, &app_data_fetch_callback())).unwrap();
    let error = client
        .fetch_app_data_from_hex("not-a-hex".to_owned(), None)
        .await
        .expect_err("malformed app-data hash must fail");
    let value = json(error);

    assert_eq!(value["kind"], "appData");
}
