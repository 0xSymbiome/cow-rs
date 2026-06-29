#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_js::exports::{
    AppDataParams, IpfsClient, OrderBookClient, SubgraphClient, TradingClient, app_data_doc,
    app_data_hex_to_cid, app_data_info, cid_to_app_data_hex, compute_order_uid,
    deployment_addresses, domain_separator, order_typed_data, supported_chain_ids,
    validate_app_data_doc, wasm_version,
};
use js_sys::Function;
use serde_json::Value;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::*;

use crate::common::{
    ADDR_OWNER, APP_DATA_CONTENT, CHAIN_MAINNET, CHAIN_UNSUPPORTED, CID_APP_DATA, HASH_APP_DATA,
    ipfs_config, orderbook_config, subgraph_config, trading_config, wasm_app_data_input,
    wasm_order_input,
};

wasm_bindgen_test_configure!(run_in_browser);

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

fn error_json(value: JsValue) -> Value {
    json(value)
}

fn callback(args: &str, body: &str) -> Function {
    Function::new_with_args(args, body)
}

fn empty_fetch_callback() -> Function {
    callback(
        "request",
        "return { status: 200, headers: {}, body: '{}' };",
    )
}

#[wasm_bindgen_test]
fn domain_separator_returns_hex_string() {
    let separator = domain_separator(CHAIN_MAINNET)
        .expect("mainnet separator should exist")
        .as_string()
        .expect("the domain separator is a hex string");

    assert_eq!(separator.len(), 66);
    assert!(separator.starts_with("0x"));
}

#[wasm_bindgen_test]
fn unsupported_chain_returns_typed_error() {
    let error = domain_separator(CHAIN_UNSUPPORTED).expect_err("unsupported chain must fail");
    let value = error_json(error);

    assert_eq!(value["kind"], "unsupportedChain");
    assert_eq!(value["chainId"], CHAIN_UNSUPPORTED);
}

#[wasm_bindgen_test]
fn supported_chain_ids_are_stable() {
    let ids = supported_chain_ids();

    assert_eq!(ids.first(), Some(&CHAIN_MAINNET));
    assert!(ids.contains(&100));
    assert!(ids.contains(&11_155_111));
}

#[wasm_bindgen_test]
fn deployment_addresses_use_schema_envelope() {
    let value = json(deployment_addresses(CHAIN_MAINNET, None).unwrap());

    assert_eq!(value["schemaVersion"], "v1");
    assert!(
        value["value"]["settlement"]
            .as_str()
            .unwrap()
            .starts_with("0x")
    );
    assert!(
        value["value"]["vaultRelayer"]
            .as_str()
            .unwrap()
            .starts_with("0x")
    );
}

#[wasm_bindgen_test]
fn order_typed_data_serializes_to_expected_js_shape() {
    let value = json(order_typed_data(wasm_order_input(), CHAIN_MAINNET).unwrap());

    assert_eq!(value["schemaVersion"], "v1");
    assert_eq!(value["value"]["primaryType"], "Order");
    assert_eq!(value["value"]["domain"]["chainId"], CHAIN_MAINNET);
    assert_eq!(
        value["value"]["message"]["sellToken"],
        crate::common::ADDR_SELL
    );
}

#[wasm_bindgen_test]
fn order_typed_data_rejects_unsupported_chain() {
    let error =
        order_typed_data(wasm_order_input(), CHAIN_UNSUPPORTED).expect_err("chain must fail");
    let value = error_json(error);

    assert_eq!(value["kind"], "unsupportedChain");
}

#[wasm_bindgen_test]
fn compute_order_uid_returns_uid_and_digest_strings() {
    let value =
        json(compute_order_uid(wasm_order_input(), CHAIN_MAINNET, ADDR_OWNER.to_owned()).unwrap());

    assert_eq!(value["schemaVersion"], "v1");
    assert_eq!(value["value"]["orderUid"].as_str().unwrap().len(), 114);
    assert_eq!(value["value"]["orderDigest"].as_str().unwrap().len(), 66);
}

#[wasm_bindgen_test]
fn compute_order_uid_rejects_malformed_owner() {
    let error = compute_order_uid(wasm_order_input(), CHAIN_MAINNET, "0x1234".to_owned())
        .expect_err("malformed owner must fail");
    let value = error_json(error);

    assert_eq!(value["kind"], "invalidInput");
    assert_eq!(value["field"], "owner");
}

#[wasm_bindgen_test]
fn app_data_doc_returns_versioned_document() {
    let value = json(app_data_doc(wasm_app_data_input()).unwrap());

    assert_eq!(value["schemaVersion"], "v1");
    assert_eq!(value["value"]["document"]["appCode"], "CoW Swap");
    assert_eq!(value["value"]["document"]["version"], "0.7.0");
}

#[wasm_bindgen_test]
fn app_data_info_returns_cid_hash_and_content() {
    let value = json(app_data_info(wasm_app_data_input()).unwrap());

    assert_eq!(value["schemaVersion"], "v1");
    assert_eq!(value["value"]["cid"], CID_APP_DATA);
    assert_eq!(value["value"]["appDataHex"], HASH_APP_DATA);
    assert_eq!(value["value"]["appDataContent"], APP_DATA_CONTENT);
}

#[wasm_bindgen_test]
fn app_data_validation_succeeds_for_canonical_doc() {
    let value = json(validate_app_data_doc(wasm_app_data_input()).unwrap());

    assert_eq!(value["schemaVersion"], "v1");
    assert_eq!(value["value"]["success"], true);
    assert!(value["value"].get("errors").is_none());
}

#[wasm_bindgen_test]
fn app_data_input_rejects_non_object_metadata() {
    let input = AppDataParams {
        app_code: "CoW Swap".to_owned(),
        metadata: Value::String("invalid".to_owned()),
        version: "0.7.0".to_owned(),
        environment: None,
    };
    let error = app_data_info(input).expect_err("metadata must be an object");
    let value = error_json(error);

    assert_eq!(value["kind"], "invalidInput");
    assert_eq!(value["field"], "metadata");
}

#[wasm_bindgen_test]
fn app_data_hex_and_cid_round_trip() {
    let cid = json(app_data_hex_to_cid(HASH_APP_DATA.to_owned()).unwrap())["value"]
        .as_str()
        .unwrap()
        .to_owned();
    let hash = json(cid_to_app_data_hex(cid.clone()).unwrap())["value"]
        .as_str()
        .unwrap()
        .to_owned();

    assert_eq!(cid, CID_APP_DATA);
    assert_eq!(hash, HASH_APP_DATA);
}

#[wasm_bindgen_test]
fn invalid_cid_returns_typed_error() {
    let error = cid_to_app_data_hex("not-a-cid".to_owned()).expect_err("malformed CID must fail");
    let value = error_json(error);

    assert_eq!(value["kind"], "appData");
    assert!(
        value["message"]
            .as_str()
            .unwrap()
            .contains("invalid cid format")
    );
}

#[wasm_bindgen_test]
fn client_constructors_accept_supported_runtime_inputs() {
    let fetch = empty_fetch_callback();

    let _orderbook = OrderBookClient::new(orderbook_config(CHAIN_MAINNET, None, &fetch)).unwrap();
    let _subgraph =
        SubgraphClient::new(subgraph_config(CHAIN_MAINNET, "test-key", &fetch)).unwrap();
    let _trading =
        TradingClient::new(trading_config(CHAIN_MAINNET, None, "CoW Swap", &fetch)).unwrap();
    let _ipfs = IpfsClient::new(ipfs_config(None, Some(500), &fetch)).unwrap();
}

#[wasm_bindgen_test]
fn client_constructors_reject_unsupported_chain() {
    let fetch = empty_fetch_callback();

    assert!(OrderBookClient::new(orderbook_config(CHAIN_UNSUPPORTED, None, &fetch)).is_err());
    assert!(SubgraphClient::new(subgraph_config(CHAIN_UNSUPPORTED, "test-key", &fetch)).is_err());
    assert!(
        TradingClient::new(trading_config(CHAIN_UNSUPPORTED, None, "CoW Swap", &fetch)).is_err()
    );
}

#[wasm_bindgen_test]
fn wasm_version_matches_crate_version() {
    assert_eq!(wasm_version(), env!("CARGO_PKG_VERSION"));
}
