//! Independent EIP-712 / ABI-word oracle for hashing-parity tests.
//!
//! It is deliberately self-contained — it does **not** route through the
//! production `cow_sdk_contracts::encode_address_word`, because the whole point
//! of an oracle is to be an *independent* re-implementation that cross-checks
//! the production hashing path.

use std::str::FromStr;

use alloy_primitives::U256;
use sha3::{Digest, Keccak256};

/// Decodes a `0x`-optional hex string and asserts it is exactly
/// `expected_len` bytes.
///
/// # Panics
/// Panics if `value` is not valid hex or is not `expected_len` bytes long.
fn parse_hex_word(value: &str, expected_len: usize) -> Vec<u8> {
    let bytes = alloy_primitives::hex::decode(value.trim_start_matches("0x"))
        .expect("test oracle hex value must decode");
    assert_eq!(bytes.len(), expected_len);
    bytes
}

/// Encodes a 20-byte address into a right-aligned 32-byte ABI word.
///
/// # Panics
/// Panics if `value` is not a valid 20-byte hex string.
#[must_use]
pub fn encode_address_word(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[12..].copy_from_slice(&parse_hex_word(value, 20));
    out
}

/// Encodes a 32-byte value into a 32-byte ABI word.
///
/// # Panics
/// Panics if `value` is not a valid 32-byte hex string.
#[must_use]
pub fn encode_bytes32_word(value: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(&parse_hex_word(value, 32));
    out
}

/// Encodes a `u32` into a right-aligned 32-byte ABI word.
#[must_use]
pub fn encode_u32_word(value: u32) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[28..].copy_from_slice(&value.to_be_bytes());
    out
}

/// Encodes a decimal- or `0x`-hex-formatted integer into a 32-byte ABI word.
///
/// # Panics
/// Panics if `value` does not parse as a `U256`.
#[must_use]
pub fn encode_u256_word(value: &str) -> [u8; 32] {
    U256::from_str(value)
        .expect("oracle U256 value must parse")
        .to_be_bytes::<32>()
}

/// Encodes a `usize` (widened to `u64`) into a 32-byte ABI word.
#[must_use]
pub fn encode_usize_word(value: usize) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[24..].copy_from_slice(&(value as u64).to_be_bytes());
    out
}

/// Encodes a `bool` into a 32-byte ABI word.
#[must_use]
pub fn encode_bool_word(value: bool) -> [u8; 32] {
    let mut out = [0u8; 32];
    out[31] = u8::from(value);
    out
}

/// `keccak256` of a string's UTF-8 bytes (the EIP-712 dynamic-field hash).
#[must_use]
pub fn keccak_word(value: &str) -> [u8; 32] {
    keccak256(value.as_bytes())
}

/// `keccak256` of an arbitrary byte slice.
#[must_use]
pub fn keccak256(bytes: impl AsRef<[u8]>) -> [u8; 32] {
    let mut out = [0u8; 32];
    out.copy_from_slice(&Keccak256::digest(bytes.as_ref()));
    out
}
