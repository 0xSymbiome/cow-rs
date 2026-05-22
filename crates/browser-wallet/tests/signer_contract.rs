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

/// Settlement contract address shipped on every supported chain.
const SETTLEMENT_CONTRACT: &str = "0x9008D19f58AAbD9eD0D60971565AA8510560ab41";

const CANONICAL_DOMAIN_NAME: &str = "Gnosis Protocol";
const CANONICAL_DOMAIN_VERSION: &str = "v2";

/// Canonical EIP-712 type list for `Order` (matches `GPv2Order.sol`).
fn order_types() -> BTreeMap<String, Vec<TypedDataField>> {
    let mut types = BTreeMap::new();
    types.insert("EIP712Domain".to_owned(), eip712_domain_type_fields());
    types.insert(
        "Order".to_owned(),
        vec![
            TypedDataField::new("sellToken".to_owned(), "address".to_owned()),
            TypedDataField::new("buyToken".to_owned(), "address".to_owned()),
            TypedDataField::new("receiver".to_owned(), "address".to_owned()),
            TypedDataField::new("sellAmount".to_owned(), "uint256".to_owned()),
            TypedDataField::new("buyAmount".to_owned(), "uint256".to_owned()),
            TypedDataField::new("validTo".to_owned(), "uint32".to_owned()),
            TypedDataField::new("appData".to_owned(), "bytes32".to_owned()),
            TypedDataField::new("feeAmount".to_owned(), "uint256".to_owned()),
            TypedDataField::new("kind".to_owned(), "string".to_owned()),
            TypedDataField::new("partiallyFillable".to_owned(), "bool".to_owned()),
            TypedDataField::new("sellTokenBalance".to_owned(), "string".to_owned()),
            TypedDataField::new("buyTokenBalance".to_owned(), "string".to_owned()),
        ],
    );
    types
}

/// Canonical EIP-712 type list for `OrderCancellations`.
fn cancellations_types() -> BTreeMap<String, Vec<TypedDataField>> {
    let mut types = BTreeMap::new();
    types.insert("EIP712Domain".to_owned(), eip712_domain_type_fields());
    types.insert(
        "OrderCancellations".to_owned(),
        vec![TypedDataField::new(
            "orderUids".to_owned(),
            "bytes[]".to_owned(),
        )],
    );
    types
}

fn eip712_domain_type_fields() -> Vec<TypedDataField> {
    vec![
        TypedDataField::new("name".to_owned(), "string".to_owned()),
        TypedDataField::new("version".to_owned(), "string".to_owned()),
        TypedDataField::new("chainId".to_owned(), "uint256".to_owned()),
        TypedDataField::new("verifyingContract".to_owned(), "address".to_owned()),
    ]
}

/// Canonical Order message body used across every fixture row.
const ORDER_MESSAGE_JSON: &str = r#"{"sellToken":"0x1111111111111111111111111111111111111111","buyToken":"0x2222222222222222222222222222222222222222","receiver":"0x3333333333333333333333333333333333333333","sellAmount":"1000000","buyAmount":"1000000000000000000","validTo":1700000000,"appData":"0xb48d38f93eaa084033fc5970bf96e559c33c4cdc07d889ab00b4d63f9590739d","feeAmount":"0","kind":"sell","partiallyFillable":false,"sellTokenBalance":"erc20","buyTokenBalance":"erc20"}"#;

/// Canonical `OrderCancellations` message body used across every fixture row.
const CANCELLATIONS_MESSAGE_JSON: &str = r#"{"orderUids":["0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa11111111"]}"#;

/// The six supported chains the `eth_signTypedData_v4` fixture covers.
const FIXTURE_CHAINS: &[(SupportedChainId, &str)] = &[
    (SupportedChainId::Mainnet, "mainnet"),
    (SupportedChainId::GnosisChain, "gnosis"),
    (SupportedChainId::Polygon, "polygon"),
    (SupportedChainId::Base, "base"),
    (SupportedChainId::ArbitrumOne, "arbitrum-one"),
    (SupportedChainId::Sepolia, "sepolia"),
];

/// Builds the canonical `TypedDataPayload` for one `(chain, primary_type)` row.
fn fixture_payload(chain: SupportedChainId, primary_type: &str) -> TypedDataPayload {
    let domain = TypedDataDomain::new(
        CANONICAL_DOMAIN_NAME.to_owned(),
        CANONICAL_DOMAIN_VERSION.to_owned(),
        u64::from(chain),
        Address::new(SETTLEMENT_CONTRACT).unwrap(),
    );
    let (types, message_json) = match primary_type {
        "Order" => (order_types(), ORDER_MESSAGE_JSON.to_owned()),
        "OrderCancellations" => (cancellations_types(), CANCELLATIONS_MESSAGE_JSON.to_owned()),
        other => panic!("unsupported primary type for fixture: {other}"),
    };
    TypedDataPayload::new(domain, primary_type.to_owned(), types, message_json)
}

/// Drives the cow `Eip1193Signer` for one `(chain, primary_type)` pair and
/// extracts the JSON the bridge passes to `eth_signTypedData_v4` as the
/// second parameter.
async fn capture_wire_payload(chain: SupportedChainId, primary_type: &str) -> serde_json::Value {
    let primary = Address::new(PRIMARY_ACCOUNT).unwrap();
    let transport = MockEip1193Transport::sepolia();
    transport.set_chain_id(chain);
    transport.set_connected(true);
    transport.set_accounts(vec![primary]);
    transport.set_default_call_result("0x".to_owned());
    let wallet = BrowserWallet::from_transport_or_panic(transport.clone());
    wallet.connect().await.expect("connect succeeds");
    let signer = wallet
        .provider()
        .create_signer(PRIMARY_ACCOUNT)
        .await
        .expect("signer constructs against the mock");

    let payload = fixture_payload(chain, primary_type);
    // The mock response is not a canonical signature; the test only
    // inspects the request log so the signer-surface Result is fine
    // either way.
    let _ = signer.sign_typed_data_payload(&payload).await;

    let log = transport.request_log();
    let record = log
        .iter()
        .find(|r| r.method == "eth_signTypedData_v4")
        .expect("signer must emit one eth_signTypedData_v4 request");
    let params = record
        .params
        .as_ref()
        .and_then(serde_json::Value::as_array)
        .expect("eth_signTypedData_v4 params must be an array");
    let typed_data_string = params
        .get(1)
        .and_then(serde_json::Value::as_str)
        .expect("eth_signTypedData_v4 params[1] must be the typed-data JSON string");
    serde_json::from_str(typed_data_string).expect("typed-data string must parse as JSON")
}

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
        address.to_hex_string(),
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
    assert_eq!(address.to_hex_string(), primary.to_hex_string());
}

/// Regen helper. Captures the canonical wire JSON the cow signer emits
/// for every `(chain, primary_type)` row in the
/// `parity/fixtures/signing/eth_sign_typed_data_request.json` fixture
/// and prints the full fixture body to stdout in the cow-rs parity
/// schema. Invoked manually when the fixture needs refreshing:
///
/// ```text
/// cargo test -p cow-sdk-browser-wallet --test signer_contract \
///     -- --ignored regen_eth_sign_typed_data_request_fixture --nocapture
/// ```
///
/// The output goes to `parity/fixtures/signing/eth_sign_typed_data_request.json`.
#[ignore = "regen helper; run with --ignored --nocapture to refresh the fixture"]
#[tokio::test(flavor = "current_thread")]
async fn regen_eth_sign_typed_data_request_fixture() {
    let mut rows: Vec<serde_json::Value> = Vec::new();
    for (chain, chain_name) in FIXTURE_CHAINS {
        for primary_type in ["Order", "OrderCancellations"] {
            let wire = capture_wire_payload(*chain, primary_type).await;
            rows.push(serde_json::json!({
                "chain_id": u64::from(*chain),
                "chain_name": chain_name,
                "primary_type": primary_type,
                "wire_payload": wire,
            }));
        }
    }
    let fixture = serde_json::json!({
        "schema_version": 1,
        "surface": "browser-wallet-eth-sign-typed-data-request",
        "dto": "cow_sdk_core::TypedDataPayload",
        "endpoint": "EIP-1193 eth_signTypedData_v4 (params[1])",
        "reviewed_at": "2026-05-21",
        "source_refs": [
            {
                "repo": "cow-sdk",
                "commit": "00c3dbd41c086ff9a51d5e5a30648615d4c66d0d",
                "path": "packages/common/src/adapters/types/index.ts",
                "line_start": 17,
                "line_end": 23
            },
            {
                "repo": "cow-sdk",
                "commit": "00c3dbd41c086ff9a51d5e5a30648615d4c66d0d",
                "path": "packages/contracts-ts/src/ContractsTs.ts",
                "line_start": 109,
                "line_end": 116
            },
            {
                "repo": "contracts",
                "commit": "main",
                "path": "src/contracts/mixins/GPv2Signing.sol",
                "line_start": 33,
                "line_end": 36
            }
        ],
        "rows": rows,
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&fixture).expect("fixture serializes")
    );
}

/// Pins the EIP-1193 `eth_signTypedData_v4` wire shape the cow
/// `Eip1193Signer` emits for the canonical `CoW` Protocol typed-data
/// payloads. Iterates every fixture row and asserts byte-identity
/// between the cow code's output and the committed fixture.
///
/// The fixture lives at
/// `parity/fixtures/signing/eth_sign_typed_data_request.json` and is
/// regenerable via the ignored `regen_eth_sign_typed_data_request_fixture`
/// helper above. Each row carries the canonical wire shape for one
/// `(supported chain, primary type)` pair across six supported chains
/// (mainnet, gnosis, polygon, base, arbitrum-one, sepolia) and two
/// primary types (`Order`, `OrderCancellations`).
#[tokio::test(flavor = "current_thread")]
#[allow(
    clippy::too_many_lines,
    reason = "single end-to-end loop walks every fixture row with defense-in-depth shape-pinning assertions alongside the byte-equality gate; splitting the body would scatter the wire-shape contract across multiple helpers"
)]
async fn typed_data_payload_emits_canonical_eip1193_wire_shape_against_fixture() {
    const FIXTURE_JSON: &str =
        include_str!("../../../parity/fixtures/signing/eth_sign_typed_data_request.json");
    let fixture: serde_json::Value =
        serde_json::from_str(FIXTURE_JSON).expect("fixture must parse as JSON");
    let rows = fixture
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .expect("fixture must carry a `rows` array");

    let chain_lookup: std::collections::HashMap<u64, SupportedChainId> = FIXTURE_CHAINS
        .iter()
        .map(|(chain, _)| (u64::from(*chain), *chain))
        .collect();

    for (index, row) in rows.iter().enumerate() {
        let chain_id = row
            .get("chain_id")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or_else(|| panic!("fixture row {index} missing chain_id"));
        let chain = *chain_lookup.get(&chain_id).unwrap_or_else(|| {
            panic!("fixture row {index} references unsupported chain {chain_id}")
        });
        let primary_type = row
            .get("primary_type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("fixture row {index} missing primary_type"));
        let expected_wire = row
            .get("wire_payload")
            .unwrap_or_else(|| panic!("fixture row {index} missing wire_payload"));

        let actual_wire = capture_wire_payload(chain, primary_type).await;

        // Defense-in-depth shape checks. The byte-equality assertion
        // below fails on any drift; these inline checks surface
        // precisely WHICH invariant the drift broke so the failure
        // diagnostic points at the root cause.
        let domain = actual_wire
            .get("domain")
            .and_then(serde_json::Value::as_object)
            .unwrap_or_else(|| {
                panic!("row {index}: wire payload must carry a JSON object `domain`")
            });
        let domain_chain_id = domain
            .get("chainId")
            .unwrap_or_else(|| panic!("row {index}: domain missing `chainId`"));
        assert!(
            domain_chain_id.is_number(),
            "row {index}: `chainId` must be a JSON Number per EIP-1193; got {domain_chain_id:?}"
        );
        assert_eq!(
            domain_chain_id.as_u64(),
            Some(chain_id),
            "row {index}: domain `chainId` must equal the cow-bound chain id"
        );
        assert_eq!(
            domain.get("name").and_then(serde_json::Value::as_str),
            Some(CANONICAL_DOMAIN_NAME),
            "row {index}: domain `name` must be the canonical cow value"
        );
        assert_eq!(
            domain.get("version").and_then(serde_json::Value::as_str),
            Some(CANONICAL_DOMAIN_VERSION),
            "row {index}: domain `version` must be the canonical cow value"
        );
        let verifying_contract = domain
            .get("verifyingContract")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_else(|| panic!("row {index}: domain missing `verifyingContract` string"));
        assert_eq!(
            verifying_contract.len(),
            42,
            "row {index}: `verifyingContract` must be a 0x-prefixed 42-char hex string; got `{verifying_contract}`"
        );
        assert!(
            verifying_contract
                .strip_prefix("0x")
                .is_some_and(|tail| tail.chars().all(|c| c.is_ascii_hexdigit())),
            "row {index}: `verifyingContract` must be a 0x-prefixed lowercase hex string; got `{verifying_contract}`"
        );
        assert!(
            !domain.contains_key("salt"),
            "row {index}: cow `TypedDataDomain` never emits a `salt` field on the wire"
        );
        assert_eq!(
            actual_wire
                .get("primaryType")
                .and_then(serde_json::Value::as_str),
            Some(primary_type),
            "row {index}: top-level `primaryType` must carry through verbatim"
        );
        assert!(
            actual_wire
                .get("types")
                .and_then(serde_json::Value::as_object)
                .is_some_and(|types| types.contains_key(primary_type)),
            "row {index}: `types` map must declare the primary type"
        );
        assert!(
            actual_wire
                .get("message")
                .is_some_and(serde_json::Value::is_object),
            "row {index}: `message` must be a JSON object"
        );

        // Byte-equality against the committed fixture. Fires last so
        // the defense-in-depth checks above pinpoint regressions.
        assert_eq!(
            &actual_wire, expected_wire,
            "fixture row {index} (chain_id={chain_id}, primary_type={primary_type}): cow wire \
             shape diverges from the committed fixture; run the regen helper to refresh"
        );
    }
}
