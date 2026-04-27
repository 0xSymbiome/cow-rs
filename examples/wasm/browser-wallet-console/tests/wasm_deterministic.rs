use cow_sdk::browser_wallet::{
    BrowserWallet, BrowserWalletError, Eip1193ProviderBuilder, InjectedWalletDiscoverySource,
    InjectedWalletInfo, MockEip1193Transport, Origin,
};
use cow_sdk_browser_wallet_console::BrowserWalletConsole;
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
fn eip1193_trust_wrapper_rejects_anonymous_provider_and_accepts_explicit_origin() {
    let error = Eip1193ProviderBuilder::new(MockEip1193Transport::sepolia())
        .build()
        .expect_err("anonymous EIP-1193 providers must require an explicit origin");

    assert!(matches!(
        error,
        BrowserWalletError::UntrustedProviderOrigin { .. }
    ));
    let rendered = error.to_string();
    assert!(rendered.contains("[redacted]"));
    assert!(!rendered.contains("sepolia"));

    let provider = Eip1193ProviderBuilder::new(MockEip1193Transport::sepolia())
        .with_trusted_origin(Origin::new("test://browser-wallet-console/smoke").unwrap())
        .build()
        .expect("explicitly trusted EIP-1193 provider must build");

    assert_eq!(
        provider.origin().map(Origin::as_str),
        Some("test://browser-wallet-console/smoke")
    );
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
        Some(InjectedWalletInfo::new(
            label.to_owned(),
            discovery_source,
            Some(format!("uuid-{label}")),
            Some(format!(
                "{}.wallet",
                label.to_ascii_lowercase().replace(' ', "-")
            )),
            None,
            label == "MetaMask",
            label == "Coinbase Wallet",
            label == "Rabby",
        )),
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

fn parse_json_result(result: Result<String, wasm_bindgen::JsValue>) -> Value {
    let payload = result.expect("wasm-bindgen export must return a JSON string");
    parse_json(payload)
}

fn parse_json(payload: String) -> Value {
    serde_json::from_str(&payload).expect("console JSON output must remain valid")
}
