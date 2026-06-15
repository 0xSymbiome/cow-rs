//! Contract suite for the browser-wallet types that genuinely cross a boundary.
//!
//! Most browser-wallet types are returned to callers as plain Rust values and
//! are never serialized through their `serde` derive (the crate is not wired to
//! the wasm-bindgen JS boundary — that lives in `cow-sdk-wasm`), so that derived
//! JSON is not a consumer contract and is not pinned here. This suite pins the
//! cases that are real:
//!
//! - `WalletNativeCurrency` — embedded in the `wallet_addEthereumChain`
//!   (EIP-3085) RPC body, so its serialized wire shape is a real contract.
//! - `RpcErrorPayload` and `WalletChainParameters` — redaction contracts: their
//!   public serialization must collapse secret-bearing fields to `[redacted]`.

use cow_sdk_browser_wallet::{RpcErrorPayload, WalletChainParameters, WalletNativeCurrency};
use cow_sdk_core::SupportedChainId;
use serde::Serialize;
use serde_json::json;

fn assert_json_bytes<T>(value: &T, expected: &str)
where
    T: Serialize,
{
    let actual = serde_json::to_string(value).expect("browser-wallet type must serialize");
    assert_eq!(actual, expected);
}

#[test]
fn rpc_error_payload_new_preserves_wire_shape() {
    let payload = RpcErrorPayload::new(4902, "missing chain", Some(json!({ "detail": "kept" })));

    assert_eq!(payload.message.as_inner(), "missing chain");
    assert_eq!(
        payload
            .data
            .as_ref()
            .expect("data value should be preserved internally")
            .as_inner(),
        &json!({ "detail": "kept" })
    );
    assert_json_bytes(
        &payload,
        r#"{"code":4902,"message":"[redacted]","data":"[redacted]"}"#,
    );
}

#[test]
fn wallet_native_currency_new_preserves_wire_shape() {
    let native_currency =
        WalletNativeCurrency::new("Ether", "ETH", 18).expect("native currency must validate");

    assert_json_bytes(
        &native_currency,
        r#"{"name":"Ether","symbol":"ETH","decimals":18}"#,
    );
}

#[test]
fn wallet_chain_parameters_public_serialize_redacts_url_values() {
    let native_currency =
        WalletNativeCurrency::new("Ether", "ETH", 18).expect("native currency must validate");
    let parameters =
        WalletChainParameters::new(SupportedChainId::Sepolia, "Sepolia", native_currency)
            .expect("chain parameters must validate")
            .try_with_rpc_url("https://rpc.sepolia.example")
            .expect("rpc url must validate")
            .try_with_block_explorer_url("https://explorer.sepolia.example")
            .expect("explorer url must validate")
            .try_with_icon_url("https://cdn.example/icon.svg")
            .expect("icon url must validate");

    assert_json_bytes(
        &parameters,
        r#"{"chainId":11155111,"chainName":"Sepolia","nativeCurrency":{"name":"Ether","symbol":"ETH","decimals":18},"rpcUrls":["[redacted]"],"blockExplorerUrls":["[redacted]"],"iconUrls":["[redacted]"]}"#,
    );

    let debug = format!("{parameters:?}");
    assert!(debug.contains(cow_sdk_core::REDACTED_PLACEHOLDER));
    assert!(!debug.contains("rpc.sepolia.example"));
    assert!(!debug.contains("explorer.sepolia.example"));
    assert!(!debug.contains("cdn.example"));
}
