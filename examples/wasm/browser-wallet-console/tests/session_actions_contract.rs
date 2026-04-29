use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use cow_sdk::browser_wallet::{BrowserWallet, MockEip1193Transport};
use cow_sdk_browser_wallet_console::BrowserWalletConsole;
use serde_json::Value;

#[test]
fn reset_session_retains_selected_wallet_and_clears_console_state() {
    let console = BrowserWalletConsole::new();
    let wallet = seeded_wallet();
    console.testing_set_injected_wallet(wallet);
    console.testing_set_last_live_order_uid(Some("0xfeedbeef".to_owned()));

    let reset = parse_json(
        console
            .injected_reset_session_json()
            .expect("reset must succeed"),
    );

    assert_eq!(reset["mode"], "injected");
    assert_eq!(reset["walletSelectionRetained"], true);
    assert_eq!(reset["session"]["connected"], false);
    assert_eq!(reset["session"]["accounts"], Value::Null);
    assert_eq!(reset["session"]["selectedAccount"], Value::Null);
    assert_eq!(console.last_live_order_uid(), None);

    let status = parse_json(
        console
            .injected_status_json()
            .expect("status must succeed after reset"),
    );
    assert_eq!(status["session"]["connected"], false);
}

#[test]
fn refresh_rehydrates_session_after_reset_without_reselecting_wallet() {
    let console = BrowserWalletConsole::new();
    let wallet = seeded_wallet();
    console.testing_set_injected_wallet(wallet);

    console
        .injected_reset_session_json()
        .expect("reset must succeed");
    let refreshed =
        parse_json(block_on(console.injected_refresh_json()).expect("refresh must succeed"));

    assert_eq!(refreshed["mode"], "injected");
    assert_eq!(refreshed["session"]["connected"], true);
    assert_eq!(
        refreshed["session"]["selectedAccount"],
        "0x4444444444444444444444444444444444444444"
    );
}

#[test]
fn forget_wallet_clears_selection_and_post_forget_actions_fail_closed() {
    let console = BrowserWalletConsole::new();
    let wallet = seeded_wallet();
    console.testing_set_injected_wallet(wallet);
    console.testing_set_last_live_order_uid(Some("0xfeedbeef".to_owned()));

    let forget = parse_json(
        console
            .injected_forget_wallet_json()
            .expect("forget must succeed"),
    );

    assert_eq!(forget["mode"], "injected");
    assert_eq!(forget["walletSelectionCleared"], true);
    assert_eq!(forget["lastLiveOrderUidCleared"], true);
    assert_eq!(console.last_live_order_uid(), None);
    assert!(!console.testing_has_injected_wallet());
}

fn seeded_wallet() -> BrowserWallet {
    let transport = MockEip1193Transport::sepolia();
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
