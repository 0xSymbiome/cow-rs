//! Behaviour tests for the composed Alloy client builder.
//!
//! These tests pin every documented branch on `AlloyClientBuilder` that the
//! happy-path integration tests do not exercise: the `InvalidUrl` and
//! `InvalidPrivateKey` rejection paths, bare-hex private-key acceptance,
//! the `ChainMismatch` rejection from `build_checked`, the
//! `From<AlloyClientError>` lift used by `?`-style propagation, and the
//! `Display` rendering of every builder error variant.
//!
//! The builder is the public entry point for native trading flows, so every
//! rejection here keeps secret material out of the rendered error and every
//! `Debug` impl redacts key bytes.

#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::{AlloyClientBuilder, AlloyClientBuilderError, AlloyClientError};
use cow_sdk_core::SupportedChainId;

/// secp256k1 test vector — the EIP-191 Ethereum test key used by the
/// alloy-signer reference vectors. Stable, well-known, never associated
/// with mainnet value.
const TEST_KEY_0X: &str = "0x59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";
const TEST_KEY_BARE: &str = "59c6995e998f97a5a0044966f0945389dc9e86dae88c7a8412f4603b6b78690d";

// -------------------------------------------------------------------------
// InvalidUrl rejection
// -------------------------------------------------------------------------

#[test]
fn http_rejects_malformed_url_without_echoing_the_input() {
    let result = AlloyClientBuilder::new().http("not-a-url");
    let Err(error) = result else {
        panic!("malformed URL must be rejected");
    };

    assert!(
        matches!(error, AlloyClientBuilderError::InvalidUrl),
        "expected InvalidUrl, got {error:?}",
    );
    let rendered = error.to_string();
    assert!(
        !rendered.contains("not-a-url"),
        "InvalidUrl display must not echo the offending input; got {rendered:?}",
    );
}

#[test]
fn http_rejects_credential_bearing_input_without_echoing_secrets() {
    let result = AlloyClientBuilder::new().http("not a url with secret-looking suffix");
    let Err(error) = result else {
        panic!("malformed URL must be rejected");
    };

    let debug = format!("{error:?}");
    let display = error.to_string();

    assert!(
        !debug.contains("secret-looking"),
        "debug echoed input: {debug:?}",
    );
    assert!(
        !display.contains("secret-looking"),
        "display echoed input: {display:?}",
    );
}

// -------------------------------------------------------------------------
// Private-key parsing — happy paths, error paths, bare-hex fallback
// -------------------------------------------------------------------------

#[test]
fn private_key_accepts_prefixed_hex_and_completes_typestate_to_terminal_form() {
    let _build_ready = AlloyClientBuilder::new()
        .http("https://example.invalid/rpc")
        .expect("valid URL")
        .private_key(TEST_KEY_0X)
        .expect("valid prefixed private key must be accepted")
        .chain_id(SupportedChainId::Mainnet);
}

#[test]
fn private_key_accepts_bare_hex_via_strip_prefix_fallback_path() {
    // The parser tries the raw value first, then `strip_prefix("0x")`. A bare
    // 64-character hex string must succeed via the strip-prefix fallback path.
    let _build_ready = AlloyClientBuilder::new()
        .http("https://example.invalid/rpc")
        .expect("valid URL")
        .private_key(TEST_KEY_BARE)
        .expect("bare hex must be accepted via strip-prefix fallback")
        .chain_id(SupportedChainId::Mainnet);
}

#[test]
fn private_key_rejects_malformed_inputs() {
    let cases = [
        ("", "empty input"),
        ("deadbeef", "too few hex bytes"),
        ("not-hex-at-all", "non-hex characters"),
        ("0x", "prefix only"),
        ("0x00", "single byte after prefix"),
        // 32 zero bytes is not a valid secp256k1 private key.
        (
            "0x0000000000000000000000000000000000000000000000000000000000000000",
            "all-zero key",
        ),
        // A key equal to the curve order is also invalid.
        (
            "0xfffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
            "key equal to curve order",
        ),
    ];

    for (input, reason) in cases {
        let result = AlloyClientBuilder::new().private_key(input);
        match result {
            Err(AlloyClientBuilderError::InvalidPrivateKey) => {}
            Err(other) => panic!("{reason}: expected InvalidPrivateKey, got {other:?}"),
            Ok(_) => panic!("{reason}: expected InvalidPrivateKey, got Ok(_)"),
        }
        // Inputs that contain potential secrets must never leak through error
        // rendering. The error message uses the static literal.
        let rendered = AlloyClientBuilderError::InvalidPrivateKey.to_string();
        assert_eq!(rendered, "invalid private key");
    }
}

#[test]
fn private_key_bytes_rejects_invalid_key_material() {
    // All-zero bytes is rejected by `PrivateKeySigner::from_bytes`.
    let result = AlloyClientBuilder::new().private_key_bytes([0u8; 32]);
    match result {
        Err(AlloyClientBuilderError::InvalidPrivateKey) => {}
        Err(other) => panic!("all-zero key bytes must be rejected; got {other:?}"),
        Ok(_) => panic!("all-zero key bytes must be rejected; got Ok(_)"),
    }
}

#[test]
fn private_key_bytes_accepts_valid_key_material_and_completes_typestate() {
    // A non-zero, less-than-curve-order key.
    let mut bytes = [0u8; 32];
    bytes[31] = 0x11;
    let _full = AlloyClientBuilder::new()
        .private_key_bytes(bytes)
        .expect("valid key bytes must be accepted")
        .http("https://example.invalid/rpc")
        .expect("valid URL")
        .chain_id(SupportedChainId::Mainnet);
}

// -------------------------------------------------------------------------
// build_checked — ChainMismatch rejection
// -------------------------------------------------------------------------

#[tokio::test]
async fn build_checked_rejects_chain_mismatch_with_both_chain_ids_in_display() {
    use wiremock::matchers::method;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    let server = MockServer::start().await;
    // wiremock returns chain id 100 (Gnosis Chain) when the client is
    // configured with chain id 1 (Mainnet) — the mismatch path.
    Mock::given(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "jsonrpc": "2.0",
            "id": 0,
            "result": "0x64"
        })))
        .mount(&server)
        .await;

    let result = AlloyClientBuilder::new()
        .http(server.uri())
        .expect("wiremock URL parses")
        .private_key(TEST_KEY_0X)
        .expect("valid private key")
        .chain_id(SupportedChainId::Mainnet)
        .build_checked()
        .await;

    let error = result.expect_err("mismatched remote chain id must be rejected");
    let display = error.to_string();
    match error {
        AlloyClientBuilderError::ChainMismatch { configured, remote } => {
            assert_eq!(configured, 1_u64, "configured chain id");
            assert_eq!(remote, 100_u64, "remote chain id");
            assert!(
                display.contains('1') && display.contains("100"),
                "ChainMismatch display must include both chain ids; got {display:?}",
            );
        }
        other => panic!("expected ChainMismatch, got {other:?}"),
    }
}

// -------------------------------------------------------------------------
// AlloyClientBuilder Debug — both reachable typestate variants
// -------------------------------------------------------------------------

#[test]
fn unset_builder_debug_renders_all_unset_fields() {
    let builder = AlloyClientBuilder::default();
    let debug = format!("{builder:?}");
    assert!(
        debug.contains("AlloyClientBuilder")
            && debug.contains("transport")
            && debug.contains("unset"),
        "unset builder debug must show all unset fields; got {debug:?}",
    );
}

#[test]
fn fully_typed_builder_debug_redacts_key_and_includes_chain_id() {
    let builder = AlloyClientBuilder::new()
        .http("https://example.invalid/rpc")
        .expect("valid URL")
        .private_key(TEST_KEY_0X)
        .expect("valid private key")
        .chain_id(SupportedChainId::Mainnet);

    let debug = format!("{builder:?}");
    assert!(
        debug.contains("[redacted]"),
        "fully-typed builder debug must redact key bytes; got {debug:?}",
    );
    assert!(
        !debug.contains(TEST_KEY_BARE),
        "fully-typed builder debug must not contain raw key material; got {debug:?}",
    );
    assert!(
        debug.contains("chain_id"),
        "fully-typed builder debug must include chain_id; got {debug:?}",
    );
}

// -------------------------------------------------------------------------
// AlloyClientBuilderError::Display + From<AlloyClientError>
// -------------------------------------------------------------------------

#[test]
fn builder_error_display_renders_each_variant_safely() {
    assert_eq!(
        AlloyClientBuilderError::InvalidUrl.to_string(),
        "rpc url failed to parse",
    );
    assert_eq!(
        AlloyClientBuilderError::InvalidPrivateKey.to_string(),
        "invalid private key",
    );

    let mismatch = AlloyClientBuilderError::ChainMismatch {
        configured: 1_u64,
        remote: 100_u64,
    };
    let rendered = mismatch.to_string();
    assert!(
        rendered.contains('1') && rendered.contains("100"),
        "ChainMismatch display must include both chain ids; got {rendered:?}",
    );
}

#[test]
fn from_alloy_client_error_lifts_into_client_variant_transparently() {
    let inner = AlloyClientError::Internal("inner detail".to_owned());
    let lifted: AlloyClientBuilderError = inner.into();

    assert!(
        matches!(lifted, AlloyClientBuilderError::Client(_)),
        "From<AlloyClientError> must lift to Client variant; got {lifted:?}",
    );
    let rendered = lifted.to_string();
    // The variant is `#[error(transparent)]` so it forwards to the wrapped
    // error's Display, which renders `internal error: [redacted]`.
    assert!(
        rendered.contains("internal error"),
        "Client variant must forward transparently; got {rendered:?}",
    );
    assert!(
        !rendered.contains("inner detail"),
        "Client variant must not leak the wrapped detail; got {rendered:?}",
    );
}
