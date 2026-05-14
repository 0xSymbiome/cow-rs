//! Behaviour tests for the composed Alloy adapter's error surface.
//!
//! `redaction_contract.rs` already pins `Debug` and `Display` redaction for
//! every direct `AlloyClientError` variant. This file complements that by
//! exercising:
//!
//! - the `class().as_str()` label table across every variant;
//! - `AlloyClientErrorClass::Display` forwarding through `as_str`;
//! - every documented `From<...>` lift used by `?`-style propagation across
//!   `CoreError`, `Cancelled`, `ContractsError`, and `AsyncProviderError`.
//!
//! The seam constructors `from_alloy_transport`, `from_alloy_signer`, and
//! `from_pending_tx_error` are covered indirectly through the wiremock-driven
//! end-to-end RPC scenarios in `asyncprovider_contract.rs` and
//! `asyncsigningprovider_contract.rs`; constructing the upstream Alloy error
//! shapes directly is intentionally avoided because their internal fields are
//! private.

#![cfg(not(target_arch = "wasm32"))]

use cow_sdk_alloy::{AlloyClientError, AlloyClientErrorClass};
use cow_sdk_alloy_provider::AsyncProviderError;
use cow_sdk_core::{Cancelled, Redacted, TransportErrorClass};

// -------------------------------------------------------------------------
// AlloyClientErrorClass label table
// -------------------------------------------------------------------------

#[test]
fn class_label_table_covers_every_variant() {
    let cases: &[(AlloyClientError, AlloyClientErrorClass, &str)] = &[
        (
            AlloyClientError::Validation("ignored".to_owned()),
            AlloyClientErrorClass::Validation,
            "validation",
        ),
        (
            AlloyClientError::Transport {
                class: TransportErrorClass::Timeout,
                detail: Redacted::new("ignored".to_owned()),
            },
            AlloyClientErrorClass::Transport,
            "transport",
        ),
        (
            AlloyClientError::Remote {
                code: -32_000,
                message: "execution reverted".to_owned(),
            },
            AlloyClientErrorClass::Remote,
            "remote",
        ),
        (
            AlloyClientError::Signing {
                detail: Redacted::new("ignored".to_owned()),
            },
            AlloyClientErrorClass::Signing,
            "signing",
        ),
        (
            AlloyClientError::PendingTransaction {
                detail: Redacted::new("ignored".to_owned()),
            },
            AlloyClientErrorClass::PendingTransaction,
            "pending_transaction",
        ),
        (
            AlloyClientError::UnsupportedTransactionRequest {
                method: "sign_transaction",
                reason: "raw transaction signing is deferred",
            },
            AlloyClientErrorClass::UnsupportedTransactionRequest,
            "unsupported_transaction_request",
        ),
        (
            AlloyClientError::Cancelled,
            AlloyClientErrorClass::Cancelled,
            "cancelled",
        ),
        (
            AlloyClientError::Internal("ignored".to_owned()),
            AlloyClientErrorClass::Internal,
            "internal",
        ),
    ];

    for (error, expected_class, expected_label) in cases {
        let observed_class = error.class();
        assert_eq!(
            observed_class, *expected_class,
            "class() mapping for {error:?}",
        );
        assert_eq!(
            observed_class.as_str(),
            *expected_label,
            "as_str() label for {observed_class:?}",
        );
        assert_eq!(
            format!("{observed_class}"),
            *expected_label,
            "Display forwarding for {observed_class:?}",
        );
    }
}

// -------------------------------------------------------------------------
// From<AsyncProviderError> lift — exercises the inter-adapter seam
// -------------------------------------------------------------------------

#[test]
fn from_async_provider_error_validation_lifts_to_validation_variant() {
    let upstream = AsyncProviderError::Validation("invalid chain id".to_owned());
    let lifted: AlloyClientError = upstream.into();
    assert!(matches!(lifted, AlloyClientError::Validation(_)));
    assert_eq!(lifted.class(), AlloyClientErrorClass::Validation);
}

#[test]
fn from_async_provider_error_transport_lifts_to_transport_variant() {
    let upstream = AsyncProviderError::Transport {
        class: TransportErrorClass::Timeout,
        detail: Redacted::new("timed out".to_owned()),
    };
    let lifted: AlloyClientError = upstream.into();
    match lifted {
        AlloyClientError::Transport { class, .. } => {
            assert_eq!(class, TransportErrorClass::Timeout);
        }
        other => panic!("expected Transport, got {other:?}"),
    }
}

#[test]
fn from_async_provider_error_remote_lifts_to_remote_variant() {
    let upstream = AsyncProviderError::Remote {
        code: -32_000,
        message: "execution reverted".to_owned(),
    };
    let lifted: AlloyClientError = upstream.into();
    match lifted {
        AlloyClientError::Remote { code, message } => {
            assert_eq!(code, -32_000);
            assert_eq!(message, "execution reverted");
        }
        other => panic!("expected Remote, got {other:?}"),
    }
}

#[test]
fn from_async_provider_error_cancelled_lifts_to_cancelled_variant() {
    let upstream = AsyncProviderError::Cancelled;
    let lifted: AlloyClientError = upstream.into();
    assert!(matches!(lifted, AlloyClientError::Cancelled));
    assert_eq!(lifted.class(), AlloyClientErrorClass::Cancelled);
}

#[test]
fn from_async_provider_error_internal_lifts_to_internal_variant() {
    let upstream = AsyncProviderError::Internal("internal detail".to_owned());
    let lifted: AlloyClientError = upstream.into();
    assert!(matches!(lifted, AlloyClientError::Internal(_)));
    assert_eq!(lifted.class(), AlloyClientErrorClass::Internal);
    // The lifted detail must not leak through Display.
    assert!(!lifted.to_string().contains("internal detail"));
}

// -------------------------------------------------------------------------
// From<cow_sdk_core::Cancelled>, From<cow_sdk_core::CoreError>,
// From<cow_sdk_contracts::ContractsError>
// -------------------------------------------------------------------------

#[test]
fn from_cancelled_token_lifts_to_cancelled_variant() {
    let lifted: AlloyClientError = Cancelled.into();
    assert!(matches!(lifted, AlloyClientError::Cancelled));
    assert_eq!(lifted.to_string(), "operation cancelled");
}

#[test]
fn from_core_error_lifts_into_validation_variant() {
    // ValidationError -> CoreError::Validation -> AlloyClientError::Validation.
    let core_err: cow_sdk_core::CoreError = cow_sdk_core::ValidationError::InvalidHexLength {
        field: "address",
        expected: 40,
    }
    .into();
    let lifted: AlloyClientError = core_err.into();
    assert!(matches!(lifted, AlloyClientError::Validation(_)));
    assert_eq!(lifted.class(), AlloyClientErrorClass::Validation);
    // The Validation Display path emits the static `[redacted]` placeholder.
    let rendered = lifted.to_string();
    assert!(rendered.starts_with("validation error:"));
    assert!(rendered.contains("[redacted]"));
}

#[test]
fn from_contracts_error_lifts_into_signing_variant_with_redacted_detail() {
    // A simple unit variant of `ContractsError` whose Display string is stable
    // and easy to confirm does not leak through the Signing wrapper.
    let contracts_err: cow_sdk_contracts::ContractsError =
        cow_sdk_contracts::ContractsError::InvalidEip1271SignatureData;
    let detail_str = contracts_err.to_string();
    let lifted: AlloyClientError = contracts_err.into();
    assert!(matches!(lifted, AlloyClientError::Signing { .. }));
    assert_eq!(lifted.class(), AlloyClientErrorClass::Signing);
    // The Signing display routes through `Redacted<String>` and emits
    // `[redacted]`; the underlying detail must not surface.
    let rendered = lifted.to_string();
    assert!(rendered.starts_with("signing error:"));
    assert!(rendered.contains("[redacted]"));
    assert!(
        !rendered.contains(&detail_str),
        "Signing variant must not leak ContractsError detail; got {rendered:?}",
    );
}
