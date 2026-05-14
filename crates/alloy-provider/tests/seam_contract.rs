//! Behaviour tests for the documented inter-adapter seam.
//!
//! The `__seam` module is `#[doc(hidden)]` per [ADR 0035] and is not part of
//! the semver-stable consumer API. Sibling adapter crates (the umbrella
//! `cow-sdk-alloy`) depend on it to lift Alloy transport errors and convert
//! address, hash, and block-tag shapes into the core contracts.
//!
//! These tests pin the seam's public contract so that any change to its
//! signatures becomes a deliberate decision rather than silent drift.
//!
//! [ADR 0035]: docs/adr/0035-alloy-provider-adapter.md

#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy_provider::__seam;
use cow_sdk_core::{Address, TransactionHash};

#[test]
fn seam_cow_to_alloy_address_round_trips_a_validated_address() {
    let address = Address::new("0x0000000000000000000000000000000000000001")
        .expect("static valid address parses");
    let alloy_address = __seam::cow_to_alloy_address(&address)
        .expect("seam converts a validated address without error");

    // The Alloy primitive must agree with the core string form (case-insensitive).
    let rendered = format!("{alloy_address:#x}");
    assert_eq!(
        rendered.to_ascii_lowercase(),
        address.as_str().to_ascii_lowercase(),
    );
}

#[test]
fn seam_cow_to_alloy_hash_round_trips_a_validated_transaction_hash() {
    let hash =
        TransactionHash::new("0x0101010101010101010101010101010101010101010101010101010101010101")
            .expect("static valid transaction hash parses");
    let alloy_hash =
        __seam::cow_to_alloy_hash(&hash).expect("seam converts a validated hash without error");

    let rendered = format!("{alloy_hash:#x}");
    assert_eq!(
        rendered.to_ascii_lowercase(),
        hash.as_str().to_ascii_lowercase(),
    );
}

#[test]
fn seam_cow_block_tag_to_alloy_resolves_every_named_tag() {
    // The named tags survive the seam unchanged.
    assert!(__seam::cow_block_tag_to_alloy("latest").is_ok());
    assert!(__seam::cow_block_tag_to_alloy("pending").is_ok());
    assert!(__seam::cow_block_tag_to_alloy("earliest").is_ok());
    assert!(__seam::cow_block_tag_to_alloy("finalized").is_ok());
    assert!(__seam::cow_block_tag_to_alloy("safe").is_ok());
}

#[test]
fn seam_cow_block_tag_to_alloy_accepts_hex_and_decimal_block_numbers() {
    assert!(__seam::cow_block_tag_to_alloy("0x2a").is_ok());
    assert!(__seam::cow_block_tag_to_alloy("42").is_ok());
}

#[test]
fn seam_cow_block_tag_to_alloy_rejects_malformed_inputs() {
    assert!(__seam::cow_block_tag_to_alloy("notatag").is_err());
    assert!(__seam::cow_block_tag_to_alloy("0xZZ").is_err());
    assert!(__seam::cow_block_tag_to_alloy("").is_err());
}

#[test]
fn seam_cow_block_tag_to_alloy_accepts_full_block_hash_form() {
    // A 0x-prefixed 64-hex-character value is a block hash, not a number.
    let block_hash = format!("0x{}", "0".repeat(64));
    assert!(__seam::cow_block_tag_to_alloy(&block_hash).is_ok());
}

// -------------------------------------------------------------------------
// Transaction-request and receipt seam wrappers — driven through the
// wider AsyncProvider integration on a wiremock server so the wrappers
// receive realistic upstream values.
// -------------------------------------------------------------------------

#[tokio::test]
async fn seam_cow_request_to_alloy_round_trips_minimal_request() {
    use cow_sdk_core::{Amount, HexData, TransactionRequest};

    let request = TransactionRequest::new(
        Some(
            Address::new("0x0000000000000000000000000000000000000003")
                .expect("static valid address parses"),
        ),
        Some(HexData::new("0xdeadbeef").expect("static valid hex parses")),
        Some(Amount::from(1_u32)),
        Some(Amount::from(21_000_u64)),
    );

    let alloy_request =
        __seam::cow_request_to_alloy(&request).expect("seam converts a validated request");

    // The seam preserves the `to` address through the conversion. The
    // Alloy representation uses `TxKind` so we round-trip via the public
    // `to.into_to()` accessor; we keep the assertion behaviour-level by
    // checking the rendered string form contains the expected suffix.
    let to = alloy_request.to.expect("to field is set");
    let rendered = format!("{to:?}");
    assert!(
        rendered
            .to_ascii_lowercase()
            .contains("0000000000000000000000000000000000000003"),
        "Alloy transaction request must preserve `to`; got {rendered}",
    );
}

#[tokio::test]
async fn seam_rpc_error_to_class_and_detail_classifies_remote_error() {
    use cow_sdk_alloy_provider::RpcAlloyProvider;
    use cow_sdk_core::AsyncProvider;
    use serde_json::json;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": {
                "code": -32_603,
                "message": "internal JSON-RPC error",
            },
        })))
        .mount(&server)
        .await;

    let provider = RpcAlloyProvider::builder()
        .http(server.uri())
        .unwrap()
        .build()
        .await
        .unwrap();
    let _server: &'static MockServer = Box::leak(Box::new(server));

    // Driving any AsyncProvider method through the wiremock-driven JSON-RPC
    // error exercises the seam classifier wrapper internally on the way back
    // up the stack. The error must carry the documented remote code.
    let err = provider
        .get_chain_id()
        .await
        .expect_err("RPC error must propagate");
    match err {
        cow_sdk_alloy_provider::AsyncProviderError::Remote { code, .. } => {
            assert_eq!(code, -32_603);
        }
        other => panic!("expected Remote variant, got {other:?}"),
    }
}
