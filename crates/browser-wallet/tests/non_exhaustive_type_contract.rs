use cow_sdk_browser_wallet::{
    InjectedWalletDetectionOptions, InjectedWalletDiscoverySource, InjectedWalletInfo,
    RpcErrorPayload, WalletChainChange, WalletChainChangeKind, WalletChainParameters, WalletEvent,
    WalletNativeCurrency, WalletSession,
};
use cow_sdk_core::{Address, SupportedChainId};
use serde::Serialize;
use serde_json::json;

const ADDR1: &str = "0x1111111111111111111111111111111111111111";
const ADDR2: &str = "0x2222222222222222222222222222222222222222";

fn assert_json_bytes<T>(value: &T, expected: &str)
where
    T: Serialize,
{
    let actual = serde_json::to_string(value).expect("browser-wallet type must serialize");
    assert_eq!(actual, expected);
}

fn address(value: &str) -> Address {
    Address::new(value).expect("address literal must stay valid")
}

#[test]
fn wallet_session_new_preserves_wire_shape() {
    let session = WalletSession::new(
        true,
        Some(u64::from(SupportedChainId::Sepolia)),
        vec![address(ADDR1), address(ADDR2)],
        Some(address(ADDR1)),
        "Rabby",
    );
    let expected = format!(
        "{{\"connected\":true,\"chainId\":11155111,\"accounts\":[\"{ADDR1}\",\"{ADDR2}\"],\
         \"selectedAccount\":\"{ADDR1}\",\"walletLabel\":\"Rabby\"}}"
    );

    assert_json_bytes(&session, &expected);
}

#[test]
fn wallet_event_preserves_wire_shape() {
    let event = WalletEvent::RequestFailed {
        method: "eth_requestAccounts".to_owned(),
        message: "rejected".to_owned(),
    };

    assert_json_bytes(
        &event,
        r#"{"kind":"requestFailed","method":"eth_requestAccounts","message":"rejected"}"#,
    );
}

#[test]
fn rpc_error_payload_new_preserves_wire_shape() {
    let payload = RpcErrorPayload::new(4902, "missing chain", Some(json!({ "detail": "kept" })));

    assert_json_bytes(
        &payload,
        r#"{"code":4902,"message":"missing chain","data":{"detail":"kept"}}"#,
    );
}

#[test]
fn injected_wallet_discovery_source_preserves_wire_shape() {
    assert_json_bytes(
        &InjectedWalletDiscoverySource::LegacyWindowEthereum,
        r#""legacyWindowEthereum""#,
    );
}

#[test]
fn injected_wallet_detection_options_new_preserves_wire_shape() {
    let options = InjectedWalletDetectionOptions::new(750);

    assert_json_bytes(&options, r#"{"timeoutMs":750}"#);
}

#[test]
fn injected_wallet_info_new_preserves_wire_shape() {
    let info = InjectedWalletInfo::new(
        "MetaMask",
        InjectedWalletDiscoverySource::Eip6963,
        Some("uuid-metamask".to_owned()),
        Some("io.metamask".to_owned()),
        Some("data:image/svg+xml,<svg/>".to_owned()),
        true,
        false,
        false,
    );

    assert_json_bytes(
        &info,
        r#"{"providerLabel":"MetaMask","discoverySource":"eip6963","providerUuid":"uuid-metamask","providerRdns":"io.metamask","providerIcon":"data:image/svg+xml,<svg/>","isMetaMask":true,"isCoinbaseWallet":false,"isRabby":false}"#,
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
fn wallet_chain_parameters_new_preserves_wire_shape() {
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
        r#"{"chainId":11155111,"chainName":"Sepolia","nativeCurrency":{"name":"Ether","symbol":"ETH","decimals":18},"rpcUrls":["https://rpc.sepolia.example"],"blockExplorerUrls":["https://explorer.sepolia.example"],"iconUrls":["https://cdn.example/icon.svg"]}"#,
    );
}

#[test]
fn wallet_chain_change_kind_preserves_wire_shape() {
    assert_json_bytes(
        &WalletChainChangeKind::AddedThenSwitched,
        r#""addedThenSwitched""#,
    );
}

#[test]
fn wallet_chain_change_new_preserves_wire_shape() {
    let session = WalletSession::new(
        true,
        Some(u64::from(SupportedChainId::Mainnet)),
        vec![address(ADDR1)],
        Some(address(ADDR1)),
        "MetaMask",
    );
    let change = WalletChainChange::new(
        SupportedChainId::Mainnet,
        WalletChainChangeKind::Switched,
        session,
    );
    let expected = format!(
        "{{\"requestedChainId\":1,\"kind\":\"switched\",\"session\":{{\"connected\":true,\
         \"chainId\":1,\"accounts\":[\"{ADDR1}\"],\"selectedAccount\":\"{ADDR1}\",\
         \"walletLabel\":\"MetaMask\"}}}}"
    );

    assert_json_bytes(&change, &expected);
}
