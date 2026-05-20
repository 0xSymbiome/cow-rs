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
        TypedDataDomain {
            name: Some("Gnosis Protocol".into()),
            version: Some("v2".into()),
            // mismatched chain id (claims Mainnet while the signer is Sepolia)
            chain_id: Some(alloy_primitives::U256::from(1u64)),
            verifying_contract: Some(*Address::new(PRIMARY_ACCOUNT).unwrap().as_alloy()),
            salt: None,
        },
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

#[tokio::test(flavor = "current_thread")]
#[allow(
    clippy::too_many_lines,
    reason = "single end-to-end loop walks every supported chain id through the bridge and asserts the full EIP-1193 wire shape (chainId numeric, verifyingContract present and lowercase-hex, salt stripped, name/version/primaryType/types/message carried verbatim); splitting the body into helpers would scatter the wire-shape contract across multiple sites"
)]
async fn typed_data_request_emits_domain_with_numeric_chain_id_and_required_verifying_contract() {
    // Walks every supported chain id through the
    // `sign_typed_data_payload` -> `typed_data_request` path and asserts
    // the EIP-1193 wire shape the browser-wallet bridge emits to
    // `eth_signTypedData_v4`:
    //
    // - `chainId` is a JSON Number (not the alloy-default `"0x1"`
    //   hex string)
    // - `verifyingContract` is present and non-null
    // - `salt` is absent (cow construction never sets it)
    //
    // The cow `TypedDataDomain` is aliased onto
    // `alloy_sol_types::Eip712Domain`. The alloy serialise default
    // emits `chainId` as `"0x1"` and `verifyingContract` as the address
    // hex string; the bridge coercion fixes the chainId shape and
    // confirms the required field before the request leaves the SDK.

    for chain in [SupportedChainId::Mainnet, SupportedChainId::Sepolia] {
        let primary = Address::new(PRIMARY_ACCOUNT).unwrap();
        let transport = MockEip1193Transport::sepolia();
        transport.set_chain_id(chain);
        transport.set_connected(true);
        transport.set_accounts(vec![primary]);
        // The signing response just has to be a non-null string so the
        // signer surface returns Ok; the test only inspects the
        // outgoing request payload.
        transport.set_default_call_result("0x".to_owned());
        let wallet = BrowserWallet::from_transport_or_panic(transport.clone());
        wallet.connect().await.expect("connect succeeds");
        let signer = wallet
            .provider()
            .create_signer(PRIMARY_ACCOUNT)
            .await
            .expect("signer constructs against the mock");

        let mut types: BTreeMap<String, Vec<TypedDataField>> = BTreeMap::new();
        types.insert(
            "Order".to_owned(),
            vec![TypedDataField::new(
                "sellToken".to_owned(),
                "address".to_owned(),
            )],
        );
        let payload = TypedDataPayload::new(
            TypedDataDomain {
                name: Some("Gnosis Protocol".into()),
                version: Some("v2".into()),
                chain_id: Some(alloy_primitives::U256::from(u64::from(chain))),
                verifying_contract: Some(*primary.as_alloy()),
                salt: None,
            },
            "Order".to_owned(),
            types,
            "{\"sellToken\":\"0x1111111111111111111111111111111111111111\"}".to_owned(),
        );

        // The signing call returns Err when the mock cannot produce a
        // canonical signature shape; that is fine for this test — the
        // assertion runs against the outgoing request log.
        let _ = signer.sign_typed_data_payload(&payload).await;

        let log = transport.request_log();
        let signed_request = log
            .iter()
            .find(|record| record.method == "eth_signTypedData_v4")
            .expect("signer must emit one eth_signTypedData_v4 call");
        let params = signed_request
            .params
            .as_ref()
            .and_then(|value| value.as_array())
            .expect("eth_signTypedData_v4 params must be an array");
        let typed_data_string = params
            .get(1)
            .and_then(|value| value.as_str())
            .expect("eth_signTypedData_v4 params[1] must be the typed-data JSON string");
        let typed_data: serde_json::Value =
            serde_json::from_str(typed_data_string).expect("typed-data string must parse as JSON");
        let domain = typed_data
            .get("domain")
            .and_then(|value| value.as_object())
            .expect("typed-data request must carry a JSON object domain");

        let chain_id = domain
            .get("chainId")
            .expect("EIP-1193 typed-data domain must carry chainId");
        assert!(
            chain_id.is_number(),
            "chainId must be a JSON Number (the EIP-1193 wire shape), got {chain_id:?}",
        );
        assert_eq!(
            chain_id.as_u64(),
            Some(u64::from(chain)),
            "chainId must match the cow-bound chain id verbatim",
        );

        let verifying_contract = domain
            .get("verifyingContract")
            .expect("EIP-1193 typed-data domain must carry verifyingContract");
        assert!(
            !verifying_contract.is_null(),
            "verifyingContract must not be null on the wire (cow construction always sets it)",
        );
        let verifying_contract_str = verifying_contract
            .as_str()
            .expect("verifyingContract must be a 0x-prefixed lowercase hex string on the wire");
        assert_eq!(
            verifying_contract_str.len(),
            42,
            "verifyingContract must be a 0x-prefixed 42-char hex string, got `{verifying_contract_str}`",
        );
        assert!(
            verifying_contract_str
                .strip_prefix("0x")
                .is_some_and(|tail| tail.chars().all(|c| c.is_ascii_hexdigit())),
            "verifyingContract must be a 0x-prefixed lowercase hex string, got `{verifying_contract_str}`",
        );

        assert!(
            !domain.contains_key("salt"),
            "the bridge coercion must strip the alloy `salt` field; cow never sets it",
        );

        // Carry-through invariants: the bridge coercion only touches the
        // domain; `name`, `version`, `primaryType`, the `types` map and
        // the `message` body all flow through `serde_json::to_value` /
        // `from_str` untouched. Pin the canonical cow shape so a future
        // regression in any of those code paths surfaces here.
        assert_eq!(
            domain.get("name"),
            Some(&serde_json::json!("Gnosis Protocol")),
            "domain.name must be carried verbatim from the cow payload",
        );
        assert_eq!(
            domain.get("version"),
            Some(&serde_json::json!("v2")),
            "domain.version must be carried verbatim from the cow payload",
        );
        assert_eq!(
            typed_data.get("primaryType"),
            Some(&serde_json::json!("Order")),
            "typed-data primaryType must be carried verbatim from the cow payload",
        );
        let types = typed_data
            .get("types")
            .and_then(|value| value.as_object())
            .expect("typed-data request must carry a JSON object types map");
        assert!(
            types.contains_key("Order"),
            "types map must carry the cow primary-type entry",
        );
        let message = typed_data
            .get("message")
            .and_then(|value| value.as_object())
            .expect("typed-data request must carry a JSON object message body");
        assert_eq!(
            message.get("sellToken"),
            Some(&serde_json::json!(
                "0x1111111111111111111111111111111111111111"
            )),
            "message body must be carried verbatim from the cow payload",
        );
    }
}
