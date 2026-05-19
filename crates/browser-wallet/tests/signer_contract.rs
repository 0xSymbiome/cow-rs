//! Behaviour tests for `Eip1193Signer` chain-binding and account fallback paths.
//!
//! The signer's happy paths are covered through the broader wallet and
//! state-machine integration tests. This file complements that by pinning:
//!
//! - `with_expected_chain` / `expected_chain_id` round trip
//! - `ensure_expected_chain` rejection when the wallet session reports a
//!   different chain id than the signer was bound to
//! - `validate_typed_data_chain` rejection when the typed-data payload's
//!   domain chain id differs from the signer's expected chain id
//! - the `account()` fallback that returns a `MalformedResponse` carrying
//!   the documented "wallet does not currently expose any account" message
//!   when the wallet's exposed account list is empty
//!
//! Every test drives the public `AsyncSigner` boundary; no private signer
//! state is inspected.

#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeMap;

use cow_sdk_browser_wallet::{BrowserWallet, BrowserWalletError, MockEip1193Transport};
use cow_sdk_core::{
    Address, AsyncSigner, AsyncSigningProvider, SupportedChainId, TypedDataDomain, TypedDataField,
    TypedDataPayload,
};

const PRIMARY_ACCOUNT: &str = "0x1111111111111111111111111111111111111111";

async fn signer_for_chain(chain: SupportedChainId) -> cow_sdk_browser_wallet::Eip1193Signer {
    let transport = MockEip1193Transport::sepolia();
    transport.set_chain_id(chain);
    transport.set_connected(true);
    transport.set_accounts(vec![Address::new(PRIMARY_ACCOUNT).unwrap()]);
    let wallet = BrowserWallet::from_transport_or_panic(transport);
    wallet.connect().await.expect("connect succeeds");
    wallet
        .provider()
        .create_signer("")
        .await
        .expect("signer constructs against the mock")
}

#[tokio::test(flavor = "current_thread")]
async fn with_expected_chain_round_trips_through_expected_chain_id_accessor() {
    let signer = signer_for_chain(SupportedChainId::Sepolia).await;
    assert_eq!(signer.expected_chain_id(), None);

    let bound = signer.with_expected_chain(SupportedChainId::Mainnet);
    assert_eq!(bound.expected_chain_id(), Some(SupportedChainId::Mainnet));

    // The setter is `const` and returns a new signer; the original is gone
    // by move-semantics here, so we only verify the bound form.
}

#[tokio::test(flavor = "current_thread")]
async fn ensure_expected_chain_rejects_session_chain_mismatch_on_get_address() {
    // Session chain is Sepolia (configured by the mock), but the signer is
    // bound to Mainnet. Any signing method must reject before dispatching.
    let signer = signer_for_chain(SupportedChainId::Sepolia)
        .await
        .with_expected_chain(SupportedChainId::Mainnet);

    let error = signer
        .get_address()
        .await
        .expect_err("chain mismatch must reject before the call dispatches");

    match error {
        BrowserWalletError::SessionChainMismatch {
            expected_chain_id,
            session_chain_id,
        } => {
            assert_eq!(expected_chain_id, u64::from(SupportedChainId::Mainnet));
            assert_eq!(session_chain_id, u64::from(SupportedChainId::Sepolia));
        }
        other => panic!("expected SessionChainMismatch, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn ensure_expected_chain_accepts_matching_session_chain_id() {
    let signer = signer_for_chain(SupportedChainId::Sepolia)
        .await
        .with_expected_chain(SupportedChainId::Sepolia);

    // Matching chain ids must not block the call; get_address succeeds.
    let address = signer
        .get_address()
        .await
        .expect("matching chain id must pass through to the wallet");
    assert_eq!(
        address.normalized_key(),
        PRIMARY_ACCOUNT.to_ascii_lowercase()
    );
}

#[tokio::test(flavor = "current_thread")]
async fn validate_typed_data_chain_rejects_payload_with_wrong_domain_chain_id() {
    let signer = signer_for_chain(SupportedChainId::Sepolia)
        .await
        .with_expected_chain(SupportedChainId::Sepolia);

    // Payload claims Mainnet (1) but the signer is bound to Sepolia (11155111).
    let mut types: BTreeMap<String, Vec<TypedDataField>> = BTreeMap::new();
    types.insert("Order".to_owned(), vec![]);
    let payload = TypedDataPayload::new(
        TypedDataDomain::new(
            "CoW Protocol".to_owned(),
            "v2".to_owned(),
            1, // mismatched chain id
            Address::new(PRIMARY_ACCOUNT).unwrap(),
        ),
        "Order".to_owned(),
        types,
        "{}".to_owned(),
    );

    let error = signer
        .sign_typed_data_payload(&payload)
        .await
        .expect_err("typed-data domain chain mismatch must be rejected");

    match error {
        BrowserWalletError::TypedDataChainMismatch {
            expected_chain_id,
            typed_data_chain_id,
        } => {
            assert_eq!(expected_chain_id, u64::from(SupportedChainId::Sepolia));
            assert_eq!(typed_data_chain_id, 1);
        }
        other => panic!("expected TypedDataChainMismatch, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn account_returns_malformed_response_when_wallet_exposes_no_account() {
    // Disconnect the mock so it reports an empty account list. The signer's
    // account fallback then surfaces the documented MalformedResponse.
    let transport = MockEip1193Transport::sepolia();
    transport.set_connected(false);
    transport.set_accounts(vec![]);
    let wallet = BrowserWallet::from_transport_or_panic(transport);
    let provider = wallet.provider();

    // Build a signer with no hint so `account()` falls through to the
    // selected_account + query_accounts path, both of which yield empty.
    let signer = provider
        .create_signer("")
        .await
        .expect("signer constructs even when wallet is disconnected");

    let error = signer
        .get_address()
        .await
        .expect_err("disconnected wallet must surface a MalformedResponse on account()");

    match error {
        BrowserWalletError::MalformedResponse { method, message } => {
            assert_eq!(method.into_inner(), "eth_accounts");
            let rendered = message.into_inner();
            assert!(
                rendered.contains("does not currently expose any account"),
                "MalformedResponse must mention the documented reason; got {rendered:?}",
            );
        }
        other => panic!("expected MalformedResponse, got {other:?}"),
    }
}

#[tokio::test(flavor = "current_thread")]
async fn account_returns_hint_when_signer_was_constructed_with_explicit_account() {
    let primary = Address::new(PRIMARY_ACCOUNT).unwrap();
    let transport = MockEip1193Transport::sepolia();
    transport.set_connected(true);
    transport.set_accounts(vec![primary]);
    let wallet = BrowserWallet::from_transport_or_panic(transport);
    wallet.connect().await.expect("connect succeeds");
    let signer = wallet
        .provider()
        .create_signer(PRIMARY_ACCOUNT)
        .await
        .expect("signer accepts a valid hint");

    // The hint short-circuits any selected_account/query_accounts dispatch.
    let address = signer
        .get_address()
        .await
        .expect("hint resolves through the account() fast path");
    assert_eq!(address.normalized_key(), primary.normalized_key());
}
