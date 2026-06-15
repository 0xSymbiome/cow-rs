//! Public-surface contract assertions for `cow_sdk_contracts::ContractsError`:
//! the cancellation lift and combinator, plus the typed shape of every
//! structured variant.
//!
//! Each variant test destructures the typed shape through an exhaustive pattern
//! match so the public contract is exercised against the canonical field set the
//! consumer-facing API documents. Any future variant whose shape drifts from
//! this contract fails the corresponding test at compile time.

use cow_sdk_contracts::ContractsError;
use cow_sdk_core::{Cancellable, CancellationToken, Cancelled};

const fn assert_typed_magic_value_bytes(_: [u8; 4]) {}

#[test]
fn cancelled_marker_lifts_to_contracts_error_cancelled() {
    let error = ContractsError::from(Cancelled);

    assert!(matches!(error, ContractsError::Cancelled));
}

#[tokio::test]
async fn cancellation_combinator_composes_with_contracts_error() {
    let token = CancellationToken::new();
    token.cancel();

    let result = async { Ok::<_, ContractsError>(()) }
        .cancel_with(&token)
        .await;

    assert!(matches!(result, Err(ContractsError::Cancelled)));
}

#[test]
fn eip1271_magic_value_mismatch_carries_typed_four_byte_arrays() {
    let expected: [u8; 4] = [0x16, 0x26, 0xba, 0x7e];
    let actual: [u8; 4] = [0xff, 0xff, 0xff, 0xff];
    let error = ContractsError::Eip1271MagicValueMismatch { expected, actual };

    let ContractsError::Eip1271MagicValueMismatch {
        expected: extracted_expected,
        actual: extracted_actual,
    } = &error
    else {
        panic!("expected Eip1271MagicValueMismatch variant, got {error:?}");
    };
    assert_typed_magic_value_bytes(*extracted_expected);
    assert_typed_magic_value_bytes(*extracted_actual);
    assert_eq!(extracted_expected, &expected);
    assert_eq!(extracted_actual, &actual);

    assert_eq!(
        error.to_string(),
        "unexpected EIP-1271 magic value: expected 0x1626ba7e, got 0xffffffff",
    );
}

#[test]
fn provider_variant_carries_structured_operation_and_message_fields() {
    let error = ContractsError::Provider {
        operation: "read_contract",
        message: "transport dropped".to_owned().into(),
    };

    let ContractsError::Provider { operation, message } = &error else {
        panic!("expected Provider variant, got {error:?}");
    };
    assert_eq!(*operation, "read_contract");
    assert_eq!(message.as_inner(), "transport dropped");
    assert_eq!(
        error.to_string(),
        "provider error during read_contract: [redacted]",
    );
}

#[test]
fn abi_variant_wraps_alloy_sol_types_error_via_from_conversion() {
    let source = alloy_sol_types::Error::Overrun;
    let error: ContractsError = source.into();

    match &error {
        ContractsError::Abi(inner) => {
            let _ = inner;
        }
        other => panic!("expected Abi(#[from] alloy_sol_types::Error), got {other:?}"),
    }
}

#[test]
fn invalid_decoded_length_variant_carries_structured_field_expected_and_actual_fields() {
    let error = ContractsError::InvalidDecodedLength {
        field: "orderUid",
        expected: 56,
        actual: 4,
    };

    let ContractsError::InvalidDecodedLength {
        field,
        expected,
        actual,
    } = &error
    else {
        panic!("expected InvalidDecodedLength variant, got {error:?}");
    };
    assert_eq!(*field, "orderUid");
    assert_eq!(*expected, 56);
    assert_eq!(*actual, 4);
}

#[test]
fn decode_hex_variant_wraps_hex_from_hex_error_source() {
    let source = alloy_primitives::hex::decode("zzzz").unwrap_err();
    let error = ContractsError::DecodeHex {
        field: "appData",
        source,
    };

    let ContractsError::DecodeHex { field, source } = &error else {
        panic!("expected DecodeHex variant, got {error:?}");
    };
    assert_eq!(*field, "appData");
    assert!(format!("{source}").to_ascii_lowercase().contains("invalid"));
}

#[test]
fn invalid_hex_prefix_variant_carries_field_context() {
    let error = ContractsError::InvalidHexPrefix {
        field: "verifyingContract",
    };

    let ContractsError::InvalidHexPrefix { field } = &error else {
        panic!("expected InvalidHexPrefix variant, got {error:?}");
    };
    assert_eq!(*field, "verifyingContract");
}

#[test]
fn serialization_variant_drops_raw_serde_error_for_structured_position() {
    let source = serde_json::from_str::<serde_json::Value>("{ malformed").unwrap_err();
    let error: ContractsError = source.into();

    let ContractsError::Serialization {
        category,
        line,
        column,
    } = &error
    else {
        panic!("expected Serialization {{ category, line, column }}, got {error:?}");
    };
    assert_eq!(*category, "syntax");
    assert!(*line >= 1 && *column >= 1);
    // The structured diagnostic renders the category and position, never the
    // raw serde error text that could echo decoded bytes (ADR 0025).
    assert_eq!(
        error.to_string(),
        format!("serialization error ({category}) at line {line} column {column}"),
    );
}

#[test]
fn class_partitions_validation_internal_and_signing() {
    use cow_sdk_core::ErrorClass;

    // Caller-supplied input that failed a shape or range check is validation.
    assert_eq!(
        ContractsError::UnsupportedChain(999_999).class(),
        ErrorClass::Validation,
    );
    assert_eq!(
        ContractsError::InvalidOrderUidLength { actual: 4 }.class(),
        ErrorClass::Validation,
    );

    // Serialization, ABI, and decode invariants are internal.
    assert_eq!(
        ContractsError::Abi(alloy_sol_types::Error::Overrun).class(),
        ErrorClass::Internal,
    );
    let serde_error: ContractsError = serde_json::from_str::<serde_json::Value>("{ malformed")
        .unwrap_err()
        .into();
    assert_eq!(serde_error.class(), ErrorClass::Internal);

    // EIP-1271, provider, and ECDSA-recovery operations are signing-edge.
    assert_eq!(
        ContractsError::SignatureSchemeNotEcdsa.class(),
        ErrorClass::Signing,
    );
    assert_eq!(
        ContractsError::Provider {
            operation: "eth_call",
            message: "boom".to_owned().into(),
        }
        .class(),
        ErrorClass::Signing,
    );
}
