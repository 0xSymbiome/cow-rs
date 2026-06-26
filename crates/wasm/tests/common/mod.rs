#![allow(
    dead_code,
    reason = "shared test-helper module aggregates fixtures, constants, and adapters that not every integration test binary exercises; an integration test may use only a subset of the shared helpers without leaving the others permanently unused"
)]

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsCast;

pub const ADDR_SELL: &str = "0x1111111111111111111111111111111111111111";
pub const ADDR_BUY: &str = "0x2222222222222222222222222222222222222222";
pub const ADDR_OWNER: &str = "0x3333333333333333333333333333333333333333";
pub const ADDR_RECEIVER: &str = "0x4444444444444444444444444444444444444444";
pub const ADDR_ZERO: &str = "0x0000000000000000000000000000000000000000";

pub const HASH_APP_DATA: &str =
    "0x337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";
pub const HASH_APP_DATA_TWO: &str =
    "0x8af4e8c9973577b08ac21d17d331aade86c11ebcc5124744d621ca8365ec9424";
pub const CID_APP_DATA: &str =
    "f01551b20337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df";
pub const CID_APP_DATA_TWO: &str =
    "f01551b208af4e8c9973577b08ac21d17d331aade86c11ebcc5124744d621ca8365ec9424";

pub const VALID_TO: u32 = 1_735_689_600;
pub const CHAIN_MAINNET: u32 = 1;
pub const CHAIN_GNOSIS: u32 = 100;
pub const CHAIN_UNSUPPORTED: u32 = 13_337;

pub const APP_DATA_CONTENT: &str = r#"{"appCode":"CoW Swap","metadata":{},"version":"0.7.0"}"#;

pub const ECDSA_SIGNATURE: &str = "0x111111111111111111111111111111111111111111111111111111111111111122222222222222222222222222222222222222222222222222222222222222221b";
pub const ECDSA_SIGNATURE_RECOVERY_28: &str = "0x111111111111111111111111111111111111111111111111111111111111111122222222222222222222222222222222222222222222222222222222222222221c";
pub const ECDSA_SIGNATURE_MODERN_V: &str = "0x1111111111111111111111111111111111111111111111111111111111111111222222222222222222222222222222222222222222222222222222222222222200";
pub const ECDSA_SIGNATURE_MODERN_V_ONE: &str = "0x1111111111111111111111111111111111111111111111111111111111111111222222222222222222222222222222222222222222222222222222222222222201";
pub const EIP1271_SIGNATURE: &str = "0x0000000000000000000000001111111111111111111111111111111111111111000000000000000000000000222222222222222222222222222222222222222200000000000000000000000044444444444444444444444444444444444444440000000000000000000000000000000000000000000000000de0b6b3a76400000000000000000000000000000000000000000000000000001bc16d674ec800000000000000000000000000000000000000000000000000000000000067748580337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df0000000000000000000000000000000000000000000000000000000000000000f3b277728b3fee749481eb3e0b3b48980dbbab78658fc419025cb16eee34677500000000000000000000000000000000000000000000000000000000000000005a28e9363bb942b639270062aa6bb295f434bcdfc42c97267bf003f272060dc95a28e9363bb942b639270062aa6bb295f434bcdfc42c97267bf003f272060dc900000000000000000000000000000000000000000000000000000000000001a00000000000000000000000000000000000000000000000000000000000000041111111111111111111111111111111111111111111111111111111111111111122222222222222222222222222222222222222222222222222222222222222221b00000000000000000000000000000000000000000000000000000000000000";

#[cfg(not(target_arch = "wasm32"))]
pub fn host_order_input() -> cow_sdk_core::OrderData {
    cow_sdk_core::OrderData {
        sell_token: cow_sdk_core::Address::new(ADDR_SELL).unwrap(),
        buy_token: cow_sdk_core::Address::new(ADDR_BUY).unwrap(),
        receiver: cow_sdk_core::Address::new(ADDR_RECEIVER).unwrap(),
        sell_amount: cow_sdk_core::Amount::new("1000000000000000000").unwrap(),
        buy_amount: cow_sdk_core::Amount::new("2000000000000000000").unwrap(),
        valid_to: VALID_TO,
        app_data: cow_sdk_core::AppDataHash::new(HASH_APP_DATA).unwrap(),
        fee_amount: cow_sdk_core::Amount::new("0").unwrap(),
        kind: cow_sdk_core::OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: cow_sdk_core::SellTokenSource::Erc20,
        buy_token_balance: cow_sdk_core::BuyTokenDestination::Erc20,
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn host_app_data_input() -> cow_sdk_wasm::helpers::dto::AppDataParams {
    cow_sdk_wasm::helpers::dto::AppDataParams {
        app_code: "CoW Swap".to_owned(),
        metadata: serde_json::json!({}),
        version: "0.7.0".to_owned(),
        environment: None,
    }
}

#[cfg(target_arch = "wasm32")]
pub fn wasm_order_input() -> cow_sdk_core::OrderData {
    cow_sdk_core::OrderData {
        sell_token: cow_sdk_core::Address::new(ADDR_SELL).unwrap(),
        buy_token: cow_sdk_core::Address::new(ADDR_BUY).unwrap(),
        receiver: cow_sdk_core::Address::new(ADDR_RECEIVER).unwrap(),
        sell_amount: cow_sdk_core::Amount::new("1000000000000000000").unwrap(),
        buy_amount: cow_sdk_core::Amount::new("2000000000000000000").unwrap(),
        valid_to: VALID_TO,
        app_data: cow_sdk_core::AppDataHash::new(HASH_APP_DATA).unwrap(),
        fee_amount: cow_sdk_core::Amount::new("0").unwrap(),
        kind: cow_sdk_core::OrderKind::Sell,
        partially_fillable: false,
        sell_token_balance: cow_sdk_core::SellTokenSource::Erc20,
        buy_token_balance: cow_sdk_core::BuyTokenDestination::Erc20,
    }
}

#[cfg(target_arch = "wasm32")]
pub fn wasm_app_data_input() -> cow_sdk_wasm::exports::AppDataParams {
    cow_sdk_wasm::exports::AppDataParams {
        app_code: "CoW Swap".to_owned(),
        metadata: serde_json::json!({}),
        version: "0.7.0".to_owned(),
        environment: None,
    }
}

#[cfg(target_arch = "wasm32")]
fn set_js(target: &js_sys::Object, key: &str, value: &wasm_bindgen::JsValue) {
    js_sys::Reflect::set(target, &wasm_bindgen::JsValue::from_str(key), value)
        .expect("test config property should be set");
}

#[cfg(target_arch = "wasm32")]
fn callback_transport(callback: &js_sys::Function) -> js_sys::Object {
    let transport = js_sys::Object::new();
    set_js(
        &transport,
        "kind",
        &wasm_bindgen::JsValue::from_str("callback"),
    );
    set_js(&transport, "callback", callback.as_ref());
    transport
}

#[cfg(target_arch = "wasm32")]
pub fn orderbook_config(
    chain_id: u32,
    env: Option<&str>,
    callback: &js_sys::Function,
) -> cow_sdk_wasm::exports::OrderBookClientConfig {
    let config = js_sys::Object::new();
    set_js(
        &config,
        "chainId",
        &wasm_bindgen::JsValue::from_f64(f64::from(chain_id)),
    );
    if let Some(env) = env {
        set_js(&config, "env", &wasm_bindgen::JsValue::from_str(env));
    }
    set_js(&config, "transport", callback_transport(callback).as_ref());
    wasm_bindgen::JsValue::from(config).unchecked_into()
}

#[cfg(target_arch = "wasm32")]
pub fn subgraph_config(
    chain_id: u32,
    api_key: &str,
    callback: &js_sys::Function,
) -> cow_sdk_wasm::exports::SubgraphClientConfig {
    let config = js_sys::Object::new();
    set_js(
        &config,
        "chainId",
        &wasm_bindgen::JsValue::from_f64(f64::from(chain_id)),
    );
    set_js(&config, "apiKey", &wasm_bindgen::JsValue::from_str(api_key));
    set_js(&config, "transport", callback_transport(callback).as_ref());
    wasm_bindgen::JsValue::from(config).unchecked_into()
}

#[cfg(target_arch = "wasm32")]
pub fn trading_config(
    chain_id: u32,
    env: Option<&str>,
    app_code: &str,
    callback: &js_sys::Function,
) -> cow_sdk_wasm::exports::TradingClientConfig {
    let config = js_sys::Object::new();
    set_js(
        &config,
        "chainId",
        &wasm_bindgen::JsValue::from_f64(f64::from(chain_id)),
    );
    if let Some(env) = env {
        set_js(&config, "env", &wasm_bindgen::JsValue::from_str(env));
    }
    set_js(
        &config,
        "appCode",
        &wasm_bindgen::JsValue::from_str(app_code),
    );
    set_js(&config, "transport", callback_transport(callback).as_ref());
    wasm_bindgen::JsValue::from(config).unchecked_into()
}

#[cfg(target_arch = "wasm32")]
pub fn ipfs_config(
    ipfs_uri: Option<&str>,
    timeout_ms: Option<u32>,
    callback: &js_sys::Function,
) -> cow_sdk_wasm::exports::IpfsClientConfig {
    let config = js_sys::Object::new();
    if let Some(ipfs_uri) = ipfs_uri {
        set_js(
            &config,
            "ipfsUri",
            &wasm_bindgen::JsValue::from_str(ipfs_uri),
        );
    }
    if let Some(timeout_ms) = timeout_ms {
        set_js(
            &config,
            "timeoutMs",
            &wasm_bindgen::JsValue::from_f64(f64::from(timeout_ms)),
        );
    }
    set_js(&config, "transport", callback_transport(callback).as_ref());
    wasm_bindgen::JsValue::from(config).unchecked_into()
}
