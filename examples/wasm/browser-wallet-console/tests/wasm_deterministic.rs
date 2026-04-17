use cow_sdk::browser_wallet::{
    BrowserWallet, InjectedWalletDiscoverySource, InjectedWalletInfo, MockEip1193Transport,
};
use cow_sdk_browser_wallet_console_wasm::BrowserWalletConsole;
use serde_json::Value;
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

wasm_bindgen_test_configure!(run_in_browser);

const SEPOLIA_CHAIN_ID: u32 = 11_155_111;

#[wasm_bindgen_test]
fn sample_trade_json_returns_valid_json_with_expected_top_level_keys() {
    let console = BrowserWalletConsole::new();
    let sample = parse_json_result(console.sample_trade_json(SEPOLIA_CHAIN_ID));
    assert!(sample.is_object(), "sample trade must be a JSON object");
    for key in ["kind", "sellToken", "buyToken", "amount", "slippageBps"] {
        assert!(
            sample.get(key).is_some(),
            "sample trade must expose `{key}`"
        );
    }
}

#[wasm_bindgen_test]
fn sample_order_json_returns_valid_json_with_expected_top_level_keys() {
    let console = BrowserWalletConsole::new();
    let sample = parse_json_result(console.sample_order_json(SEPOLIA_CHAIN_ID));
    assert!(sample.is_object(), "sample order must be a JSON object");
    for key in [
        "sellToken",
        "buyToken",
        "sellAmount",
        "buyAmount",
        "kind",
        "receiver",
        "validTo",
        "appData",
    ] {
        assert!(
            sample.get(key).is_some(),
            "sample order must expose `{key}`"
        );
    }
}

#[wasm_bindgen_test]
fn sample_approval_json_returns_valid_json_with_expected_top_level_keys() {
    let console = BrowserWalletConsole::new();
    let sample = parse_json_result(console.sample_approval_json(SEPOLIA_CHAIN_ID));
    assert!(sample.is_object(), "sample approval must be a JSON object");
    for key in ["tokenAddress", "amount"] {
        assert!(
            sample.get(key).is_some(),
            "sample approval must expose `{key}`"
        );
    }
}

#[wasm_bindgen_test]
async fn selection_confirmation_sequence_runs_end_to_end_under_mock_transport() {
    let console = BrowserWalletConsole::new();
    let metamask = cached_wallet("MetaMask", InjectedWalletDiscoverySource::Eip6963).await;
    let rabby = cached_wallet("Rabby", InjectedWalletDiscoverySource::Eip6963).await;
    console.testing_set_cached_injected_wallets(vec![metamask, rabby], 500, false);

    let detection = parse_json(console.testing_cached_detection_json());
    assert_eq!(detection["walletCount"], 2);
    assert_eq!(detection["requiresExplicitSelection"], true);
    assert_eq!(detection["confirmedSelectionPresent"], false);
    assert_eq!(detection["connectReady"], false);

    let confirmed = parse_json(
        console
            .testing_confirm_injected_selection_json(1)
            .expect("confirmation must succeed"),
    );
    assert_eq!(confirmed["confirmedSelectionPresent"], true);
    assert_eq!(confirmed["confirmedSelectionIndex"], 1);
    assert_eq!(confirmed["confirmedWalletInfo"]["providerLabel"], "Rabby");
    assert_eq!(confirmed["connectReady"], true);

    let connected = parse_json(
        console
            .testing_injected_connect_json()
            .await
            .expect("connect must succeed over the wasm bridge"),
    );
    assert_eq!(connected["walletInfo"]["providerLabel"], "Rabby");
    assert_eq!(connected["selectionIndex"], 1);
    assert!(
        matches!(
            connected["connectionSource"].as_str(),
            Some("cachedDetection" | "selectedWallet")
        ),
        "connect source must be a reviewed injected-connect source"
    );
}

async fn cached_wallet(
    label: &str,
    discovery_source: InjectedWalletDiscoverySource,
) -> (BrowserWallet, Option<InjectedWalletInfo>) {
    (
        seeded_wallet(label).await,
        Some(InjectedWalletInfo {
            provider_label: label.to_owned(),
            discovery_source,
            provider_uuid: Some(format!("uuid-{label}")),
            provider_rdns: Some(format!(
                "{}.wallet",
                label.to_ascii_lowercase().replace(' ', "-")
            )),
            provider_icon: None,
            is_meta_mask: label == "MetaMask",
            is_coinbase_wallet: label == "Coinbase Wallet",
            is_rabby: label == "Rabby",
        }),
    )
}

async fn seeded_wallet(label: &str) -> BrowserWallet {
    let transport = MockEip1193Transport::sepolia().with_label(label);
    transport.set_connected(true);
    let wallet = BrowserWallet::from_transport(transport);
    wallet
        .refresh_session()
        .await
        .expect("mock wallet refresh must succeed under the wasm bridge");
    let _ = wallet.take_events();
    wallet
}

fn parse_json_result(
    result: Result<String, wasm_bindgen::JsValue>,
) -> Value {
    let payload = result.expect("wasm-bindgen export must return a JSON string");
    parse_json(payload)
}

fn parse_json(payload: String) -> Value {
    serde_json::from_str(&payload).expect("console JSON output must remain valid")
}
