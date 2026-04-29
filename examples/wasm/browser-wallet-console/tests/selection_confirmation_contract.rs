use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use cow_sdk::browser_wallet::{
    BrowserWallet, InjectedWalletDiscoverySource, InjectedWalletInfo, MockEip1193Transport,
};
use cow_sdk_browser_wallet_console::BrowserWalletConsole;
use serde_json::Value;

#[test]
fn multi_wallet_connect_fails_closed_until_confirmation() {
    let console = BrowserWalletConsole::new();
    console.testing_set_cached_injected_wallets(
        vec![
            cached_wallet("MetaMask", InjectedWalletDiscoverySource::Eip6963),
            cached_wallet("Phantom", InjectedWalletDiscoverySource::Eip6963),
        ],
        500,
        false,
    );

    let detection = parse_json(console.testing_cached_detection_json());
    assert_eq!(detection["walletCount"], 2);
    assert_eq!(detection["requiresExplicitSelection"], true);
    assert_eq!(detection["confirmedSelectionPresent"], false);
    assert_eq!(detection["connectReady"], false);

    let error =
        block_on(console.testing_injected_connect_json()).expect_err("connect must fail closed");
    assert!(error.contains("confirm a detected wallet before connecting"));
    assert_eq!(console.testing_confirmed_wallet_index(), None);
}

#[test]
fn confirmed_selection_enables_connect_and_tracks_provider() {
    let console = BrowserWalletConsole::new();
    console.testing_set_cached_injected_wallets(
        vec![
            cached_wallet("MetaMask", InjectedWalletDiscoverySource::Eip6963),
            cached_wallet("Rabby", InjectedWalletDiscoverySource::Eip6963),
        ],
        750,
        false,
    );

    let confirmed = parse_json(
        console
            .testing_confirm_injected_selection_json(1)
            .expect("confirmation must succeed"),
    );
    assert_eq!(confirmed["confirmedSelectionPresent"], true);
    assert_eq!(confirmed["confirmedSelectionIndex"], 1);
    assert_eq!(confirmed["confirmedWalletInfo"]["providerLabel"], "Rabby");
    assert_eq!(confirmed["connectReady"], true);
    assert_eq!(console.testing_confirmed_wallet_index(), Some(1));

    let connected = parse_json(
        block_on(console.testing_injected_connect_json()).expect("connect must succeed"),
    );
    assert_eq!(connected["connectionSource"], "cachedDetection");
    assert_eq!(connected["walletInfo"]["providerLabel"], "Rabby");
    assert_eq!(connected["selectionIndex"], 1);
    assert_eq!(console.testing_selected_wallet_index(), Some(1));
}

#[test]
fn reconnect_after_confirmation_uses_retained_wallet_handle() {
    let console = BrowserWalletConsole::new();
    console.testing_set_cached_injected_wallets(
        vec![
            cached_wallet("Phantom", InjectedWalletDiscoverySource::Eip6963),
            cached_wallet("MetaMask", InjectedWalletDiscoverySource::Eip6963),
        ],
        500,
        false,
    );

    console
        .testing_confirm_injected_selection_json(1)
        .expect("confirmation must succeed");
    let _ =
        block_on(console.testing_injected_connect_json()).expect("initial connect must succeed");

    console
        .injected_reset_session_json()
        .expect("reset must keep the retained wallet and confirmation");
    let reconnect = parse_json(
        block_on(console.testing_injected_connect_json()).expect("reconnect must succeed"),
    );

    assert_eq!(reconnect["connectionSource"], "selectedWallet");
    assert_eq!(reconnect["walletInfo"]["providerLabel"], "MetaMask");

    let status = parse_json(console.injected_status_json().expect("status must succeed"));
    assert_eq!(status["confirmedSelectionPresent"], true);
    assert_eq!(status["confirmedSelectionIndex"], 1);
    assert_eq!(status["confirmedWalletInfo"]["providerLabel"], "MetaMask");
}

#[test]
fn rescan_revalidates_then_clears_confirmed_selection() {
    let console = BrowserWalletConsole::new();
    console.testing_set_cached_injected_wallets(
        vec![
            cached_wallet("Frame", InjectedWalletDiscoverySource::Eip6963),
            cached_wallet("Rabby", InjectedWalletDiscoverySource::Eip6963),
        ],
        500,
        false,
    );

    console
        .testing_confirm_injected_selection_json(1)
        .expect("initial confirmation must succeed");

    console.testing_set_cached_injected_wallets(
        vec![
            cached_wallet("Rabby", InjectedWalletDiscoverySource::Eip6963),
            cached_wallet("MetaMask", InjectedWalletDiscoverySource::Eip6963),
        ],
        500,
        false,
    );

    let revalidated = parse_json(console.testing_cached_detection_json());
    assert_eq!(revalidated["confirmedSelectionPresent"], true);
    assert_eq!(revalidated["confirmedSelectionIndex"], 0);
    assert_eq!(revalidated["confirmedWalletInfo"]["providerLabel"], "Rabby");
    assert_eq!(revalidated["connectReady"], true);
    assert_eq!(console.testing_confirmed_wallet_index(), Some(0));

    console.testing_set_cached_injected_wallets(
        vec![
            cached_wallet("MetaMask", InjectedWalletDiscoverySource::Eip6963),
            cached_wallet("Brave Wallet", InjectedWalletDiscoverySource::Eip6963),
        ],
        500,
        false,
    );

    let cleared = parse_json(console.testing_cached_detection_json());
    assert_eq!(cleared["confirmedSelectionPresent"], false);
    assert_eq!(cleared["confirmedSelectionIndex"], Value::Null);
    assert_eq!(cleared["connectReady"], false);
    assert_eq!(console.testing_confirmed_wallet_index(), None);
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

fn seeded_wallet(label: &str) -> BrowserWallet {
    let transport = MockEip1193Transport::sepolia().with_label(label);
    transport.set_connected(true);
    let wallet = BrowserWallet::from_transport_or_panic(transport);
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
