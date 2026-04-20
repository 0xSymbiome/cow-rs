use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
};

use cow_sdk_browser_wallet_console_wasm::BrowserWalletConsole;
use serde_json::Value;

#[test]
fn walkthrough_mock_cycle_drives_connect_sign_and_trading_flow_in_order() {
    let console = BrowserWalletConsole::new();
    let envelope = parse_json(
        block_on(console.walkthrough_mock_cycle_json())
            .expect("walkthrough must succeed over the host-side mock transport"),
    );

    assert_eq!(envelope["name"], "browser-wallet-console.mock-cycle");
    assert_eq!(envelope["completed"], true);
    assert!(
        envelope["failedAt"].is_null(),
        "deterministic walkthrough must never fail"
    );

    let steps = envelope["steps"]
        .as_array()
        .expect("walkthrough envelope must expose a steps array");

    let names: Vec<&str> = steps
        .iter()
        .map(|step| step["name"].as_str().expect("step name must be a string"))
        .collect();

    assert_eq!(
        names,
        vec!["mock-connect", "mock-sign-message", "mock-trading-flow"],
        "walkthrough step order must stay stable"
    );

    for step in steps {
        let result = &step["result"];
        assert!(
            result.is_object(),
            "each walkthrough step must carry a reviewable result object"
        );
        assert_eq!(
            result["mode"], "mock",
            "walkthrough must never leave the deterministic mock lane"
        );
    }
}

fn parse_json(payload: String) -> Value {
    serde_json::from_str(&payload).expect("walkthrough JSON must parse")
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
