//! Public-surface regressions for the typed shape of every structured
//! [`cow_sdk_contracts::ContractsError`] variant that previously carried
//! stringly-typed payloads.
//!
//! Each test destructures the current shape through an exhaustive pattern
//! match; any regression to a `(String)` payload (or any shape other than the
//! reviewed typed form) fails this file at compile time.

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
fn decode_variant_carries_structured_field_and_message_fields() {
    let error = ContractsError::Decode {
        field: "orderUid",
        message: "value must be 56 bytes, got 4".to_owned(),
    };

    let ContractsError::Decode { field, message } = &error else {
        panic!("expected Decode variant, got {error:?}");
    };
    assert_eq!(*field, "orderUid");
    assert!(message.contains("must be 56 bytes"));
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
