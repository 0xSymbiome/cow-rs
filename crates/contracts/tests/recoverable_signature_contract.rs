//! Curated contract suite for [`RecoverableSignature`].
//!
//! Pins the strict ADR 0022 input surface, the canonical legacy byte
//! shape, the round-trip behaviour, the ERC-2098 compact-form bridge,
//! the opt-in low-s canonicalisation, and the scheme-bundled recovery
//! API. Together with the parity fixture rows in
//! `parity/fixtures/ecdsa/v_normalization.json`, this file is the
//! current-state proof for the `RecoverableSignature` public surface.
//!
//! The accept set is `{0, 1, 27, 28}`; every other trailing byte is
//! rejected through the typed
//! [`ContractsError::InvalidSignatureRecoveryByte`] variant, including
//! the EIP-155 chain-encoded range `35..=255` that
//! [`alloy_primitives::Signature::from_raw`] would otherwise admit.

use alloy_primitives::Address as AlloyAddress;
use cow_sdk_contracts::{ContractsError, RecoverableSignature, SigningScheme};
use cow_sdk_core::{Address, Hash32};
use k256::ecdsa::SigningKey;
use sha3::{Digest, Keccak256};

fn deterministic_signing_key() -> SigningKey {
    SigningKey::from_slice(
        &alloy_primitives::hex::decode(
            "4c0883a69102937d6231471b5dbb6204fe5129617082792ae468d01a3f362318",
        )
        .unwrap(),
    )
    .unwrap()
}

fn expected_address_for_key(signing_key: &SigningKey) -> Address {
    Address::new(AlloyAddress::from_private_key(signing_key).to_string()).unwrap()
}

fn ecdsa_signature_for_prehash(signing_key: &SigningKey, prehash: &[u8; 32]) -> Vec<u8> {
    let (signature, recovery_id) = signing_key.sign_prehash_recoverable(prehash).unwrap();
    let mut bytes = Vec::with_capacity(65);
    bytes.extend_from_slice(signature.to_bytes().as_slice());
    bytes.push(recovery_id.to_byte() + 27);
    bytes
}

fn eip191_prehash_of_digest(digest_bytes: &[u8; 32]) -> [u8; 32] {
    let mut payload = Vec::with_capacity(60);
    payload.extend_from_slice(b"\x19Ethereum Signed Message:\n32");
    payload.extend_from_slice(digest_bytes);
    Keccak256::digest(payload).into()
}

fn synthetic_bytes(v: u8) -> [u8; 65] {
    let mut bytes = [0u8; 65];
    for (i, b) in bytes.iter_mut().enumerate().take(64) {
        let index = u8::try_from(i).expect("index must fit in u8 (loop bound is 64)");
        *b = index.wrapping_mul(17).wrapping_add(1);
    }
    bytes[64] = v;
    bytes
}

#[test]
fn parse_bytes_accepts_the_canonical_v_set_and_emits_legacy_form() {
    for &input_v in &[0u8, 1, 27, 28] {
        let bytes = synthetic_bytes(input_v);
        let sig = RecoverableSignature::parse_bytes(&bytes).unwrap();
        let out = sig.to_bytes();
        assert_eq!(
            &out[..64],
            &bytes[..64],
            "r || s must be preserved byte-identically for v = {input_v}",
        );
        assert!(
            matches!(out[64], 27 | 28),
            "output v must be in {{27, 28}}, got {} for input v = {input_v}",
            out[64],
        );
        let expected_out_v = match input_v {
            0 | 27 => 27,
            1 | 28 => 28,
            _ => unreachable!(),
        };
        assert_eq!(out[64], expected_out_v);
    }
}

#[test]
fn parse_bytes_rejects_every_recovery_byte_outside_the_canonical_set() {
    for invalid_v in (2u8..27).chain(29u8..=255) {
        let bytes = synthetic_bytes(invalid_v);
        let error = RecoverableSignature::parse_bytes(&bytes).unwrap_err();
        match error {
            ContractsError::InvalidSignatureRecoveryByte { value } => {
                assert_eq!(value, invalid_v);
            }
            other => panic!("expected InvalidSignatureRecoveryByte for v = {invalid_v}, got {other:?}"),
        }
    }
}

#[test]
fn parse_bytes_rejects_eip155_chain_encoded_recovery_bytes() {
    // The EIP-155 transaction encoding mixes the chain id into v starting at
    // 35. The wider alloy parity-normalization path would silently accept
    // these values and map them onto {27, 28}. The contracts-boundary
    // surface rejects them so the typed error reaches the call site.
    for &eip155_v in &[35u8, 36, 37, 38, 50, 100, 199, 200, 254, 255] {
        let bytes = synthetic_bytes(eip155_v);
        let error = RecoverableSignature::parse_bytes(&bytes).unwrap_err();
        match error {
            ContractsError::InvalidSignatureRecoveryByte { value } => {
                assert_eq!(value, eip155_v);
            }
            other => panic!("expected InvalidSignatureRecoveryByte for v = {eip155_v}, got {other:?}"),
        }
    }
}

#[test]
fn parse_bytes_rejects_wrong_length() {
    let short = RecoverableSignature::parse_bytes(&[0u8; 64]).unwrap_err();
    assert!(matches!(
        short,
        ContractsError::InvalidSignatureLength { actual: 64 }
    ));

    let long = RecoverableSignature::parse_bytes(&[0u8; 66]).unwrap_err();
    assert!(matches!(
        long,
        ContractsError::InvalidSignatureLength { actual: 66 }
    ));

    let empty = RecoverableSignature::parse_bytes(&[]).unwrap_err();
    assert!(matches!(
        empty,
        ContractsError::InvalidSignatureLength { actual: 0 }
    ));
}

#[test]
fn parse_hex_rejects_envelope_failures() {
    let no_prefix = RecoverableSignature::parse_hex("abababab").unwrap_err();
    assert!(matches!(
        no_prefix,
        ContractsError::InvalidHexPrefix { field } if field == "signature"
    ));

    let invalid_hex_body =
        RecoverableSignature::parse_hex(&format!("0x{}", "z".repeat(130))).unwrap_err();
    assert!(matches!(
        invalid_hex_body,
        ContractsError::DecodeHex { field, source: _ } if field == "signature"
    ));
}

#[test]
fn parse_hex_to_hex_string_is_idempotent_on_accepted_inputs() {
    for &v in &[0u8, 1, 27, 28] {
        let bytes = synthetic_bytes(v);
        let first = RecoverableSignature::parse_bytes(&bytes).unwrap().to_hex_string();
        let second = RecoverableSignature::parse_hex(&first).unwrap().to_hex_string();
        assert_eq!(first, second, "to_hex_string must be idempotent for v = {v}");
    }
}

#[test]
fn to_hex_string_is_lowercase_and_prefixed() {
    let bytes = synthetic_bytes(27);
    let sig = RecoverableSignature::parse_bytes(&bytes).unwrap();
    let hex = sig.to_hex_string();
    assert!(hex.starts_with("0x"), "hex must keep the 0x prefix, got {hex}");
    assert_eq!(hex.len(), 132, "hex body must be 65 bytes encoded");
    assert!(
        hex[2..]
            .chars()
            .all(|c| c.is_ascii_digit() || c.is_ascii_lowercase()),
        "hex tail must be lowercase ASCII hex, got {hex}",
    );
}

#[test]
fn recover_eip712_round_trip_returns_the_signing_address() {
    let signing_key = deterministic_signing_key();
    let digest_bytes = [0x11u8; 32];
    let digest = Hash32::new(format!("0x{}", alloy_primitives::hex::encode(digest_bytes))).unwrap();

    let raw = ecdsa_signature_for_prehash(&signing_key, &digest_bytes);
    let sig = RecoverableSignature::parse_bytes(&raw).unwrap();

    let recovered = sig.recover(&digest, SigningScheme::Eip712).unwrap();
    assert_eq!(recovered, expected_address_for_key(&signing_key));
}

#[test]
fn recover_eth_sign_round_trip_applies_eip191_prehash_internally() {
    let signing_key = deterministic_signing_key();
    let digest_bytes = [0x22u8; 32];
    let digest = Hash32::new(format!("0x{}", alloy_primitives::hex::encode(digest_bytes))).unwrap();

    let prehash = eip191_prehash_of_digest(&digest_bytes);
    let raw = ecdsa_signature_for_prehash(&signing_key, &prehash);
    let sig = RecoverableSignature::parse_bytes(&raw).unwrap();

    let recovered = sig.recover(&digest, SigningScheme::EthSign).unwrap();
    assert_eq!(recovered, expected_address_for_key(&signing_key));
}

#[test]
fn recover_rejects_non_ecdsa_schemes() {
    let bytes = synthetic_bytes(27);
    let sig = RecoverableSignature::parse_bytes(&bytes).unwrap();
    let digest = Hash32::new(format!("0x{}", "11".repeat(32))).unwrap();

    let eip1271 = sig.recover(&digest, SigningScheme::Eip1271).unwrap_err();
    assert!(matches!(eip1271, ContractsError::SignatureSchemeNotEcdsa));

    let presign = sig.recover(&digest, SigningScheme::PreSign).unwrap_err();
    assert!(matches!(presign, ContractsError::SignatureSchemeNotEcdsa));
}

#[test]
fn canonicalized_low_s_is_idempotent_and_preserves_recovery() {
    let signing_key = deterministic_signing_key();
    let digest_bytes = [0x33u8; 32];
    let digest = Hash32::new(format!("0x{}", alloy_primitives::hex::encode(digest_bytes))).unwrap();

    let raw = ecdsa_signature_for_prehash(&signing_key, &digest_bytes);
    let sig = RecoverableSignature::parse_bytes(&raw).unwrap();

    let low_s_once = sig.canonicalized_low_s();
    let low_s_twice = low_s_once.canonicalized_low_s();
    assert_eq!(
        low_s_once.to_bytes(),
        low_s_twice.to_bytes(),
        "canonicalized_low_s must be idempotent",
    );

    let recovered = low_s_once.recover(&digest, SigningScheme::Eip712).unwrap();
    assert_eq!(recovered, expected_address_for_key(&signing_key));
}

#[test]
fn erc2098_round_trip_produces_the_same_signer() {
    let signing_key = deterministic_signing_key();
    let digest_bytes = [0x44u8; 32];
    let digest = Hash32::new(format!("0x{}", alloy_primitives::hex::encode(digest_bytes))).unwrap();

    let raw = ecdsa_signature_for_prehash(&signing_key, &digest_bytes);
    let sig = RecoverableSignature::parse_bytes(&raw).unwrap();

    let compact = sig.to_erc2098();
    assert_eq!(compact.len(), 64);

    let round_trip = RecoverableSignature::parse_erc2098(&compact).unwrap();
    let recovered = round_trip.recover(&digest, SigningScheme::Eip712).unwrap();
    assert_eq!(recovered, expected_address_for_key(&signing_key));
}

#[test]
fn parse_erc2098_rejects_wrong_length() {
    let short = RecoverableSignature::parse_erc2098(&[0u8; 63]).unwrap_err();
    assert!(matches!(
        short,
        ContractsError::InvalidSignatureLength { actual: 63 }
    ));

    let long = RecoverableSignature::parse_erc2098(&[0u8; 65]).unwrap_err();
    assert!(matches!(
        long,
        ContractsError::InvalidSignatureLength { actual: 65 }
    ));
}

#[test]
fn as_alloy_exposes_canonical_parity_to_the_inner_primitive() {
    // The cow wrapper hides the alloy primitive behind a borrow-only
    // escape hatch; reading the inner value must observe the canonical
    // parity computed by the strict v-set match in `parse_bytes`.
    let bytes_v0 = synthetic_bytes(0);
    let from_v0 = RecoverableSignature::parse_bytes(&bytes_v0).unwrap();
    assert!(!from_v0.as_alloy().v(), "v = 0 must canonicalize to parity false");

    let bytes_v1 = synthetic_bytes(1);
    let from_v1 = RecoverableSignature::parse_bytes(&bytes_v1).unwrap();
    assert!(from_v1.as_alloy().v(), "v = 1 must canonicalize to parity true");

    let bytes_v27 = synthetic_bytes(27);
    let from_v27 = RecoverableSignature::parse_bytes(&bytes_v27).unwrap();
    assert!(!from_v27.as_alloy().v(), "v = 27 must canonicalize to parity false");

    let bytes_v28 = synthetic_bytes(28);
    let from_v28 = RecoverableSignature::parse_bytes(&bytes_v28).unwrap();
    assert!(from_v28.as_alloy().v(), "v = 28 must canonicalize to parity true");
}
