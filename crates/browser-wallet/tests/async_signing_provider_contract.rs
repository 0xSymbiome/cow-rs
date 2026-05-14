//! Behaviour tests for `AsyncSigningProvider for Eip1193Provider`.
//!
//! `create_signer` is the only method on this trait and binds a returned
//! [`Eip1193Signer`] to either an explicit account hint (when the caller
//! supplied one) or the wallet's currently selected account (when the
//! hint is empty). The tests below pin every documented branch:
//!
//! - empty and whitespace-only hints produce a signer that resolves to the
//!   wallet's currently selected account through the documented fallback
//! - a valid hint that matches the wallet's exposed accounts produces a
//!   signer whose `get_address` returns that hint directly
//! - a valid hint that the wallet does not expose returns a typed
//!   `MalformedResponse` error from `create_signer` whose method label is
//!   `"create_signer"` and whose message references the documented reason
//! - a hint that fails address validation surfaces the underlying
//!   `Address::new` error directly (no `MalformedResponse` lift)
//! - when the cached session accounts are empty the implementation
//!   queries the wallet via the documented `query_accounts(false)` path
//!   before validating the hint
//!
//! Internal state is intentionally not inspected; behavioural assertions
//! go through the public `AsyncSigner::get_address` boundary, the typed
//! `BrowserWalletError` shape, and the `Redacted<String>` contents on
//! the error variants.
//!
//! [`Eip1193Signer`]: cow_sdk_browser_wallet::Eip1193Signer

#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_browser_wallet::{BrowserWallet, BrowserWalletError, MockEip1193Transport};
use cow_sdk_core::{Address, AsyncSigner, AsyncSigningProvider};

const PRIMARY_ACCOUNT: &str = "0x1111111111111111111111111111111111111111";
const OTHER_ACCOUNT: &str = "0x2222222222222222222222222222222222222222";

async fn provider_with_accounts(accounts: Vec<Address>) -> cow_sdk_browser_wallet::Eip1193Provider {
    let transport = MockEip1193Transport::sepolia();
    transport.set_connected(true);
    transport.set_accounts(accounts);
    let wallet = BrowserWallet::from_transport_or_panic(transport);
    wallet
        .connect()
        .await
        .expect("connect succeeds against the mock transport");
    wallet.provider()
}

#[tokio::test(flavor = "current_thread")]
async fn create_signer_with_empty_hint_falls_back_to_wallet_selected_account() {
    let primary = Address::new(PRIMARY_ACCOUNT).unwrap();
    let provider = provider_with_accounts(vec![primary.clone()]).await;

    let signer = provider
        .create_signer("")
        .await
        .expect("empty hint must succeed");
    // With no explicit hint, `get_address` resolves through the wallet's
    // selected account, which the mock reports as the first connected one.
    let address = signer
        .get_address()
        .await
        .expect("get_address resolves through the selected-account fallback");
    assert_eq!(address.normalized_key(), primary.normalized_key());
}

#[tokio::test(flavor = "current_thread")]
async fn create_signer_with_whitespace_hint_trims_to_empty_and_falls_back() {
    let primary = Address::new(PRIMARY_ACCOUNT).unwrap();
    let provider = provider_with_accounts(vec![primary.clone()]).await;

    let signer = provider
        .create_signer("   \t \n  ")
        .await
        .expect("whitespace-only hint must trim to empty and succeed");
    let address = signer.get_address().await.expect("fallback resolves");
    assert_eq!(address.normalized_key(), primary.normalized_key());
}

#[tokio::test(flavor = "current_thread")]
async fn create_signer_with_valid_hint_in_wallet_accounts_returns_signer_bound_to_hint() {
    let primary = Address::new(PRIMARY_ACCOUNT).unwrap();
    let other = Address::new(OTHER_ACCOUNT).unwrap();
    let provider = provider_with_accounts(vec![primary.clone(), other.clone()]).await;

    let signer = provider
        .create_signer(OTHER_ACCOUNT)
        .await
        .expect("hint in account list must succeed");

    // The signer is bound to the requested account; `get_address` returns it.
    let address = signer
        .get_address()
        .await
        .expect("get_address returns the explicit hint");
    assert_eq!(address.normalized_key(), other.normalized_key());
}

#[tokio::test(flavor = "current_thread")]
async fn create_signer_with_valid_hint_not_in_wallet_accounts_returns_malformed_response() {
    let primary = Address::new(PRIMARY_ACCOUNT).unwrap();
    let provider = provider_with_accounts(vec![primary]).await;

    let error = provider
        .create_signer(OTHER_ACCOUNT)
        .await
        .expect_err("hint outside wallet accounts must be rejected");

    match error {
        BrowserWalletError::MalformedResponse { method, message } => {
            assert_eq!(method.into_inner(), "create_signer");
            let message = message.into_inner();
            assert!(
                message.contains("wallet does not expose account"),
                "MalformedResponse must mention the documented reason; got {message:?}",
            );
        }
        other => panic!("expected MalformedResponse, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn create_signer_queries_wallet_when_session_accounts_are_empty() {
    // Simulate a session whose cached accounts are empty by NOT connecting
    // the wallet at session level. The provider's `create_signer` falls
    // through to the documented `query_accounts(false)` path and picks up
    // the freshly queried list. The mock returns the configured accounts
    // even when the session has not been connected through the SDK helpers.
    let transport = MockEip1193Transport::sepolia();
    transport.set_connected(true);
    let primary = Address::new(PRIMARY_ACCOUNT).unwrap();
    transport.set_accounts(vec![primary.clone()]);
    let wallet = BrowserWallet::from_transport_or_panic(transport);
    // No `wallet.connect().await` here — the session accounts cache is
    // empty, forcing the query path inside `create_signer`.
    let provider = wallet.provider();

    let signer = provider
        .create_signer(PRIMARY_ACCOUNT)
        .await
        .expect("query-accounts path resolves a matching account");
    let address = signer.get_address().await.expect("get_address resolves");
    assert_eq!(address.normalized_key(), primary.normalized_key());
}

#[tokio::test(flavor = "current_thread")]
async fn create_signer_with_malformed_hint_propagates_address_validation_error() {
    let provider = provider_with_accounts(vec![Address::new(PRIMARY_ACCOUNT).unwrap()]).await;

    let error = provider
        .create_signer("not-a-valid-address")
        .await
        .expect_err("malformed hint must surface the address validation error");

    // The underlying error is `cow_sdk_core::CoreError::Validation(...)` lifted
    // into `BrowserWalletError`. Any non-MalformedResponse variant is
    // acceptable as long as it carries the validation cause directly.
    assert!(
        !matches!(error, BrowserWalletError::MalformedResponse { .. }),
        "malformed hint must lift the Address::new error, not a MalformedResponse; got {error:?}",
    );
}
