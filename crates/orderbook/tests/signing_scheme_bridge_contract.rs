//! Bridge parity test for the cow-sdk-contracts ↔ cow-sdk-orderbook
//! `SigningScheme` enums.
//!
//! The two enums share the same four variants (`Eip712`, `EthSign`, `Eip1271`,
//! `PreSign`) but their on-the-wire representations diverge:
//!
//! * `cow_sdk_contracts::SigningScheme` is `#[repr(u8)]` and matches the
//!   protocol u8 discriminant.
//! * `cow_sdk_orderbook::SigningScheme` is serialised with
//!   `#[serde(rename_all = "lowercase")]` to match the orderbook HTTP wire
//!   form (`"eip712"`, `"ethsign"`, `"eip1271"`, `"presign"`).
//!
//! Per ADR 0052, the two enums stay distinct. A `From` / `TryFrom`
//! bridge connects them so callers can convert losslessly without paying
//! the orphan-rules tax. This test asserts the bridge is
//! variant-by-variant identity, the `EcdsaSigningScheme` narrowing
//! round-trips through the parent, and the fallible inverse rejects the
//! two non-ECDSA variants with the typed `SigningSchemeNotEcdsa` error.
//!
//! If the upstream contracts crate adds a new `SigningScheme` variant, the
//! orderbook crate must add the matching variant in the same patch (the
//! `From<ContractsSigningScheme>` impl uses `unreachable!` on unknown
//! variants per the bridge contract). This test pins every existing variant
//! by name so a missing branch fails the test rather than silently mapping
//! to an unrelated scheme.

use cow_sdk_contracts::SigningScheme as ContractsSigningScheme;
use cow_sdk_orderbook::{EcdsaSigningScheme, SigningScheme, SigningSchemeNotEcdsa};

const ALL_ORDERBOOK_VARIANTS: &[SigningScheme] = &[
    SigningScheme::Eip712,
    SigningScheme::EthSign,
    SigningScheme::Eip1271,
    SigningScheme::PreSign,
];

const ALL_CONTRACTS_VARIANTS: &[ContractsSigningScheme] = &[
    ContractsSigningScheme::Eip712,
    ContractsSigningScheme::EthSign,
    ContractsSigningScheme::Eip1271,
    ContractsSigningScheme::PreSign,
];

const ALL_ECDSA_VARIANTS: &[EcdsaSigningScheme] =
    &[EcdsaSigningScheme::Eip712, EcdsaSigningScheme::EthSign];

fn expected_contracts_for(scheme: SigningScheme) -> ContractsSigningScheme {
    match scheme {
        SigningScheme::Eip712 => ContractsSigningScheme::Eip712,
        SigningScheme::EthSign => ContractsSigningScheme::EthSign,
        SigningScheme::Eip1271 => ContractsSigningScheme::Eip1271,
        SigningScheme::PreSign => ContractsSigningScheme::PreSign,
        // SAFETY: cow_sdk_orderbook::SigningScheme is `#[non_exhaustive]`; if a new
        // variant is added without updating this oracle the parity test fails loudly.
        _ => panic!("orderbook SigningScheme gained a variant; update the bridge oracle"),
    }
}

fn expected_orderbook_for(scheme: ContractsSigningScheme) -> SigningScheme {
    match scheme {
        ContractsSigningScheme::Eip712 => SigningScheme::Eip712,
        ContractsSigningScheme::EthSign => SigningScheme::EthSign,
        ContractsSigningScheme::Eip1271 => SigningScheme::Eip1271,
        ContractsSigningScheme::PreSign => SigningScheme::PreSign,
        // SAFETY: cow_sdk_contracts::SigningScheme is `#[non_exhaustive]`; if a new
        // variant is added without updating this oracle the parity test fails loudly.
        _ => panic!("contracts SigningScheme gained a variant; update the bridge oracle"),
    }
}

#[test]
fn orderbook_to_contracts_maps_variant_by_variant() {
    for &scheme in ALL_ORDERBOOK_VARIANTS {
        assert_eq!(
            ContractsSigningScheme::from(scheme),
            expected_contracts_for(scheme),
            "orderbook→contracts mapping diverged for {scheme:?}",
        );
    }
}

#[test]
fn contracts_to_orderbook_maps_variant_by_variant() {
    for &scheme in ALL_CONTRACTS_VARIANTS {
        assert_eq!(
            SigningScheme::from(scheme),
            expected_orderbook_for(scheme),
            "contracts→orderbook mapping diverged for {scheme:?}",
        );
    }
}

#[test]
fn orderbook_round_trip_through_contracts_is_identity() {
    for &scheme in ALL_ORDERBOOK_VARIANTS {
        let round_tripped = SigningScheme::from(ContractsSigningScheme::from(scheme));
        assert_eq!(round_tripped, scheme, "round-trip drift for {scheme:?}");
    }
}

#[test]
fn contracts_round_trip_through_orderbook_is_identity() {
    for &scheme in ALL_CONTRACTS_VARIANTS {
        let round_tripped = ContractsSigningScheme::from(SigningScheme::from(scheme));
        assert_eq!(round_tripped, scheme, "round-trip drift for {scheme:?}");
    }
}

#[test]
fn ecdsa_to_orderbook_embeds_two_variants() {
    for &scheme in ALL_ECDSA_VARIANTS {
        let widened = SigningScheme::from(scheme);
        let expected = match scheme {
            EcdsaSigningScheme::Eip712 => SigningScheme::Eip712,
            EcdsaSigningScheme::EthSign => SigningScheme::EthSign,
            // SAFETY: cow_sdk_orderbook::EcdsaSigningScheme is `#[non_exhaustive]`; if a
            // new variant is added without updating this oracle the parity test fails.
            _ => panic!("EcdsaSigningScheme gained a variant; update the bridge oracle"),
        };
        assert_eq!(
            widened, expected,
            "EcdsaSigningScheme→SigningScheme drift for {scheme:?}"
        );
    }
}

#[test]
fn ecdsa_to_contracts_embeds_two_variants() {
    for &scheme in ALL_ECDSA_VARIANTS {
        let widened = ContractsSigningScheme::from(scheme);
        let expected = match scheme {
            EcdsaSigningScheme::Eip712 => ContractsSigningScheme::Eip712,
            EcdsaSigningScheme::EthSign => ContractsSigningScheme::EthSign,
            // SAFETY: cow_sdk_orderbook::EcdsaSigningScheme is `#[non_exhaustive]`; if a
            // new variant is added without updating this oracle the parity test fails.
            _ => panic!("EcdsaSigningScheme gained a variant; update the bridge oracle"),
        };
        assert_eq!(
            widened, expected,
            "EcdsaSigningScheme→ContractsSigningScheme drift for {scheme:?}"
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
