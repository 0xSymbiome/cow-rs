//! Extension traits that expose the cow-side identity primitive accessors
//! (`new`, `from_bytes`, `as_str`, etc.) on the canonical
//! [`alloy_primitives`] types.
//!
//! These traits are the forward-compatible foundation for the staged
//! collapse of the cow identity newtypes onto `alloy_primitives` per
//! ADR 0052. Today the cow newtypes ([`crate::Address`], [`crate::Hash32`],
//! [`crate::HexData`], [`crate::OrderUid`]) keep their own `String`-backed
//! storage and inherent methods; once a future stage retires those
//! newtypes in favour of `alloy_primitives` type aliases, callsites that
//! bring the extension traits into scope (typically via
//! [`crate::prelude`]) continue to resolve `Address::new(value)` style
//! constructors against the trait method exposed here.
//!
//! The traits are intentionally sealed so downstream crates cannot
//! implement them for unrelated types — the canonical wire-form
//! invariants belong to the cow primitive contract.
//!
//! # Wire-form invariants
//!
//! Every accessor preserves the lowercase `0x`-prefixed hexadecimal
//! string contract documented in `PROPERTIES.md` and exercised by
//! `crates/core/tests/wire_format_preservation_contract.rs`:
//!
//! - [`AddressExt::as_str`] emits a 42-character lowercase hex string
//!   (`0x` + 40 hex characters) regardless of the EIP-55 mixed-case
//!   checksum form the alloy `Display` impl would otherwise produce;
//! - [`Hash32Ext::as_str`] emits a 66-character lowercase hex string;
//! - [`OrderUidExt::as_str`] emits a 114-character lowercase hex string;
//! - [`HexDataExt::as_str`] emits a variable-length lowercase hex string
//!   with the `0x` prefix preserved across the round-trip.

use alloy_primitives::{Address as AlloyAddress, B256, Bytes, FixedBytes};

use crate::errors::{CoreError, ValidationError};
use crate::types::hex::{
    EVM_ADDRESS_HEX_CHARS, HASH32_HEX_CHARS, ORDER_UID_HEX_CHARS, hex_encode_20, hex_encode_32,
    hex_encode_56,
};

mod sealed {
    use alloy_primitives::{Address as AlloyAddress, B256, Bytes, FixedBytes};

    /// Sealed marker trait keeping the cow identity-extension traits
    /// implementable only by the canonical alloy primitive types.
    ///
    /// The trait is intentionally unnameable from outside this crate so the
    /// sealed-trait idiom holds; downstream crates cannot synthesise their
    /// own `Sealed` impls for foreign types, and the orphan rules already
    /// prevent them from impl-ing it for the canonical alloy primitives.
    #[allow(
        unnameable_types,
        reason = "Sealed trait pattern intentionally hides the marker; downstream impls are gated by orphan rules."
    )]
    pub trait Sealed {}

    impl Sealed for AlloyAddress {}
    impl Sealed for B256 {}
    impl Sealed for Bytes {}
    impl Sealed for FixedBytes<56> {}
}

/// Cow extension trait providing the canonical lowercase-hex accessors
/// for an [`alloy_primitives::Address`].
///
/// The trait mirrors the inherent methods that the cow [`crate::Address`]
/// newtype exposes today. Once the newtype collapses onto an
/// `alloy_primitives::Address` type alias in a future migration stage,
/// every consumer call to `Address::new(value)` or `address.as_str()`
/// continues to resolve against the trait method below, so the public
/// constructor and accessor names stay stable across the migration.
pub trait AddressExt: sealed::Sealed + Sized {
    /// Creates a validated address from a `0x`-prefixed 42-character
    /// hexadecimal string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed,
    /// has the wrong length, or contains non-hex characters.
    fn new(value: impl AsRef<str>) -> Result<Self, CoreError>;

    /// Creates an address from its raw 20-byte representation.
    fn from_bytes(bytes: [u8; 20]) -> Self;

    /// Returns the canonical lowercase `0x`-prefixed 42-character hex
    /// string. Allocates a fresh [`String`] because the canonical
    /// storage is the raw 20-byte representation.
    fn as_str(&self) -> String;

    /// Returns the raw 20 address bytes.
    fn as_slice(&self) -> &[u8];

    /// Returns the fixed 20-byte decoded length of an EVM address.
    fn byte_length(&self) -> usize {
        EVM_ADDRESS_HEX_CHARS / 2
    }

    /// Returns the lowercase `0x`-prefixed hex form used for the
    /// documented case-insensitive comparison contract.
    fn normalized_key(&self) -> String {
        Self::as_str(self)
    }
}

impl AddressExt for AlloyAddress {
    fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let raw = value.as_ref();
        validate_hex_field("address", raw, EVM_ADDRESS_HEX_CHARS)?;
        let stripped = raw
            .strip_prefix("0x")
            .ok_or(ValidationError::InvalidHexPrefix { field: "address" })?;
        let bytes = hex::decode(stripped)
            .map_err(|_| ValidationError::InvalidHexCharacters { field: "address" })?;
        let array: [u8; 20] = bytes
            .try_into()
            .map_err(|_| ValidationError::InvalidHexLength {
                field: "address",
                expected: EVM_ADDRESS_HEX_CHARS,
            })?;
        Ok(Self::from(array))
    }

    fn from_bytes(bytes: [u8; 20]) -> Self {
        Self::from(bytes)
    }

    /// Returns the canonical lowercase `0x`-prefixed 42-character hex string.
    ///
    /// # Panics
    ///
    /// Panics only if `hex_encode_20` ever emits non-ASCII bytes. The encoder
    /// shipped in this crate writes exclusively ASCII hex digits plus the `0x`
    /// prefix, so this panic cannot be reached from an unmodified binary.
    fn as_str(&self) -> String {
        let hex_bytes = hex_encode_20(self.0.0);
        // SAFETY: hex_encode_20 only emits ASCII hex characters plus the 0x
        // prefix, which is valid UTF-8 by construction.
        String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_20 only emits ASCII hex characters plus the 0x prefix")
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

/// Cow extension trait providing the canonical lowercase-hex accessors
/// for an [`alloy_primitives::B256`] (32-byte hash).
pub trait Hash32Ext: sealed::Sealed + Sized {
    /// Creates a validated 32-byte hash from a `0x`-prefixed 66-character
    /// hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed,
    /// has the wrong length, or contains non-hex characters.
    fn new(value: impl AsRef<str>) -> Result<Self, CoreError>;

    /// Creates a 32-byte hash from its raw 32-byte representation.
    fn from_bytes(bytes: [u8; 32]) -> Self;

    /// Returns the canonical lowercase `0x`-prefixed 66-character hex
    /// string.
    fn as_str(&self) -> String;

    /// Returns the raw 32 hash bytes.
    fn as_slice(&self) -> &[u8];

    /// Returns the fixed 32-byte decoded length of a 32-byte hash.
    fn byte_length(&self) -> usize {
        HASH32_HEX_CHARS / 2
    }
}

impl Hash32Ext for B256 {
    fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let raw = value.as_ref();
        validate_hex_field("hash32", raw, HASH32_HEX_CHARS)?;
        let stripped = raw
            .strip_prefix("0x")
            .ok_or(ValidationError::InvalidHexPrefix { field: "hash32" })?;
        let bytes = hex::decode(stripped)
            .map_err(|_| ValidationError::InvalidHexCharacters { field: "hash32" })?;
        let array: [u8; 32] = bytes
            .try_into()
            .map_err(|_| ValidationError::InvalidHexLength {
                field: "hash32",
                expected: HASH32_HEX_CHARS,
            })?;
        Ok(Self::from(array))
    }

    fn from_bytes(bytes: [u8; 32]) -> Self {
        Self::from(bytes)
    }

    /// Returns the canonical lowercase `0x`-prefixed 66-character hex string.
    ///
    /// # Panics
    ///
    /// Panics only if `hex_encode_32` ever emits non-ASCII bytes. The encoder
    /// shipped in this crate writes exclusively ASCII hex digits plus the `0x`
    /// prefix, so this panic cannot be reached from an unmodified binary.
    fn as_str(&self) -> String {
        let hex_bytes = hex_encode_32(self.0);
        // SAFETY: hex_encode_32 only emits ASCII hex characters plus the 0x
        // prefix, which is valid UTF-8 by construction.
        String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_32 only emits ASCII hex characters plus the 0x prefix")
    }

    fn as_slice(&self) -> &[u8] {
        AsRef::<[u8]>::as_ref(self)
    }
}

/// Cow extension trait providing the canonical lowercase-hex accessors
/// for an [`alloy_primitives::Bytes`] (variable-length hex payload).
pub trait HexDataExt: sealed::Sealed + Sized {
    /// Creates a validated hex payload from a `0x`-prefixed hex string.
    ///
    /// Odd-length payloads left-pad with one zero nibble so the stored
    /// value remains canonical byte-aligned hex.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed,
    /// or contains non-hex characters.
    fn new(value: impl AsRef<str>) -> Result<Self, CoreError>;

    /// Returns the canonical empty payload (`0x`).
    fn empty() -> Self;

    /// Returns the canonical lowercase `0x`-prefixed hex string.
    fn as_str(&self) -> String;

    /// Returns the decoded byte length of the payload.
    fn byte_length(&self) -> usize;
}

impl HexDataExt for Bytes {
    /// Creates a validated hex payload from a `0x`-prefixed hex string.
    ///
    /// Odd-length payloads are left-padded with one zero nibble so the stored
    /// value remains canonical byte-aligned hex.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, or
    /// contains non-hex characters.
    ///
    /// # Panics
    ///
    /// Panics only if `normalize_hex_payload` ever returns a string without
    /// the canonical `0x` prefix. The shipped implementation re-prefixes
    /// every accepted payload before returning, so this panic cannot be
    /// reached from an unmodified binary.
    fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let raw = value.as_ref();
        let normalized = normalize_hex_payload("hex_data", raw)?;
        // SAFETY: normalize_hex_payload preserves the leading 0x prefix on
        // every successfully validated payload.
        let stripped = normalized
            .strip_prefix("0x")
            .expect("normalize_hex_payload preserves the 0x prefix");
        let bytes = hex::decode(stripped)
            .map_err(|_| ValidationError::InvalidHexCharacters { field: "hex_data" })?;
        Ok(Self::from(bytes))
    }

    fn empty() -> Self {
        Self::from(Vec::<u8>::new())
    }

    fn as_str(&self) -> String {
        format!("0x{}", hex::encode(self.as_ref()))
    }

    fn byte_length(&self) -> usize {
        self.len()
    }
}

/// Cow extension trait providing the canonical lowercase-hex accessors
/// for a 56-byte [`alloy_primitives::FixedBytes`] order UID.
pub trait OrderUidExt: sealed::Sealed + Sized {
    /// Creates a validated order UID from a `0x`-prefixed 114-character
    /// hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed,
    /// has the wrong length, or contains non-hex characters.
    fn new(value: impl AsRef<str>) -> Result<Self, CoreError>;

    /// Creates an order UID from its raw 56-byte representation.
    fn from_bytes(bytes: [u8; 56]) -> Self;

    /// Returns the canonical lowercase `0x`-prefixed 114-character hex
    /// string.
    fn as_str(&self) -> String;

    /// Returns the raw 56 order-UID bytes.
    fn as_slice(&self) -> &[u8];

    /// Returns the fixed 56-byte decoded length of an order UID.
    fn byte_length(&self) -> usize {
        ORDER_UID_HEX_CHARS / 2
    }
}

impl OrderUidExt for FixedBytes<56> {
    fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let raw = value.as_ref();
        validate_hex_field("order_uid", raw, ORDER_UID_HEX_CHARS)?;
        let stripped = raw
            .strip_prefix("0x")
            .ok_or(ValidationError::InvalidHexPrefix { field: "order_uid" })?;
        let bytes = hex::decode(stripped)
            .map_err(|_| ValidationError::InvalidHexCharacters { field: "order_uid" })?;
        let array: [u8; 56] = bytes
            .try_into()
            .map_err(|_| ValidationError::InvalidHexLength {
                field: "order_uid",
                expected: ORDER_UID_HEX_CHARS,
            })?;
        Ok(Self::from(array))
    }

    fn from_bytes(bytes: [u8; 56]) -> Self {
        Self::from(bytes)
    }

    /// Returns the canonical lowercase `0x`-prefixed 114-character hex string.
    ///
    /// # Panics
    ///
    /// Panics only if `hex_encode_56` ever emits non-ASCII bytes. The encoder
    /// shipped in this crate writes exclusively ASCII hex digits plus the `0x`
    /// prefix, so this panic cannot be reached from an unmodified binary.
    fn as_str(&self) -> String {
        let hex_bytes = hex_encode_56(self.0);
        // SAFETY: hex_encode_56 only emits ASCII hex characters plus the 0x
        // prefix, which is valid UTF-8 by construction.
        String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_56 only emits ASCII hex characters plus the 0x prefix")
    }

    fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }
}

fn validate_hex_field(
    field: &'static str,
    value: &str,
    expected_hex_chars: usize,
) -> Result<(), CoreError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyField { field }.into());
    }

    let Some(hex_data) = value.strip_prefix("0x") else {
        return Err(ValidationError::InvalidHexPrefix { field }.into());
    };

    if hex_data.len() != expected_hex_chars {
        return Err(ValidationError::InvalidHexLength {
            field,
            expected: expected_hex_chars,
        }
        .into());
    }

    if hex::decode(hex_data).is_err() {
        return Err(ValidationError::InvalidHexCharacters { field }.into());
    }

    Ok(())
}

fn normalize_hex_payload(field: &'static str, value: &str) -> Result<String, CoreError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyField { field }.into());
    }

    let Some(hex_data) = value.strip_prefix("0x") else {
        return Err(ValidationError::InvalidHexPrefix { field }.into());
    };

    let normalized = if hex_data.len() % 2 == 1 {
        format!("0x0{hex_data}")
    } else {
        value.to_owned()
    };

    if hex::decode(normalized.trim_start_matches("0x")).is_err() {
        return Err(ValidationError::InvalidHexCharacters { field }.into());
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ADDRESS_HEX: &str = "0x6810e776880c02933d47db1b9fc05908e5386b96";
    const HASH32_HEX: &str = "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const ORDER_UID_HEX: &str = "0x59920c85de0162e9e55df8d396e75f3b6b7c2dfdb535f03e5c807731c31585eaff714b8b0e2700303ec912bd40496c3997ceea2b616d6710";

    #[test]
    fn address_ext_round_trips_canonical_lowercase_hex() {
        let address = <AlloyAddress as AddressExt>::new(ADDRESS_HEX).unwrap();
        assert_eq!(AddressExt::as_str(&address), ADDRESS_HEX);
        assert_eq!(AddressExt::as_slice(&address).len(), 20);
        assert_eq!(AddressExt::byte_length(&address), 20);
        assert_eq!(AddressExt::normalized_key(&address), ADDRESS_HEX);
    }

    #[test]
    fn address_ext_rejects_malformed_inputs() {
        assert!(<AlloyAddress as AddressExt>::new("").is_err());
        assert!(<AlloyAddress as AddressExt>::new("not-hex").is_err());
        assert!(<AlloyAddress as AddressExt>::new("0x1234").is_err());
        assert!(<AlloyAddress as AddressExt>::new(ADDRESS_HEX.replace("0x", "")).is_err());
    }

    #[test]
    fn hash32_ext_round_trips_canonical_lowercase_hex() {
        let hash = <B256 as Hash32Ext>::new(HASH32_HEX).unwrap();
        assert_eq!(Hash32Ext::as_str(&hash), HASH32_HEX);
        assert_eq!(Hash32Ext::as_slice(&hash).len(), 32);
        assert_eq!(Hash32Ext::byte_length(&hash), 32);
    }

    #[test]
    fn hex_data_ext_round_trips_with_odd_length_padding() {
        let payload = <Bytes as HexDataExt>::new("0x123").unwrap();
        assert_eq!(HexDataExt::as_str(&payload), "0x0123");
        assert_eq!(HexDataExt::byte_length(&payload), 2);

        let empty = <Bytes as HexDataExt>::empty();
        assert_eq!(HexDataExt::as_str(&empty), "0x");
        assert_eq!(HexDataExt::byte_length(&empty), 0);
    }

    #[test]
    fn order_uid_ext_round_trips_canonical_lowercase_hex() {
        let uid = <FixedBytes<56> as OrderUidExt>::new(ORDER_UID_HEX).unwrap();
        assert_eq!(OrderUidExt::as_str(&uid), ORDER_UID_HEX);
        assert_eq!(OrderUidExt::as_slice(&uid).len(), 56);
        assert_eq!(OrderUidExt::byte_length(&uid), 56);
    }
}
