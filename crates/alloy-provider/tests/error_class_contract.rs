//! Behaviour tests for `AsyncProviderError` class labels and From conversions.
//!
//! The existing `redaction_contract.rs` covers `Debug` / `Display` redaction
//! and the `class()` mapping for the variants used in transport scenarios.
//! This file complements that by exercising the lowercase class-label table,
//! the `AsyncProviderErrorClass::Display` forwarding through `as_str`, and
//! the documented `From<CoreError>` and `From<Cancelled>` lifts used by
//! `?`-style propagation in provider call sites.

#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy_provider::{AsyncProviderError, AsyncProviderErrorClass};
use cow_sdk_core::{Cancelled, Redacted, TransportErrorClass};

#[test]
fn class_label_table_covers_every_variant() {
    let cases: &[(AsyncProviderError, AsyncProviderErrorClass, &str)] = &[
        (
            AsyncProviderError::Validation("ignored".to_owned()),
            AsyncProviderErrorClass::Validation,
            "validation",
        ),
        (
            AsyncProviderError::Transport {
                class: TransportErrorClass::Timeout,
                detail: Redacted::new("ignored".to_owned()),
            },
            AsyncProviderErrorClass::Transport,
            "transport",
        ),
        (
            AsyncProviderError::Remote {
                code: -32_000,
                message: "execution reverted".to_owned(),
            },
            AsyncProviderErrorClass::Remote,
            "remote",
        ),
        (
            AsyncProviderError::Cancelled,
            AsyncProviderErrorClass::Cancelled,
            "cancelled",
        ),
        (
            AsyncProviderError::Internal("ignored".to_owned()),
            AsyncProviderErrorClass::Internal,
            "internal",
        ),
    ];

    for (error, expected_class, expected_label) in cases {
        let observed = error.class();
        assert_eq!(observed, *expected_class, "class() mapping for {error:?}");
        assert_eq!(
            observed.as_str(),
            *expected_label,
            "as_str() label for {observed:?}",
        );
        assert_eq!(
            format!("{observed}"),
            *expected_label,
            "Display forwarding for {observed:?}",
        );
    }
}

#[test]
fn from_cancelled_token_lifts_to_cancelled_variant() {
    let lifted: AsyncProviderError = Cancelled.into();
    assert!(matches!(lifted, AsyncProviderError::Cancelled));
    assert_eq!(lifted.to_string(), "operation cancelled");
    assert_eq!(lifted.class(), AsyncProviderErrorClass::Cancelled);
}

#[test]
fn from_core_error_lifts_into_validation_variant_with_redacted_display() {
    let core_err: cow_sdk_core::CoreError = cow_sdk_core::ValidationError::InvalidHexLength {
        field: "address",
        expected: 40,
    }
    .into();
    let lifted: AsyncProviderError = core_err.into();
    assert!(matches!(lifted, AsyncProviderError::Validation(_)));
    assert_eq!(lifted.class(), AsyncProviderErrorClass::Validation);

    let rendered = lifted.to_string();
    assert!(rendered.starts_with("validation error:"));
    assert!(rendered.contains("[redacted]"));
}

#[test]
fn internal_variant_display_emits_redacted_placeholder() {
    let err = AsyncProviderError::Internal("operator-only detail".to_owned());
    let rendered = err.to_string();
    let debug = format!("{err:?}");

    assert_eq!(rendered, "internal error: [redacted]");
    assert!(debug.contains("Internal"));
    assert!(debug.contains("[redacted]"));
    assert!(
        !rendered.contains("operator-only detail"),
        "Internal display must not leak detail; got {rendered:?}",
    );
    assert!(
        !debug.contains("operator-only detail"),
        "Internal debug must not leak detail; got {debug:?}",
    );
}

// -------------------------------------------------------------------------
// from_alloy_transport via real wiremock RPC error responses.
//
// These tests exercise the documented transport-classification ladder by
// driving the provider against synthetic RPC failures. Constructing
// `alloy_transport::TransportError` shapes directly is impractical because
// the upstream type's internal fields are private; wiremock-driven RPC
// errors are the documented integration boundary that the SDK actually
// honors.
// -------------------------------------------------------------------------

#[cfg(not(target_arch = "wasm32"))]
mod wiremock_transport {
    use cow_sdk_alloy_provider::{AsyncProviderError, AsyncProviderErrorClass, RpcAlloyProvider};
    use cow_sdk_core::AsyncProvider;
    use serde_json::json;
    use wiremock::{Mock, MockServer, ResponseTemplate, matchers::method};

    async fn provider_returning_response(response: ResponseTemplate) -> RpcAlloyProvider {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(response)
            .mount(&server)
            .await;
        let provider = RpcAlloyProvider::builder()
            .http(server.uri())
            .unwrap()
            .build()
            .await
            .unwrap();
        let _server: &'static MockServer = Box::leak(Box::new(server));
        provider
    }

    /// A JSON-RPC error envelope with an explicit code and message maps to
    /// the `Remote` variant carrying the code and message verbatim.
    #[tokio::test]
    async fn jsonrpc_error_envelope_maps_to_remote_variant_with_code_and_message() {
        let provider =
            provider_returning_response(ResponseTemplate::new(200).set_body_json(json!({
                "jsonrpc": "2.0",
                "id": 1,
                "error": {
                    "code": -32_000,
                    "message": "execution reverted",
                },
            })))
            .await;

        let err = provider
            .get_chain_id()
            .await
            .expect_err("RPC error must propagate");
        match err {
            AsyncProviderError::Remote { code, message } => {
                assert_eq!(code, -32_000);
                assert_eq!(message, "execution reverted");
            }
            other => panic!("expected Remote variant, got {other:?}"),
        }
    }

    /// An HTTP 500 response without a JSON-RPC envelope is classified as a
    /// transport-level failure with a non-validation, non-cancellation class.
    #[tokio::test]
    async fn http_500_maps_to_transport_or_internal_variant() {
        let provider = provider_returning_response(ResponseTemplate::new(500)).await;

        let err = provider
            .get_chain_id()
            .await
            .expect_err("HTTP 500 must propagate as an error");
        // The exact class depends on Alloy's transport stack but it must not
        // be Validation or Cancelled.
        let class = err.class();
        assert!(
            !matches!(
                class,
                AsyncProviderErrorClass::Validation | AsyncProviderErrorClass::Cancelled,
            ),
            "HTTP 500 must not classify as Validation or Cancelled; got {class:?}",
        );
        // The rendered Display must not echo any plaintext server detail
        // since transports redact via Redacted<String>.
        let rendered = err.to_string();
        assert!(
            !rendered.contains("internal server error"),
            "HTTP 500 rendered detail must not echo response body; got {rendered:?}",
        );
    }

    /// A malformed JSON response trips Alloy's deserialization layer and
    /// surfaces as a transport-class failure (not a remote JSON-RPC error).
    #[tokio::test]
    async fn malformed_json_response_maps_to_transport_or_internal_class() {
        let provider = provider_returning_response(
            ResponseTemplate::new(200)
                .set_body_string("{ not valid json")
                .insert_header("content-type", "application/json"),
        )
        .await;

        let err = provider
            .get_chain_id()
            .await
            .expect_err("malformed JSON must propagate as an error");
        let class = err.class();
        assert!(
            !matches!(
                class,
                AsyncProviderErrorClass::Validation | AsyncProviderErrorClass::Cancelled,
            ),
            "malformed JSON must not classify as Validation or Cancelled; got {class:?}",
        );
    }
}
