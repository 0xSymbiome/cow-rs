//! Public-surface contract assertion for the typed shape of
//! [`cow_sdk_trading::TradingError::ClientRejected`] and the typed
//! [`cow_sdk_trading::ClientRejection`] surface it carries.
//!
//! Each variant of [`ClientRejection`] carries typed fields so downstream
//! callers pattern-match on the typed payload without re-parsing error
//! messages. The recoverable-signature owner check has merged into the
//! typed [`ClientRejection::OwnerMismatch`] variant and this test proves
//! the typed addresses still survive the round-trip.

use cow_sdk_core::Address;
use cow_sdk_trading::{ClientRejection, TradingError};

const fn assert_typed_address(_: &Address) {}

#[test]
fn client_rejection_owner_mismatch_carries_typed_addresses() {
    let owner =
        Address::new("0x1111111111111111111111111111111111111111").expect("literal must parse");
    let signer =
        Address::new("0x2222222222222222222222222222222222222222").expect("literal must parse");
    let rejection = ClientRejection::OwnerMismatch {
        expected: owner.clone(),
        recovered: signer.clone(),
    };
    let error: TradingError = rejection.into();

    let TradingError::ClientRejected(ClientRejection::OwnerMismatch {
        expected,
        recovered,
    }) = &error
    else {
        panic!("expected ClientRejected(OwnerMismatch) variant, got {error:?}");
    };
    assert_typed_address(expected);
    assert_typed_address(recovered);
    assert_eq!(expected, &owner);
    assert_eq!(recovered, &signer);

    let rendered = error.to_string();
    assert!(
        rendered.contains(owner.as_str()) && rendered.contains(signer.as_str()),
        "ClientRejection::OwnerMismatch must render the typed addresses, got: {rendered}",
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
