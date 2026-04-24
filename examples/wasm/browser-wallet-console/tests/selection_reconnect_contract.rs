use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use cow_sdk::browser_wallet::{
    BrowserWallet, InjectedWalletDiscoverySource, InjectedWalletInfo, MockEip1193Transport,
};
use cow_sdk_browser_wallet_console_wasm::BrowserWalletConsole;
use serde_json::Value;

#[test]
fn legacy_only_cached_detection_connects_without_rescan() {
    let console = BrowserWalletConsole::new();
    console.testing_set_cached_injected_wallets(
        vec![cached_wallet(
            "MetaMask",
            InjectedWalletDiscoverySource::LegacyWindowEthereum,
        )],
        250,
        true,
    );

    let detection = parse_json(console.testing_cached_detection_json());
    assert_eq!(detection["walletCount"], 1);
    assert_eq!(detection["usedWindowEthereumFallback"], true);

    let connected =
        parse_json(block_on(console.injected_connect_json()).expect("connect must succeed"));
    assert_eq!(connected["connectionSource"], "cachedDetection");
    assert_eq!(connected["selectionIndex"], 0);
    assert_eq!(
        connected["walletInfo"]["discoverySource"],
        "legacyWindowEthereum"
    );
    assert_eq!(console.testing_selected_wallet_index(), Some(0));
}

#[test]
fn multi_wallet_reconnect_reuses_selected_wallet_handle() {
    let console = BrowserWalletConsole::new();
    console.testing_set_cached_injected_wallets(
        vec![
            cached_wallet("MetaMask", InjectedWalletDiscoverySource::Eip6963),
            cached_wallet("Rabby", InjectedWalletDiscoverySource::Eip6963),
        ],
        750,
        false,
    );

    let first = parse_json(
        block_on(console.injected_connect_selected_json(1)).expect("initial connect must succeed"),
    );
    assert_eq!(first["connectionSource"], "cachedDetection");
    assert_eq!(first["walletInfo"]["providerLabel"], "Rabby");

    console
        .injected_reset_session_json()
        .expect("reset must preserve the selected wallet");
    let reconnect = parse_json(
        block_on(console.injected_connect_selected_json(1)).expect("reconnect must succeed"),
    );
    assert_eq!(reconnect["connectionSource"], "selectedWallet");
    assert_eq!(reconnect["walletInfo"]["providerLabel"], "Rabby");
    assert_eq!(console.testing_selected_wallet_index(), Some(1));
}

#[test]
fn explicit_rescan_replaces_cached_candidates_for_next_selection() {
    let console = BrowserWalletConsole::new();
    console.testing_set_cached_injected_wallets(
        vec![cached_wallet(
            "MetaMask",
            InjectedWalletDiscoverySource::LegacyWindowEthereum,
        )],
        250,
        true,
    );
    let _ = block_on(console.injected_connect_json()).expect("initial connect must succeed");

    console.testing_set_cached_injected_wallets(
        vec![
            cached_wallet("Frame", InjectedWalletDiscoverySource::Eip6963),
            cached_wallet("Rabby", InjectedWalletDiscoverySource::Eip6963),
        ],
        750,
        false,
    );

    let detection = parse_json(console.testing_cached_detection_json());
    assert_eq!(detection["walletCount"], 2);
    assert_eq!(detection["selectedWalletPresent"], true);
    assert_eq!(detection["selectedIndex"], 0);

    let connected = parse_json(
        block_on(console.injected_connect_selected_json(1))
            .expect("connect after rescan must succeed"),
    );
    assert_eq!(connected["connectionSource"], "cachedDetection");
    assert_eq!(connected["walletInfo"]["providerLabel"], "Rabby");
    assert_eq!(console.testing_selected_wallet_index(), Some(1));
}

#[test]
fn cached_eip6963_detection_supports_one_shot_connect_flow() {
    let console = BrowserWalletConsole::new();
    console.testing_set_cached_injected_wallets(
        vec![cached_wallet(
            "Rabby",
            InjectedWalletDiscoverySource::Eip6963,
        )],
        750,
        false,
    );

    let detection = parse_json(console.testing_cached_detection_json());
    assert_eq!(detection["walletCount"], 1);
    assert_eq!(detection["usedWindowEthereumFallback"], false);

    let connected = parse_json(
        block_on(console.injected_connect_selected_json(0)).expect("cached connect must succeed"),
    );
    assert_eq!(connected["connectionSource"], "cachedDetection");
    assert_eq!(connected["walletInfo"]["providerLabel"], "Rabby");
}

fn cached_wallet(
    label: &str,
    discovery_source: InjectedWalletDiscoverySource,
) -> (BrowserWallet, Option<InjectedWalletInfo>) {
    (
        seeded_wallet(label),
        Some(InjectedWalletInfo::new(
            label.to_owned(),
            discovery_source,
            None,
            None,
            None,
            label == "MetaMask",
            label == "Coinbase Wallet",
            label == "Rabby",
        )),
    )
}

fn seeded_wallet(label: &str) -> BrowserWallet {
    let transport = MockEip1193Transport::sepolia().with_label(label);
    transport.set_connected(true);
    let wallet = BrowserWallet::from_transport(transport);
    let _ = block_on(wallet.refresh_session()).expect("mock wallet refresh must succeed");
    let _ = wallet.take_events();
    wallet
}

fn parse_json(payload: String) -> Value {
    serde_json::from_str(&payload).expect("console JSON output must remain valid")
}

fn block_on<F>(future: F) -> F::Output
where
    F: Future,
{
    let waker = noop_waker();
    let mut future = Box::pin(future);
    let mut context = Context::from_waker(&waker);
    loop {
        match Pin::as_mut(&mut future).poll(&mut context) {
            Poll::Ready(output) => return output,
            Poll::Pending => std::thread::yield_now(),
        }
    }
}

fn noop_waker() -> Waker {
    // These futures resolve immediately in host-mode contract tests.
    unsafe { Waker::from_raw(noop_raw_waker()) }
}

fn noop_raw_waker() -> RawWaker {
    RawWaker::new(std::ptr::null(), &NOOP_WAKER_VTABLE)
}

unsafe fn noop_clone(_: *const ()) -> RawWaker {
    noop_raw_waker()
}

unsafe fn noop(_: *const ()) {}

static NOOP_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(noop_clone, noop, noop, noop);
