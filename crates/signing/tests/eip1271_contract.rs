#![cfg(not(target_arch = "wasm32"))]
#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::iter_on_single_items,
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::redundant_clone,
    clippy::too_many_lines,
    clippy::unnested_or_patterns,
    reason = "pedantic, nursery, and perf lints acceptable in test helper code"
)]

mod common;

use cow_sdk_contracts::{ContractsError, decode_eip1271_signature_data};
use cow_sdk_core::OrderKind;
use cow_sdk_signing::SigningError;
use cow_sdk_signing::eip1271_signature_payload;
use num_bigint::BigUint;
use sha3::{Digest, Keccak256};

use common::{fixture_case, sample_order, sample_signature};

#[test]
fn eip1271_payload_hashes_string_fields_before_tuple_encoding() {
    let order = sample_order();
    let signature = sample_signature("12");
    let payload = eip1271_signature_payload(&order, &signature).unwrap();
    let expected = independent_payload(&order, &signature);
    let case = fixture_case("signing-eip1271-encoding");

    assert_eq!(
        case["expected"]["string_fields_hashed"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap())
            .collect::<Vec<_>>(),
        vec!["kind", "sellTokenBalance", "buyTokenBalance"]
    );
    assert_eq!(payload, expected);
}

#[test]
fn eip1271_payload_changes_when_order_kind_changes() {
    let mut buy_order = sample_order();
    buy_order.kind = OrderKind::Buy;

    let sell_payload = eip1271_signature_payload(&sample_order(), &sample_signature("34")).unwrap();
    let buy_payload = eip1271_signature_payload(&buy_order, &sample_signature("34")).unwrap();

    assert_ne!(sell_payload, buy_payload);
}

#[test]
fn eip1271_signature_data_rejects_malformed_verifier_or_payload() {
    let malformed_verifier = "0x1234";
    let verifier_error = decode_eip1271_signature_data(malformed_verifier)
        .expect_err("compact EIP-1271 data must include a verifier address and payload");
    assert!(matches!(
        verifier_error,
        ContractsError::InvalidEip1271SignatureData
    ));

    let payload_error = eip1271_signature_payload(&sample_order(), "0x1234")
        .expect_err("EIP-1271 helper must reject malformed ECDSA payloads");
    assert!(matches!(
        payload_error,
        SigningError::Contracts(ContractsError::InvalidSignatureLength { actual: 2 })
    ));
}

fn independent_payload(order: &cow_sdk_core::UnsignedOrder, ecdsa_signature: &str) -> String {
    let mut encoded = Vec::new();
    encoded.extend_from_slice(&encode_address(order.sell_token.as_str()));
    encoded.extend_from_slice(&encode_address(order.buy_token.as_str()));
    encoded.extend_from_slice(&encode_address(order.receiver.as_str()));
    encoded.extend_from_slice(&encode_u256(&order.sell_amount.to_string()));
    encoded.extend_from_slice(&encode_u256(&order.buy_amount.to_string()));
    encoded.extend_from_slice(&encode_u32(order.valid_to));
    encoded.extend_from_slice(&parse_hex32(order.app_data.as_str()));
    encoded.extend_from_slice(&encode_u256(&order.fee_amount.to_string()));
    encoded.extend_from_slice(&keccak256(match order.kind {
        OrderKind::Buy => b"buy".as_slice(),
        OrderKind::Sell => b"sell".as_slice(),
    }));
    encoded.extend_from_slice(&encode_bool(order.partially_fillable));
    encoded.extend_from_slice(&keccak256(b"erc20"));
    encoded.extend_from_slice(&keccak256(b"erc20"));
    encoded.extend_from_slice(&encode_usize(32 * 13));

    let signature = hex::decode(ecdsa_signature.trim_start_matches("0x")).unwrap();
    encoded.extend_from_slice(&encode_usize(signature.len()));
    encoded.extend_from_slice(&signature);
    encoded.extend(std::iter::repeat_n(
        0u8,
        padded_len(signature.len()) - signature.len(),
    ));

    format!("0x{}", hex::encode(encoded))
}

// SAFETY: hand-rolled oracle that proves the production path via byte-identity.
// Production code uses `alloy_primitives::keccak256` per ADR 0052; this test
// helper deliberately exercises the underlying `sha3::Keccak256` backend so
// the parity assertions above are not tautological alloy-vs-alloy checks.
fn keccak256(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    let digest = Keccak256::digest(bytes.as_ref());
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    out
}

fn encode_address(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    let bytes = hex::decode(value.trim_start_matches("0x")).unwrap();
    out[12..].copy_from_slice(&bytes);
    out
}

fn encode_u32(value: u32) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[28..].copy_from_slice(&value.to_be_bytes());
    out
}

fn encode_bool(value: bool) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[31] = u8::from(value);
    out
}

fn encode_u256(value: &str) -> [u8; 32] {
    let parsed = BigUint::parse_bytes(value.as_bytes(), 10).unwrap();
    let bytes = parsed.to_bytes_be();
    let mut out = [0u8; 32];
    out[32 - bytes.len()..].copy_from_slice(&bytes);
    out
}

fn encode_usize(value: usize) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&(value as u64).to_be_bytes());
    out
}

fn parse_hex32(value: &str) -> [u8; 32] {
    let bytes = hex::decode(value.trim_start_matches("0x")).unwrap();
    let mut out = [0u8; 32];
    out.copy_from_slice(&bytes);
    out
}

fn padded_len(len: usize) -> usize {
    if len == 0 {
        0
    } else {
        ((len - 1) / 32 + 1) * 32
    }
}
