//! Public-surface contract assertion for the typed shape of
//! [`cow_sdk_trading::TradingError::RecoverableSignatureOwnerMismatch`].
//!
//! The variant carries typed [`cow_sdk_core::Address`] fields for the
//! `owner` and `signer` so downstream callers pattern-match on the typed
//! payload without re-parsing error messages. The `owner` is the address
//! resolved from the order payload and the `signer` is the address
//! produced by the signing backend. This test destructures the typed
//! shape through an exhaustive pattern match; any future variant whose
//! shape drifts from this contract fails this file at compile time.

use cow_sdk_core::Address;
use cow_sdk_orderbook::SigningScheme;
use cow_sdk_trading::TradingError;

const fn assert_typed_scheme(_: SigningScheme) {}
const fn assert_typed_address(_: &Address) {}

#[test]
fn recoverable_signature_owner_mismatch_carries_typed_addresses() {
    let owner =
        Address::new("0x1111111111111111111111111111111111111111").expect("literal must parse");
    let signer =
        Address::new("0x2222222222222222222222222222222222222222").expect("literal must parse");
    let error = TradingError::RecoverableSignatureOwnerMismatch {
        scheme: SigningScheme::Eip712,
        owner: owner.clone(),
        signer: signer.clone(),
    };

    let TradingError::RecoverableSignatureOwnerMismatch {
        scheme,
        owner: extracted_owner,
        signer: extracted_signer,
    } = &error
    else {
        panic!("expected RecoverableSignatureOwnerMismatch variant, got {error:?}");
    };
    assert_typed_scheme(*scheme);
    assert_typed_address(extracted_owner);
    assert_typed_address(extracted_signer);
    assert_eq!(*scheme, SigningScheme::Eip712);
    assert_eq!(extracted_owner, &owner);
    assert_eq!(extracted_signer, &signer);

    assert_eq!(
        error.to_string(),
        format!(
            "recoverable signing scheme `Eip712` requires owner `{}` to match signer `{}`",
            owner.as_str(),
            signer.as_str(),
        ),
    );
}

#[test]
fn invalid_input_carries_typed_field_and_validation_reason() {
    use cow_sdk_core::ValidationReason;

    let error = TradingError::InvalidInput {
        field: "buyAmount",
        reason: ValidationReason::OutOfRange {
            details: "buyAmount must be greater than 0",
        },
    };

    let TradingError::InvalidInput { field, reason } = &error else {
        panic!("expected InvalidInput variant, got {error:?}");
    };
    assert_eq!(*field, "buyAmount");
    assert!(matches!(
        reason,
        ValidationReason::OutOfRange { details } if *details == "buyAmount must be greater than 0"
    ));

    let rendered = error.to_string();
    assert!(
        rendered.contains("buyAmount") && rendered.contains("out of range"),
        "InvalidInput must render the field and the validation reason, got: {rendered}",
    );
}
