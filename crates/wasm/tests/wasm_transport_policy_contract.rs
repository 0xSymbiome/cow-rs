#![cfg(target_arch = "wasm32")]

mod common;

use cow_sdk_wasm::exports::{
    IpfsClient, IpfsClientConfig, OrderBookClient, OrderBookClientConfig, SubgraphClient,
    SubgraphClientConfig, TradingClient, TradingClientConfig,
};
use js_sys::{Function, Object, Reflect};
use serde_json::Value;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::*;

use crate::common::{APP_DATA_CONTENT, CHAIN_MAINNET, CID_APP_DATA};

wasm_bindgen_test_configure!(run_in_browser);

fn callback(args: &str, body: &str) -> Function {
    Function::new_with_args(args, body)
}

fn json(value: JsValue) -> Value {
    serde_wasm_bindgen::from_value(value).expect("JS value should decode to JSON")
}

fn set_js(target: &Object, key: &str, value: &JsValue) {
    Reflect::set(target, &JsValue::from_str(key), value).expect("test property should be set");
}

fn callback_transport(callback: &Function) -> Object {
    let transport = Object::new();
    set_js(&transport, "kind", &JsValue::from_str("callback"));
    set_js(&transport, "callback", callback.as_ref());
    transport
}

fn transport_policy(
    max_attempts: Option<u32>,
    base_delay_ms: Option<u32>,
    max_delay_ms: Option<u32>,
    jitter_strategy: Option<&str>,
    tokens_per_interval: Option<u32>,
    interval_ms: Option<u32>,
) -> Object {
    let policy = Object::new();

    if max_attempts.is_some() || base_delay_ms.is_some() || max_delay_ms.is_some() {
        let retry = Object::new();
        if let Some(max_attempts) = max_attempts {
            set_js(
                &retry,
                "maxAttempts",
                &JsValue::from_f64(f64::from(max_attempts)),
            );
        }
        if let Some(base_delay_ms) = base_delay_ms {
            set_js(
                &retry,
                "baseDelayMs",
                &JsValue::from_f64(f64::from(base_delay_ms)),
            );
        }
        if let Some(max_delay_ms) = max_delay_ms {
            set_js(
                &retry,
                "maxDelayMs",
                &JsValue::from_f64(f64::from(max_delay_ms)),
            );
        }
        set_js(&policy, "retryPolicy", retry.as_ref());
    }

    if let Some(jitter_strategy) = jitter_strategy {
        set_js(
            &policy,
            "jitterStrategy",
            &JsValue::from_str(jitter_strategy),
        );
    }

    if tokens_per_interval.is_some() || interval_ms.is_some() {
        let rate_limiter = Object::new();
        if let Some(tokens_per_interval) = tokens_per_interval {
            set_js(
                &rate_limiter,
                "tokensPerInterval",
                &JsValue::from_f64(f64::from(tokens_per_interval)),
            );
        }
        if let Some(interval_ms) = interval_ms {
            set_js(
                &rate_limiter,
                "intervalMs",
                &JsValue::from_f64(f64::from(interval_ms)),
            );
        }
        set_js(&rate_limiter, "scope", &JsValue::from_str("global"));
        set_js(&policy, "requestRateLimiter", rate_limiter.as_ref());
    }

    policy
}

fn ipfs_config(callback: &Function, policy: Option<&Object>) -> IpfsClientConfig {
    let config = Object::new();
    set_js(
        &config,
        "ipfsUri",
        &JsValue::from_str("https://ipfs.example.test/ipfs"),
    );
    set_js(&config, "transport", callback_transport(callback).as_ref());
    if let Some(policy) = policy {
        set_js(&config, "transportPolicy", policy.as_ref());
    }
    JsValue::from(config).unchecked_into()
}

fn orderbook_config(callback: &Function, policy: &Object) -> OrderBookClientConfig {
    let config = Object::new();
    set_js(
        &config,
        "chainId",
        &JsValue::from_f64(f64::from(CHAIN_MAINNET)),
    );
    set_js(&config, "env", &JsValue::from_str("prod"));
    set_js(&config, "transport", callback_transport(callback).as_ref());
    set_js(&config, "transportPolicy", policy.as_ref());
    JsValue::from(config).unchecked_into()
}

fn subgraph_config(callback: &Function, policy: &Object) -> SubgraphClientConfig {
    let config = Object::new();
    set_js(
        &config,
        "chainId",
        &JsValue::from_f64(f64::from(CHAIN_MAINNET)),
    );
    set_js(&config, "apiKey", &JsValue::from_str("test-api-key"));
    set_js(&config, "transport", callback_transport(callback).as_ref());
    set_js(&config, "transportPolicy", policy.as_ref());
    JsValue::from(config).unchecked_into()
}

fn trading_config(callback: &Function, policy: &Object) -> TradingClientConfig {
    let config = Object::new();
    set_js(
        &config,
        "chainId",
        &JsValue::from_f64(f64::from(CHAIN_MAINNET)),
    );
    set_js(&config, "env", &JsValue::from_str("prod"));
    set_js(&config, "appCode", &JsValue::from_str("test-app"));
    set_js(&config, "transport", callback_transport(callback).as_ref());
    set_js(&config, "transportPolicy", policy.as_ref());
    JsValue::from(config).unchecked_into()
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

#[wasm_bindgen_test]
fn all_client_constructors_accept_transport_policy() {
    let callback = success_callback();
    let policy = transport_policy(Some(2), Some(0), Some(0), Some("none"), Some(0), None);

    let _ = OrderBookClient::new(orderbook_config(&callback, &policy)).unwrap();
    let _ = SubgraphClient::new(subgraph_config(&callback, &policy)).unwrap();
    let _ = TradingClient::new(trading_config(&callback, &policy)).unwrap();
    let _ = IpfsClient::new(ipfs_config(&callback, Some(&policy))).unwrap();
}

#[wasm_bindgen_test]
async fn omitted_ipfs_policy_preserves_single_attempt_default() {
    let fetch = callback(
        "request",
        "globalThis.__cowDefaultPolicyAttempts = (globalThis.__cowDefaultPolicyAttempts || 0) + 1;
         return { status: 503, headers: {}, body: 'retryable' };",
    );
    let client = IpfsClient::new(ipfs_config(&fetch, None)).unwrap();
    let error = client
        .fetch_app_data_from_cid(CID_APP_DATA.to_owned(), None)
        .await
        .expect_err("default IPFS policy should not retry");
    let value = json(error);
    let attempts = js_sys::eval("globalThis.__cowDefaultPolicyAttempts")
        .unwrap()
        .as_f64()
        .unwrap() as u32;

    assert_eq!(value["kind"], "appData");
    assert_eq!(attempts, 1);
}

#[wasm_bindgen_test]
async fn retry_policy_override_retries_ipfs_fetch() {
    let fetch = callback(
        "request",
        &format!(
            "globalThis.__cowRetryPolicyAttempts = (globalThis.__cowRetryPolicyAttempts || 0) + 1;
             if (globalThis.__cowRetryPolicyAttempts === 1) {{
               return {{ status: 503, headers: {{}}, body: 'retryable' }};
             }}
             return {{ status: 200, headers: {{}}, body: '{}' }};",
            APP_DATA_CONTENT.replace('\'', "\\'")
        ),
    );
    let policy = transport_policy(Some(2), Some(0), Some(0), Some("none"), None, None);
    let client = IpfsClient::new(ipfs_config(&fetch, Some(&policy))).unwrap();
    let value = json(
        client
            .fetch_app_data_from_cid(CID_APP_DATA.to_owned(), None)
            .await
            .unwrap(),
    );
    let attempts = js_sys::eval("globalThis.__cowRetryPolicyAttempts")
        .unwrap()
        .as_f64()
        .unwrap() as u32;

    assert_eq!(value["value"]["document"]["appCode"], "CoW Swap");
    assert_eq!(attempts, 2);
}

#[wasm_bindgen_test]
async fn retry_delay_and_jitter_override_are_applied() {
    let fetch = callback(
        "request",
        &format!(
            "globalThis.__cowRetryPolicyTimes = globalThis.__cowRetryPolicyTimes || [];
             globalThis.__cowRetryPolicyTimes.push(Date.now());
             if (globalThis.__cowRetryPolicyTimes.length === 1) {{
               return {{ status: 503, headers: {{}}, body: 'retryable' }};
             }}
             return {{ status: 200, headers: {{}}, body: '{}' }};",
            APP_DATA_CONTENT.replace('\'', "\\'")
        ),
    );
    let policy = transport_policy(Some(2), Some(35), Some(35), Some("none"), None, None);
    let client = IpfsClient::new(ipfs_config(&fetch, Some(&policy))).unwrap();
    client
        .fetch_app_data_from_cid(CID_APP_DATA.to_owned(), None)
        .await
        .unwrap();
    let delta =
        js_sys::eval("globalThis.__cowRetryPolicyTimes[1] - globalThis.__cowRetryPolicyTimes[0]")
            .unwrap()
            .as_f64()
            .unwrap();

    assert!(
        delta >= 20.0,
        "retry delay should be visible, got {delta}ms"
    );
}

#[wasm_bindgen_test]
async fn rate_limiter_override_throttles_ipfs_fetches() {
    let fetch = callback(
        "request",
        &format!(
            "globalThis.__cowRateLimitTimes = globalThis.__cowRateLimitTimes || [];
             globalThis.__cowRateLimitTimes.push(Date.now());
             return {{ status: 200, headers: {{}}, body: '{}' }};",
            APP_DATA_CONTENT.replace('\'', "\\'")
        ),
    );
    let policy = transport_policy(Some(1), None, None, None, Some(1), Some(35));
    let client = IpfsClient::new(ipfs_config(&fetch, Some(&policy))).unwrap();

    client
        .fetch_app_data_from_cid(CID_APP_DATA.to_owned(), None)
        .await
        .unwrap();
    client
        .fetch_app_data_from_cid(CID_APP_DATA.to_owned(), None)
        .await
        .unwrap();

    let delta =
        js_sys::eval("globalThis.__cowRateLimitTimes[1] - globalThis.__cowRateLimitTimes[0]")
            .unwrap()
            .as_f64()
            .unwrap();

    assert!(
        delta >= 20.0,
        "rate limiter should be visible, got {delta}ms"
    );
}

#[wasm_bindgen_test]
fn invalid_transport_policy_user_agent_is_rejected() {
    let fetch = callback(
        "request",
        "globalThis.__cowInvalidPolicyDispatched = true;
         return { status: 200, headers: {}, body: '{}' };",
    );
    let policy = Object::new();
    set_js(&policy, "userAgent", &JsValue::from_str(""));
    let error = match IpfsClient::new(ipfs_config(&fetch, Some(&policy))) {
        Ok(_) => panic!("invalid transport policy should fail construction"),
        Err(error) => error,
    };
    let value = json(error);
    let dispatched = js_sys::eval("Boolean(globalThis.__cowInvalidPolicyDispatched)")
        .unwrap()
        .as_bool()
        .unwrap();

    assert_eq!(value["kind"], "invalidInput");
    assert_eq!(value["field"], "transportPolicy.userAgent");
    assert!(!dispatched);
}
