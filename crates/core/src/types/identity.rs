use std::fmt;

use alloy_primitives::{Address as AlloyAddress, B256, Bytes, FixedBytes};
use serde::{Deserialize, Serialize};

use super::hex::{
    APP_DATA_HASH_HEX_CHARS, EVM_ADDRESS_HEX_CHARS, HASH32_HEX_CHARS, ORDER_UID_HEX_CHARS,
    hex_encode_20, hex_encode_32, hex_encode_56,
};
use crate::errors::{CoreError, ValidationError};

/// Numeric EVM chain id.
pub type ChainId = u64;

/// Validated EVM address.
///
/// The wire form is the protocol-canonical `0x`-prefixed 42-character
/// hexadecimal string. The struct stores the original input casing on the
/// [`Address::as_str`] surface so existing call sites keep their exact byte
/// view through the `&str` accessor, while the canonical alloy
/// [`alloy_primitives::Address`] primitive (a packed 20-byte representation)
/// is exposed through [`Address::as_alloy`] for cross-crate interop with the
/// `alloy_primitives`-typed signing, contract, and provider seams.
///
/// [`PartialEq`], [`Eq`], [`Hash`](std::hash::Hash), [`PartialOrd`], and
/// [`Ord`] compare addresses on their packed 20-byte representation, which is
/// equivalent to the documented case-insensitive comparison contract because
/// every valid address parses to the same bytes regardless of input casing.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Address {
    inner: AlloyAddress,
    hex: String,
}

impl Address {
    /// Creates a validated address from a `0x`-prefixed hexadecimal string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("address", &value, EVM_ADDRESS_HEX_CHARS)?;
        let stripped = value
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
        Ok(Self {
            inner: AlloyAddress::from(array),
            hex: value,
        })
    }

    /// Creates an address from its raw 20-byte representation.
    ///
    /// The input bytes are encoded into the canonical lowercase hex form
    /// without re-validating the character set, so this path is intended for
    /// protocol constants and other inputs whose byte-level shape is already
    /// known to the caller.
    ///
    /// # Panics
    ///
    /// Never panics in practice; the internal UTF-8 assertion exists only to
    /// keep the constructor free of `unsafe` code while still asserting the
    /// ASCII-only invariant produced by [`hex_encode_20`].
    #[inline]
    #[must_use]
    pub fn from_bytes(bytes: [u8; 20]) -> Self {
        let hex_bytes = hex_encode_20(bytes);
        // SAFETY: hex_encode_20 emits only the ASCII bytes `0`, `x`, and
        // lowercase hex digits.
        let hex = String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_20 only emits valid ASCII hex characters plus the 0x prefix");
        Self {
            inner: AlloyAddress::from(bytes),
            hex,
        }
    }

    /// Returns the original address string.
    ///
    /// The stored string preserves the input casing exactly; equality and
    /// hashing operate on the packed 20-byte representation instead.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.hex
    }

    /// Returns the stored hex string as a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.hex.as_bytes()
    }

    /// Returns the underlying packed [`alloy_primitives::Address`].
    ///
    /// Use this accessor when handing the address to an `alloy_primitives`-
    /// typed surface (signing, contract bindings, RPC adapters) without
    /// re-parsing the lowercase hex string.
    #[inline]
    #[must_use]
    pub const fn as_alloy(&self) -> &AlloyAddress {
        &self.inner
    }

    /// Returns the fixed decoded byte length of an EVM address.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        EVM_ADDRESS_HEX_CHARS / 2
    }

    /// Returns the lowercase key form used for case-insensitive comparisons.
    #[inline]
    #[must_use]
    pub fn normalized_key(&self) -> String {
        self.hex.to_ascii_lowercase()
    }
}

impl PartialEq for Address {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Address {}

impl PartialOrd for Address {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Address {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl std::hash::Hash for Address {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl TryFrom<String> for Address {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Address {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<Address> for String {
    fn from(value: Address) -> Self {
        value.hex
    }
}

impl AsRef<str> for Address {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validated hex payload used for calldata and byte blobs.
///
/// The wire form is the protocol-canonical `0x`-prefixed lowercase
/// hexadecimal string. The struct caches the input string on the
/// [`HexData::as_str`] surface and exposes the canonical
/// [`alloy_primitives::Bytes`] primitive through [`HexData::as_alloy`] for
/// cross-crate interop.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct HexData {
    inner: Bytes,
    hex: String,
}

impl HexData {
    /// Creates validated hex data from a `0x`-prefixed hexadecimal string.
    ///
    /// Odd-length payloads are left-padded with one zero nibble so the stored
    /// value remains canonical byte-aligned hex.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, or
    /// contains non-hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = normalize_hex_payload("hex_data", &value.into())?;
        let stripped = value
            .strip_prefix("0x")
            .ok_or(ValidationError::InvalidHexPrefix { field: "hex_data" })?;
        let bytes = hex::decode(stripped)
            .map_err(|_| ValidationError::InvalidHexCharacters { field: "hex_data" })?;
        Ok(Self {
            inner: Bytes::from(bytes),
            hex: value,
        })
    }

    /// Returns the canonical empty payload.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            inner: Bytes::new(),
            hex: "0x".to_owned(),
        }
    }

    /// Returns the original hex string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.hex
    }

    /// Returns the stored hex string as a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.hex.as_bytes()
    }

    /// Returns the underlying [`alloy_primitives::Bytes`] payload.
    #[inline]
    #[must_use]
    pub const fn as_alloy(&self) -> &Bytes {
        &self.inner
    }

    /// Returns the decoded byte length of the payload.
    #[inline]
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.inner.len()
    }
}

impl Default for HexData {
    fn default() -> Self {
        Self::empty()
    }
}

impl PartialEq for HexData {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for HexData {}

impl PartialOrd for HexData {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HexData {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl std::hash::Hash for HexData {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl TryFrom<String> for HexData {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for HexData {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<HexData> for String {
    fn from(value: HexData) -> Self {
        value.hex
    }
}

impl AsRef<str> for HexData {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for HexData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Validated 32-byte app-data hash.
///
/// The wire form is the protocol-canonical `0x`-prefixed 66-character
/// lowercase hexadecimal string. The struct caches the input string and
/// exposes the canonical [`alloy_primitives::B256`] primitive through
/// [`AppDataHash::as_alloy`] for cross-crate interop.
#[doc(alias = "app-data")]
#[doc(alias = "AppData")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AppDataHash {
    inner: B256,
    hex: String,
}

impl AppDataHash {
    /// Creates a validated app-data hash from a `0x`-prefixed 32-byte hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("app_data_hash", &value, APP_DATA_HASH_HEX_CHARS)?;
        let stripped = value
            .strip_prefix("0x")
            .ok_or(ValidationError::InvalidHexPrefix {
                field: "app_data_hash",
            })?;
        let bytes = hex::decode(stripped).map_err(|_| ValidationError::InvalidHexCharacters {
            field: "app_data_hash",
        })?;
        let array: [u8; 32] = bytes
            .try_into()
            .map_err(|_| ValidationError::InvalidHexLength {
                field: "app_data_hash",
                expected: APP_DATA_HASH_HEX_CHARS,
            })?;
        Ok(Self {
            inner: B256::from(array),
            hex: value,
        })
    }

    /// Creates an app-data hash from its raw 32-byte representation.
    ///
    /// The input bytes are encoded into the canonical lowercase hex form
    /// without re-validating the character set, so this path is intended for
    /// protocol constants and other inputs whose byte-level shape is already
    /// known to the caller.
    ///
    /// # Panics
    ///
    /// Never panics in practice; the internal UTF-8 assertion exists only to
    /// keep the constructor free of `unsafe` code while still asserting the
    /// ASCII-only invariant produced by [`hex_encode_32`].
    #[inline]
    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        let hex_bytes = hex_encode_32(bytes);
        // SAFETY: hex_encode_32 emits only the ASCII bytes `0`, `x`, and
        // lowercase hex digits.
        let hex = String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_32 only emits valid ASCII hex characters plus the 0x prefix");
        Self {
            inner: B256::from(bytes),
            hex,
        }
    }

    /// Returns the original hash string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.hex
    }

    /// Returns the stored hex string as a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.hex.as_bytes()
    }

    /// Returns the underlying [`alloy_primitives::B256`] hash.
    #[inline]
    #[must_use]
    pub const fn as_alloy(&self) -> &B256 {
        &self.inner
    }

    /// Returns the fixed decoded byte length of an app-data hash.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        APP_DATA_HASH_HEX_CHARS / 2
    }
}

impl PartialEq for AppDataHash {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for AppDataHash {}

impl PartialOrd for AppDataHash {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AppDataHash {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl std::hash::Hash for AppDataHash {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl TryFrom<String> for AppDataHash {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for AppDataHash {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<AppDataHash> for String {
    fn from(value: AppDataHash) -> Self {
        value.hex
    }
}

impl AsRef<str> for AppDataHash {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AppDataHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Backward-compatible alias for the app-data hash hex representation.
pub type AppDataHex = AppDataHash;

/// Generic validated 32-byte hash wrapper for user-domain and contract surfaces.
///
/// The wire form is the protocol-canonical `0x`-prefixed 66-character
/// lowercase hexadecimal string. The struct caches the input string and
/// exposes the canonical [`alloy_primitives::B256`] primitive through
/// [`Hash32::as_alloy`] for cross-crate interop.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hash32 {
    inner: B256,
    hex: String,
}

impl Hash32 {
    /// Creates a validated 32-byte hash from a `0x`-prefixed hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("hash32", &value, HASH32_HEX_CHARS)?;
        let stripped = value
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
        Ok(Self {
            inner: B256::from(array),
            hex: value,
        })
    }

    /// Creates a 32-byte hash from its raw 32-byte representation.
    ///
    /// The input bytes are encoded into the canonical lowercase hex form
    /// without re-validating the character set, so this path is intended for
    /// protocol constants and other inputs whose byte-level shape is already
    /// known to the caller.
    ///
    /// # Panics
    ///
    /// Never panics in practice; the internal UTF-8 assertion exists only to
    /// keep the constructor free of `unsafe` code while still asserting the
    /// ASCII-only invariant produced by [`hex_encode_32`].
    #[inline]
    #[must_use]
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        let hex_bytes = hex_encode_32(bytes);
        // SAFETY: hex_encode_32 emits only the ASCII bytes `0`, `x`, and
        // lowercase hex digits.
        let hex = String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_32 only emits valid ASCII hex characters plus the 0x prefix");
        Self {
            inner: B256::from(bytes),
            hex,
        }
    }

    /// Returns the original hash string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.hex
    }

    /// Returns the stored hex string as a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.hex.as_bytes()
    }

    /// Returns the underlying [`alloy_primitives::B256`] hash.
    #[inline]
    #[must_use]
    pub const fn as_alloy(&self) -> &B256 {
        &self.inner
    }

    /// Returns the fixed decoded byte length of a 32-byte hash.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        HASH32_HEX_CHARS / 2
    }
}

impl PartialEq for Hash32 {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for Hash32 {}

impl PartialOrd for Hash32 {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Hash32 {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl std::hash::Hash for Hash32 {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl TryFrom<String> for Hash32 {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Hash32 {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<Hash32> for String {
    fn from(value: Hash32) -> Self {
        value.hex
    }
}

impl AsRef<str> for Hash32 {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Transaction hash alias.
pub type TransactionHash = Hash32;
/// Block hash alias.
pub type BlockHash = Hash32;
/// Order digest alias.
pub type OrderDigest = Hash32;

/// Validated `CoW` order UID.
///
/// The wire form is the protocol-canonical `0x`-prefixed 114-character
/// lowercase hexadecimal string. The struct caches the input string and
/// exposes the canonical [`alloy_primitives::FixedBytes<56>`] primitive
/// through [`OrderUid::as_alloy`] for cross-crate interop.
#[doc(alias = "UID")]
#[doc(alias = "Uid")]
#[doc(alias = "order-id")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct OrderUid {
    inner: FixedBytes<56>,
    hex: String,
}

impl OrderUid {
    /// Creates a validated order UID from a `0x`-prefixed 56-byte hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        validate_hex_field("order_uid", &value, ORDER_UID_HEX_CHARS)?;
        let stripped = value
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
        Ok(Self {
            inner: FixedBytes::<56>::from(array),
            hex: value,
        })
    }

    /// Creates an order UID from its raw 56-byte representation.
    ///
    /// The input bytes are encoded into the canonical lowercase hex form
    /// without re-validating the character set, so this path is intended for
    /// protocol constants and other inputs whose byte-level shape is already
    /// known to the caller.
    ///
    /// # Panics
    ///
    /// Never panics in practice; the internal UTF-8 assertion exists only to
    /// keep the constructor free of `unsafe` code while still asserting the
    /// ASCII-only invariant produced by [`hex_encode_56`].
    #[inline]
    #[must_use]
    pub fn from_bytes(bytes: [u8; 56]) -> Self {
        let hex_bytes = hex_encode_56(bytes);
        // SAFETY: hex_encode_56 emits only the ASCII bytes `0`, `x`, and
        // lowercase hex digits.
        let hex = String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_56 only emits valid ASCII hex characters plus the 0x prefix");
        Self {
            inner: FixedBytes::<56>::from(bytes),
            hex,
        }
    }

    /// Returns the original order UID string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.hex
    }

    /// Returns the stored hex string as a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.hex.as_bytes()
    }

    /// Returns the underlying [`alloy_primitives::FixedBytes<56>`] UID.
    #[inline]
    #[must_use]
    pub const fn as_alloy(&self) -> &FixedBytes<56> {
        &self.inner
    }

    /// Returns the fixed decoded byte length of an order UID.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        ORDER_UID_HEX_CHARS / 2
    }
}

impl PartialEq for OrderUid {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.inner == other.inner
    }
}

impl Eq for OrderUid {}

impl PartialOrd for OrderUid {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for OrderUid {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl std::hash::Hash for OrderUid {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.inner.hash(state);
    }
}

impl TryFrom<String> for OrderUid {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for OrderUid {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<OrderUid> for String {
    fn from(value: OrderUid) -> Self {
        value.hex
    }
}

impl AsRef<str> for OrderUid {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for OrderUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
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
