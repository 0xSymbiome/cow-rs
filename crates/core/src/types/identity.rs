use std::fmt;

use serde::{Deserialize, Serialize};

use super::hex::{
    APP_DATA_HASH_HEX_CHARS, EVM_ADDRESS_HEX_CHARS, HASH32_HEX_CHARS, ORDER_UID_HEX_CHARS,
    hex_encode_20, hex_encode_32, hex_encode_56,
};
use crate::errors::{CoreError, ValidationError};
/// Numeric EVM chain id.
pub type ChainId = u64;

/// Validated EVM address string.
///
/// [`PartialEq`], [`Eq`], [`Hash`](std::hash::Hash), [`PartialOrd`], and [`Ord`]
/// compare addresses case-insensitively so mixed-case hexadecimal variants of the
/// same address are treated as equal. [`Address::as_str`] preserves the original
/// input casing exactly, while [`Address::normalized_key`] exposes the lowercase
/// form used for those comparisons.
///
/// ```compile_fail
/// use cow_sdk_core::Address;
///
/// let _: Address = String::from("0x0000000000000000000000000000000000000001").into();
/// ```
///
/// ```compile_fail
/// use cow_sdk_core::Address;
///
/// let _: Address = "0x0000000000000000000000000000000000000001".into();
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Address(String);

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
        Ok(Self(value))
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
        let value = String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_20 only emits valid ASCII hex characters plus the 0x prefix");
        Self(value)
    }

    /// Returns the original address string.
    ///
    /// The stored string preserves the input casing exactly; equality and
    /// hashing operate on the lowercase form instead.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the stored hex string as a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
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
        self.0.to_ascii_lowercase()
    }
}

impl PartialEq for Address {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0.eq_ignore_ascii_case(&other.0)
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
        self.0
            .bytes()
            .map(|byte| byte.to_ascii_lowercase())
            .cmp(other.0.bytes().map(|byte| byte.to_ascii_lowercase()))
    }
}

impl std::hash::Hash for Address {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for byte in self.0.as_bytes() {
            state.write_u8(byte.to_ascii_lowercase());
        }
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
        value.0
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
/// ```compile_fail
/// use cow_sdk_core::HexData;
///
/// let _: HexData = String::from("0x1234").into();
/// ```
///
/// ```compile_fail
/// use cow_sdk_core::HexData;
///
/// let _: HexData = "0x1234".into();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct HexData(String);

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
        Ok(Self(value))
    }

    /// Returns the canonical empty payload.
    #[must_use]
    pub fn empty() -> Self {
        Self("0x".to_owned())
    }

    /// Returns the original hex string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the stored hex string as a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns the decoded byte length of the payload.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        (self.0.len() - 2) / 2
    }
}

impl Default for HexData {
    fn default() -> Self {
        Self::empty()
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
        value.0
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

/// Validated 32-byte app-data hash string.
///
/// ```compile_fail
/// use cow_sdk_core::AppDataHash;
///
/// let _: AppDataHash =
///     String::from("0x0000000000000000000000000000000000000000000000000000000000000000")
///         .into();
/// ```
///
/// ```compile_fail
/// use cow_sdk_core::AppDataHash;
///
/// let _: AppDataHash =
///     "0x0000000000000000000000000000000000000000000000000000000000000000".into();
/// ```
#[doc(alias = "app-data")]
#[doc(alias = "AppData")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct AppDataHash(String);

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
        Ok(Self(value))
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
        let value = String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_32 only emits valid ASCII hex characters plus the 0x prefix");
        Self(value)
    }

    /// Returns the original hash string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the stored hex string as a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns the fixed decoded byte length of an app-data hash.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        APP_DATA_HASH_HEX_CHARS / 2
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
        value.0
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
/// ```compile_fail
/// use cow_sdk_core::Hash32;
///
/// let _: Hash32 =
///     String::from("0x0000000000000000000000000000000000000000000000000000000000000000")
///         .into();
/// ```
///
/// ```compile_fail
/// use cow_sdk_core::Hash32;
///
/// let _: Hash32 =
///     "0x0000000000000000000000000000000000000000000000000000000000000000".into();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Hash32(String);

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
        Ok(Self(value))
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
        let value = String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_32 only emits valid ASCII hex characters plus the 0x prefix");
        Self(value)
    }

    /// Returns the original hash string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the stored hex string as a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns the fixed decoded byte length of a 32-byte hash.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        HASH32_HEX_CHARS / 2
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
        value.0
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

/// Validated `CoW` order UID string.
///
/// ```compile_fail
/// use cow_sdk_core::OrderUid;
///
/// let _: OrderUid = String::from(
///     "0x59920c85de0162e9e55df8d396e75f3b6b7c2dfdb535f03e5c807731c31585eaff714b8b0e2700303ec912bd40496c3997ceea2b616d6710",
/// )
/// .into();
/// ```
///
/// ```compile_fail
/// use cow_sdk_core::OrderUid;
///
/// let _: OrderUid =
///     "0x59920c85de0162e9e55df8d396e75f3b6b7c2dfdb535f03e5c807731c31585eaff714b8b0e2700303ec912bd40496c3997ceea2b616d6710"
///         .into();
/// ```
#[doc(alias = "UID")]
#[doc(alias = "Uid")]
#[doc(alias = "order-id")]
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct OrderUid(String);

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
        Ok(Self(value))
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
        let value = String::from_utf8(hex_bytes.to_vec())
            .expect("hex_encode_56 only emits valid ASCII hex characters plus the 0x prefix");
        Self(value)
    }

    /// Returns the original order UID string.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the stored hex string as a byte slice.
    #[inline]
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    /// Returns the fixed decoded byte length of an order UID.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        ORDER_UID_HEX_CHARS / 2
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
        value.0
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
