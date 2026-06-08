#![cfg(all(feature = "tracing", not(target_arch = "wasm32")))]
//! Telemetry contract for browser-wallet connection and session operations.
//!
//! `connect`, `request_accounts`, and `refresh_session` are wallet-mediated RPC
//! operations that drive `eth_requestAccounts`/`eth_accounts`/`eth_chainId`.
//! Each emits one span carrying an explicit `method` label, matching the
//! chain-management operations already covered.

use cow_sdk_browser_wallet::{BrowserWallet, MockEip1193Transport};
use cow_sdk_test_utils::trace::TraceCapture;

#[tokio::test(flavor = "current_thread")]
async fn connect_request_accounts_and_refresh_emit_method_labelled_spans() {
    let capture = TraceCapture::install();
    let transport = MockEip1193Transport::sepolia();
    let wallet = BrowserWallet::from_transport_or_panic(transport);

    wallet.connect().await.expect("mock wallet connects");
    wallet
        .request_accounts()
        .await
        .expect("mock wallet returns accounts");
    wallet
        .refresh_session()
        .await
        .expect("mock wallet refreshes the session");

    let spans = capture.spans();
    for method in [
        "browser_wallet.connect",
        "browser_wallet.request_accounts",
        "browser_wallet.refresh_session",
    ] {
        assert!(
            spans.iter().any(|span| span.field("method") == Some(method)),
            "expected exactly one span carrying method = {method}: {spans:#?}"
        );
    }
}
