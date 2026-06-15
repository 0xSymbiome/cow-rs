//! Conversion contract for the orderbook `SigningScheme` / `EcdsaSigningScheme`
//! pair.
//!
//! Order cancellations accept only the ECDSA signing schemes (`Eip712`,
//! `EthSign`), a strict subset of the four wire-form `SigningScheme` variants.
//! Two conversions express that relationship:
//!
//! * `From<EcdsaSigningScheme> for SigningScheme` widens a cancellation scheme
//!   to its wire-form parent.
//! * `TryFrom<SigningScheme> for EcdsaSigningScheme` narrows a wire-form scheme
//!   back to the ECDSA subset, rejecting `Eip1271` and `PreSign` with the typed
//!   [`SigningSchemeNotEcdsa`] error rather than silently dropping data.
//!
//! These tests pin every variant by name so a missing or reordered branch fails
//! loudly rather than mapping to an unrelated scheme.

use cow_sdk_orderbook::{EcdsaSigningScheme, SigningScheme, SigningSchemeNotEcdsa};

const ALL_ECDSA_VARIANTS: &[EcdsaSigningScheme] =
    &[EcdsaSigningScheme::Eip712, EcdsaSigningScheme::EthSign];

#[test]
fn ecdsa_widens_to_signing_scheme_variant_by_variant() {
    for &scheme in ALL_ECDSA_VARIANTS {
        let widened = SigningScheme::from(scheme);
        let expected = match scheme {
            EcdsaSigningScheme::Eip712 => SigningScheme::Eip712,
            EcdsaSigningScheme::EthSign => SigningScheme::EthSign,
            // SAFETY: cow_sdk_orderbook::EcdsaSigningScheme is `#[non_exhaustive]`; if a
            // new variant is added without updating this oracle the test fails loudly.
            _ => panic!("EcdsaSigningScheme gained a variant; update the conversion oracle"),
        };
        assert_eq!(
            widened, expected,
            "EcdsaSigningScheme->SigningScheme drift for {scheme:?}"
        );
    }
}

#[test]
fn ecdsa_round_trip_through_signing_scheme_is_identity() {
    for &scheme in ALL_ECDSA_VARIANTS {
        let round_tripped = EcdsaSigningScheme::try_from(SigningScheme::from(scheme))
            .expect("ECDSA variants always narrow back");
        assert_eq!(round_tripped, scheme, "round-trip drift for {scheme:?}");
    }
}

#[test]
fn try_from_signing_scheme_errors_on_eip1271() {
    let err = EcdsaSigningScheme::try_from(SigningScheme::Eip1271)
        .expect_err("Eip1271 must not narrow to ECDSA");
    assert_eq!(err, SigningSchemeNotEcdsa(SigningScheme::Eip1271));
}

#[test]
fn try_from_signing_scheme_errors_on_pre_sign() {
    let err = EcdsaSigningScheme::try_from(SigningScheme::PreSign)
        .expect_err("PreSign must not narrow to ECDSA");
    assert_eq!(err, SigningSchemeNotEcdsa(SigningScheme::PreSign));
}
