//! Public-surface regression for the typed shape of
//! [`cow_sdk_contracts::ContractsError::MissingClearingPrice`] and
//! [`cow_sdk_contracts::ContractsError::Eip1271MagicValueMismatch`].
//!
//! The two variants previously carried stringly-typed payloads that
//! lost information at construction time: `MissingClearingPrice` took a
//! token-address string, and `Eip1271MagicValueMismatch` carried
//! hex-encoded strings for two values that are always 4-byte function
//! selectors. Both variants are now structured around their typed
//! shapes so downstream callers can pattern-match on the typed
//! payload without re-parsing error messages. This test destructures
//! the new shape and asserts the typed field types through an
//! exhaustive pattern match; if either variant regresses to a
//! `(String)` payload (or any shape other than the reviewed typed
//! form), this file fails to compile.

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

    let ContractsError::MissingClearingPrice { token: extracted } = error.clone() else {
        panic!("expected MissingClearingPrice variant, got {error:?}");
    };
    assert_typed_token_address(&extracted);
    assert_eq!(extracted, token);

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
    } = error.clone()
    else {
        panic!("expected Eip1271MagicValueMismatch variant, got {error:?}");
    };
    assert_typed_magic_value_bytes(extracted_expected);
    assert_typed_magic_value_bytes(extracted_actual);
    assert_eq!(extracted_expected, expected);
    assert_eq!(extracted_actual, actual);

    assert_eq!(
        error.to_string(),
        "unexpected EIP-1271 magic value: expected 0x1626ba7e, got 0xffffffff",
    );
}
