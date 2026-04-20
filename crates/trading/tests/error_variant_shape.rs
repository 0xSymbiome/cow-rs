//! Public-surface regression for the typed shape of
//! [`cow_sdk_trading::TradingError::RecoverableSignatureOwnerMismatch`].
//!
//! The variant previously carried stringly-typed `owner` and `signer`
//! payloads that lost information at construction time: both values
//! are always Ethereum addresses resolved from the order payload and
//! the signing backend respectively. The variant now carries typed
//! [`cow_sdk_core::Address`] fields so downstream callers can
//! pattern-match on the typed payload without re-parsing error
//! messages. This test destructures the new shape and asserts the
//! typed field types through an exhaustive pattern match; if the
//! variant regresses to a stringly-typed shape, this file fails to
//! compile.

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
