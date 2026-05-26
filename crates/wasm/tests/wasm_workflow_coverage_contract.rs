#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_wasm::exports::{
    AllowanceParametersInput, LimitTradeParametersInput, OrderBookClient, OrderBookClientConfig,
    OrderKindDto, OrderTraderParametersInput, PaginationOptions, QuoteResponseRefInput,
    QuoteResultsInput, TokenBalanceDto, TradesQueryInput, TradingClient, build_cancel_order_tx,
    build_presign_tx, compute_order_uid,
};
use js_sys::{Function, Object, Reflect};
use serde_json::Value;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

use crate::common::{
    ADDR_BUY, ADDR_OWNER, ADDR_RECEIVER, ADDR_SELL, CHAIN_MAINNET, ECDSA_SIGNATURE, HASH_APP_DATA,
    wasm_order_input,
};

wasm_bindgen_test_configure!(run_in_browser);

const NATIVE_TOKEN: &str = "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE";

fn callback(args: &str, body: &str) -> Function {
    Function::new_with_args(args, body)
}

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

fn set_js(target: &Object, key: &str, value: &JsValue) {
    Reflect::set(target, &JsValue::from_str(key), value).expect("test object field should be set");
}

fn callback_transport(callback: &Function) -> Object {
    let transport = Object::new();
    set_js(&transport, "kind", &JsValue::from_str("callback"));
    set_js(&transport, "callback", callback.as_ref());
    transport
}

fn orderbook_config_with_api_key(callback: &Function, api_key: &str) -> OrderBookClientConfig {
    let config = Object::new();
    set_js(
        &config,
        "chainId",
        &JsValue::from_f64(f64::from(CHAIN_MAINNET)),
    );
    set_js(&config, "apiKey", &JsValue::from_str(api_key));
    set_js(&config, "transport", callback_transport(callback).as_ref());
    JsValue::from(config).unchecked_into()
}

fn recorded_requests() -> Vec<Value> {
    json(js_sys::eval("globalThis.__cowCoverageRequests").unwrap())
        .as_array()
        .expect("recorded requests should be an array")
        .clone()
}

fn generated_order_uid() -> String {
    let value =
        json(compute_order_uid(wasm_order_input(), CHAIN_MAINNET, ADDR_OWNER.to_owned()).unwrap());
    value["value"]["orderUid"].as_str().unwrap().to_owned()
}

fn future_valid_to() -> u32 {
    (js_sys::Date::now() / 1000.0) as u32 + 3_600
}

fn post_fetch_callback(uid: &str) -> Function {
    callback(
        "request",
        &format!(
            r#"
            globalThis.__cowCoverageRequests = globalThis.__cowCoverageRequests || [];
            globalThis.__cowCoverageRequests.push(request);
            if (request.method === "PUT" && request.url.includes("/api/v1/app_data/")) {{
              const body = JSON.parse(request.body);
              return {{ status: 200, headers: {{}}, body: JSON.stringify({{ fullAppData: body.fullAppData }}) }};
            }}
            return {{ status: 200, headers: {{}}, body: JSON.stringify("{}") }};
            "#,
            uid
        ),
    )
}

fn signer_callback() -> Function {
    callback(
        "envelope",
        &format!(
            "globalThis.__cowCoverageSignerEnvelope = envelope; return '{}';",
            ECDSA_SIGNATURE
        ),
    )
}

fn limit_params(valid_to: u32) -> LimitTradeParametersInput {
    LimitTradeParametersInput {
        kind: OrderKindDto::Sell,
        owner: None,
        sell_token: ADDR_SELL.to_owned(),
        buy_token: ADDR_BUY.to_owned(),
        sell_amount: "1000000000000000000".to_owned(),
        buy_amount: "2000000000000000000".to_owned(),
        quote_id: None,
        env: None,
        settlement_contract_override: None,
        eth_flow_contract_override: None,
        partially_fillable: false,
        sell_token_balance: Some(TokenBalanceDto::Erc20),
        buy_token_balance: Some(TokenBalanceDto::Erc20),
        slippage_bps: Some(0),
        receiver: Some(ADDR_RECEIVER.to_owned()),
        valid_for: None,
        valid_to: Some(valid_to),
        partner_fee: None,
    }
}

#[wasm_bindgen_test]
async fn orderbook_lists_support_api_key_routing_and_owner_trade_pagination() {
    js_sys::eval("globalThis.__cowCoverageRequests = []").unwrap();
    let fetch = callback(
        "request",
        "globalThis.__cowCoverageRequests.push(request);
         return { status: 200, headers: {}, body: '[]' };",
    );
    let client = OrderBookClient::new(orderbook_config_with_api_key(&fetch, "test-api-key"))
        .expect("orderbook config should accept apiKey");

    let orders = json(
        client
            .get_orders(
                ADDR_OWNER.to_owned(),
                Some(PaginationOptions {
                    offset: Some(7),
                    limit: Some(13),
                }),
                None,
            )
            .await
            .unwrap(),
    );
    let trades = json(
        client
            .get_trades(
                TradesQueryInput {
                    owner: Some(ADDR_OWNER.to_owned()),
                    order_uid: None,
                    offset: Some(3),
                    limit: Some(5),
                },
                None,
            )
            .await
            .unwrap(),
    );
    let requests = recorded_requests();

    assert_eq!(orders["value"].as_array().unwrap().len(), 0);
    assert_eq!(trades["value"].as_array().unwrap().len(), 0);
    assert!(
        requests[0]["url"]
            .as_str()
            .unwrap()
            .starts_with("https://partners.cow.fi/mainnet")
    );
    assert!(
        requests[0]["url"]
            .as_str()
            .unwrap()
            .contains(&format!("/api/v1/account/{ADDR_OWNER}/orders"))
    );
    assert!(requests[0]["url"].as_str().unwrap().contains("offset=7"));
    assert!(requests[0]["url"].as_str().unwrap().contains("limit=13"));
    assert!(
        requests[1]["url"]
            .as_str()
            .unwrap()
            .contains(&format!("owner={ADDR_OWNER}"))
    );
    assert!(requests[1]["url"].as_str().unwrap().contains("offset=3"));
    assert!(requests[1]["url"].as_str().unwrap().contains("limit=5"));
    assert!(
        requests[0]["headers"]
            .as_object()
            .unwrap()
            .iter()
            .any(|(name, value)| name.eq_ignore_ascii_case("x-api-key")
                && value.as_str() == Some("test-api-key"))
    );
}

#[wasm_bindgen_test]
async fn trading_posts_swap_from_quote_and_limit_orders_through_typed_signers() {
    js_sys::eval("globalThis.__cowCoverageRequests = []").unwrap();
    let uid = generated_order_uid();
    let fetch = post_fetch_callback(&uid);
    let client = TradingClient::new(crate::common::trading_config(
        CHAIN_MAINNET,
        None,
        "CoW Swap",
        &fetch,
    ))
    .unwrap();
    let valid_to = future_valid_to();

    let swap = json(
        client
            .post_swap_order_from_quote(
                QuoteResultsInput {
                    order_to_sign: wasm_order_input(),
                    quote_response: Some(QuoteResponseRefInput { id: Some(91) }),
                    quote_id: None,
                },
                ADDR_OWNER.to_owned(),
                signer_callback(),
                None,
            )
            .await
            .unwrap(),
    );
    let limit = json(
        client
            .post_limit_order(
                limit_params(valid_to),
                ADDR_OWNER.to_owned(),
                signer_callback(),
                None,
            )
            .await
            .unwrap(),
    );
    let requests = recorded_requests();
    let swap_body: Value =
        serde_json::from_str(requests[0]["body"].as_str().unwrap()).expect("order body");
    let limit_body: Value =
        serde_json::from_str(requests[2]["body"].as_str().unwrap()).expect("order body");
    let signer_envelope = json(js_sys::eval("globalThis.__cowCoverageSignerEnvelope").unwrap());

    assert_eq!(swap["value"]["orderId"], uid);
    assert_eq!(swap_body["quoteId"], 91);
    assert_eq!(swap_body["signingScheme"], "eip712");
    assert_eq!(limit["value"]["orderId"], uid);
    assert_eq!(limit_body["from"], ADDR_OWNER);
    assert_eq!(limit_body["validTo"], valid_to);
    assert_eq!(signer_envelope["primaryType"], "Order");
}

#[wasm_bindgen_test]
async fn trading_exposes_allowance_and_transaction_builders() {
    let fetch = callback(
        "request",
        "return { status: 200, headers: {}, body: '{}' };",
    );
    let client = TradingClient::new(crate::common::trading_config(
        CHAIN_MAINNET,
        None,
        "CoW Swap",
        &fetch,
    ))
    .unwrap();
    let allowance_reader = callback(
        "request",
        "globalThis.__cowAllowanceRead = request; return '57';",
    );

    let allowance = json(
        client
            .get_cow_protocol_allowance(
                AllowanceParametersInput {
                    token_address: ADDR_SELL.to_owned(),
                    owner: ADDR_OWNER.to_owned(),
                    chain_id: None,
                    env: None,
                    vault_relayer_override: None,
                },
                allowance_reader,
                None,
            )
            .await
            .unwrap(),
    );
    let allowance_request = json(js_sys::eval("globalThis.__cowAllowanceRead").unwrap());

    let mut native_order = wasm_order_input();
    native_order.sell_token = NATIVE_TOKEN.to_owned();
    native_order.valid_to = future_valid_to();
    native_order.app_data = HASH_APP_DATA.to_owned();
    let ethflow = json(
        client
            .build_sell_native_currency_tx(native_order, 77, ADDR_OWNER.to_owned(), None)
            .await
            .unwrap(),
    );
    let order_uid = generated_order_uid();
    let presign = json(
        build_presign_tx(OrderTraderParametersInput {
            order_uid: order_uid.clone(),
            chain_id: Some(CHAIN_MAINNET),
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        })
        .unwrap(),
    );
    let cancel = json(
        build_cancel_order_tx(OrderTraderParametersInput {
            order_uid,
            chain_id: Some(CHAIN_MAINNET),
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        })
        .unwrap(),
    );

    assert_eq!(allowance["value"], "57");
    assert_eq!(allowance_request["method"], "allowance");
    assert_eq!(allowance_request["address"], ADDR_SELL);
    assert_eq!(
        ethflow["value"]["transaction"]["value"],
        "1000000000000000000"
    );
    assert!(
        ethflow["value"]["transaction"]["data"]
            .as_str()
            .unwrap()
            .starts_with("0x")
    );
    assert_eq!(ethflow["value"]["orderUid"].as_str().unwrap().len(), 114);
    assert_eq!(presign["value"]["value"], "0");
    assert_eq!(cancel["value"]["value"], "0");
    assert!(presign["value"]["data"].as_str().unwrap().starts_with("0x"));
    assert!(cancel["value"]["data"].as_str().unwrap().starts_with("0x"));
}
