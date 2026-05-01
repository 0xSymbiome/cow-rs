//! Public-surface contract assertions for every structured
//! [`cow_sdk_contracts::ContractsError`] variant.
//!
//! Each test destructures the typed shape of one variant through an
//! exhaustive pattern match so the public contract is exercised against
//! the canonical field set the consumer-facing API documents. Any future
//! variant whose shape drifts from this contract fails the corresponding
//! test at compile time.

use cow_sdk_contracts::ContractsError;
use cow_sdk_core::Address;

const fn assert_typed_token_address(_: &Address) {}
const fn assert_typed_magic_value_bytes(_: [u8; 4]) {}

#[test]
fn missing_clearing_price_carries_typed_token_address() {
    let token =
        Address::new("0x1111111111111111111111111111111111111111").expect("literal must parse");
    let error = ContractsError::MissingClearingPrice {
        token: token.clone(),
    };

    let ContractsError::MissingClearingPrice { token: extracted } = &error else {
        panic!("expected MissingClearingPrice variant, got {error:?}");
    };
    assert_typed_token_address(extracted);
    assert_eq!(extracted, &token);

    assert_eq!(
        error.to_string(),
        format!("missing clearing price for token {}", token.as_str()),
    );
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
        message: "transport dropped".to_owned(),
    };

    let ContractsError::Provider { operation, message } = &error else {
        panic!("expected Provider variant, got {error:?}");
    };
    assert_eq!(*operation, "read_contract");
    assert_eq!(message, "transport dropped");
    assert_eq!(
        error.to_string(),
        "provider error during read_contract: transport dropped",
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
fn forbidden_interaction_target_carries_typed_target_address() {
    let target =
        Address::new("0x1111111111111111111111111111111111111111").expect("literal must parse");
    let error = ContractsError::ForbiddenInteractionTarget {
        target: target.clone(),
    };

    let ContractsError::ForbiddenInteractionTarget { target: extracted } = &error else {
        panic!("expected ForbiddenInteractionTarget variant, got {error:?}");
    };
    assert_typed_token_address(extracted);
    assert_eq!(extracted, &target);
    assert_eq!(
        error.to_string(),
        format!(
            "forbidden settlement interaction target: {}",
            target.as_str()
        ),
    );
}

#[test]
fn decode_hex_variant_wraps_hex_from_hex_error_source() {
    let source = hex::decode("zzzz").unwrap_err();
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
fn serialization_variant_wraps_serde_json_error_via_from_conversion() {
    let source = serde_json::from_str::<serde_json::Value>("{ malformed").unwrap_err();
    let error: ContractsError = source.into();

    match &error {
        ContractsError::Serialization(inner) => {
            let _ = inner;
        }
        other => panic!("expected Serialization(#[from] serde_json::Error), got {other:?}"),
    }
}
