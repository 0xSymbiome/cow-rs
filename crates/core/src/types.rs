use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};

use crate::errors::{CoreError, ValidationError};

/// Numeric EVM chain id.
pub type ChainId = u64;

/// Hex character count for an EVM address without the `0x` prefix.
pub const EVM_ADDRESS_HEX_CHARS: usize = 40;
/// Hex character count for a 32-byte app-data hash without the `0x` prefix.
pub const APP_DATA_HASH_HEX_CHARS: usize = 64;
/// Hex character count for an order UID without the `0x` prefix.
pub const ORDER_UID_HEX_CHARS: usize = 112;
/// Hex character count for a 32-byte hash without the `0x` prefix.
pub const HASH32_HEX_CHARS: usize = 64;
/// Maximum bit width accepted for unsigned protocol quantities.
pub const U256_MAX_BITS: u64 = 256;

/// Canonical EIP-712 order field names in struct-hash order.
pub const ORDER_TYPE_FIELD_NAMES: [&str; 12] = [
    "sellToken",
    "buyToken",
    "receiver",
    "sellAmount",
    "buyAmount",
    "validTo",
    "appData",
    "feeAmount",
    "kind",
    "partiallyFillable",
    "sellTokenBalance",
    "buyTokenBalance",
];

/// Canonical quote amount stage names used by [`QuoteAmountsAndCosts`].
pub const QUOTE_AMOUNT_STAGE_NAMES: [&str; 7] = [
    "beforeAllFees",
    "beforeNetworkCosts",
    "afterProtocolFees",
    "afterNetworkCosts",
    "afterPartnerFees",
    "afterSlippage",
    "amountsToSign",
];

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

/// Minimum relative-window duration accepted by [`ValidTo::relative`], in seconds.
pub const VALID_TO_MIN_RELATIVE_SECONDS: u32 = 30;

/// Maximum relative-window duration accepted by [`ValidTo::relative`], in seconds.
///
/// The default ceiling of 90 days matches the longest order horizon the
/// orderbook accepts today and keeps typed construction ahead of the
/// server-side 422 response path.
pub const VALID_TO_MAX_RELATIVE_SECONDS: u32 = 90 * 24 * 60 * 60;

/// Validated order expiration timestamp encoded as a UNIX epoch in seconds.
///
/// `ValidTo` guards construction of order-deadline values so relative durations
/// that would produce an instantly-expired order or run past the orderbook's
/// accepted horizon fail closed with a typed
/// [`ValidationError::ValidToOutOfRange`] at the client boundary. Absolute
/// epochs that already fit the `u32` range are accepted as-is so existing
/// orderbook quote responses continue to round-trip without additional
/// validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ValidTo(u32);

impl ValidTo {
    /// Creates a [`ValidTo`] from an absolute UNIX epoch timestamp in seconds.
    #[inline]
    #[must_use]
    pub const fn absolute(epoch_seconds: u32) -> Self {
        Self(epoch_seconds)
    }

    /// Creates a [`ValidTo`] by adding a relative window to the supplied UNIX epoch anchor.
    ///
    /// # Errors
    ///
    /// Returns [`ValidationError::ValidToOutOfRange`] when the window falls
    /// outside the inclusive `[VALID_TO_MIN_RELATIVE_SECONDS,
    /// VALID_TO_MAX_RELATIVE_SECONDS]` range.
    pub fn relative(now_epoch_seconds: u64, duration_seconds: u64) -> Result<Self, CoreError> {
        if duration_seconds < u64::from(VALID_TO_MIN_RELATIVE_SECONDS)
            || duration_seconds > u64::from(VALID_TO_MAX_RELATIVE_SECONDS)
        {
            return Err(ValidationError::ValidToOutOfRange {
                actual_seconds: duration_seconds,
                min: VALID_TO_MIN_RELATIVE_SECONDS,
                max: VALID_TO_MAX_RELATIVE_SECONDS,
            }
            .into());
        }

        let projected = now_epoch_seconds.saturating_add(duration_seconds);
        let clamped = projected.min(u64::from(u32::MAX));
        u32::try_from(clamped).map(Self).map_err(|_| {
            ValidationError::ValidToOutOfRange {
                actual_seconds: duration_seconds,
                min: VALID_TO_MIN_RELATIVE_SECONDS,
                max: VALID_TO_MAX_RELATIVE_SECONDS,
            }
            .into()
        })
    }

    /// Returns the validated absolute UNIX epoch timestamp.
    #[inline]
    #[must_use]
    pub const fn as_u32(self) -> u32 {
        self.0
    }

    /// Returns the validated absolute UNIX epoch timestamp as a `u64`.
    #[inline]
    #[must_use]
    pub const fn as_u64(self) -> u64 {
        self.0 as u64
    }
}

impl From<ValidTo> for u32 {
    #[inline]
    fn from(value: ValidTo) -> Self {
        value.0
    }
}

impl From<u32> for ValidTo {
    #[inline]
    fn from(value: u32) -> Self {
        Self::absolute(value)
    }
}

impl fmt::Display for ValidTo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Canonical non-negative `uint256` quantity rendered in the smallest token unit.
///
/// `Amount` is the typed boundary for atomic token values on every
/// `CoW` Protocol surface: contract hashing, EIP-712 typed data,
/// orderbook DTOs, and decimal-aware display. The inner `BigUint`
/// stays the authoritative storage while the wire format remains the
/// canonical base-10 string accepted by the orderbook and contract
/// layer. The custom `Serialize`/`Deserialize` impls emit and parse
/// that decimal-string form without changing the stored numeric.
///
/// For decimal-aware values that also carry a scale, see
/// [`DecimalAmount`]. For signed quantities, see [`SignedAmount`].
///
/// ```compile_fail
/// use cow_sdk_core::Amount;
///
/// let _: Amount = String::from("1").into();
/// ```
///
/// ```compile_fail
/// use cow_sdk_core::Amount;
///
/// let _: Amount = "1".into();
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Amount(BigUint);

impl Amount {
    /// Parses a canonical non-negative `uint256` quantity from a
    /// decimal or `0x`-prefixed hexadecimal literal.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, cannot be parsed, or
    /// exceeds `uint256` bounds.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        parse_u256_quantity("amount", &value.into()).map(Self)
    }

    /// Returns the zero quantity.
    #[inline]
    #[must_use]
    pub fn zero() -> Self {
        Self(BigUint::from(0u32))
    }

    /// Creates an amount from a raw `BigUint` quantity.
    #[inline]
    #[must_use]
    pub const fn from_atoms(atoms: BigUint) -> Self {
        Self(atoms)
    }

    /// Consumes the amount and returns the raw `BigUint` quantity.
    #[inline]
    #[must_use]
    pub fn into_biguint(self) -> BigUint {
        self.0
    }

    /// Returns a borrow of the raw `BigUint` quantity.
    #[inline]
    #[must_use]
    pub const fn as_biguint(&self) -> &BigUint {
        &self.0
    }

    /// Returns `true` when this amount equals the zero quantity.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == BigUint::from(0u32)
    }

    /// Returns the sum of two typed amounts.
    #[inline]
    #[must_use]
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        Some(Self(&self.0 + &other.0))
    }

    /// Returns the difference of two typed amounts, or `None` on underflow.
    #[inline]
    #[must_use]
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        if self.0 < other.0 {
            return None;
        }

        Some(Self(&self.0 - &other.0))
    }

    /// Returns the product of two typed amounts.
    #[inline]
    #[must_use]
    pub fn checked_mul(&self, other: &Self) -> Option<Self> {
        let product = &self.0 * &other.0;
        (product.bits() <= U256_MAX_BITS).then_some(Self(product))
    }
}

impl Default for Amount {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<BigUint> for Amount {
    #[inline]
    fn from(value: BigUint) -> Self {
        Self(value)
    }
}

impl From<Amount> for BigUint {
    #[inline]
    fn from(value: Amount) -> Self {
        value.0
    }
}

impl From<u32> for Amount {
    #[inline]
    fn from(value: u32) -> Self {
        Self(BigUint::from(value))
    }
}

impl From<u64> for Amount {
    #[inline]
    fn from(value: u64) -> Self {
        Self(BigUint::from(value))
    }
}

impl From<u128> for Amount {
    #[inline]
    fn from(value: u128) -> Self {
        Self(BigUint::from(value))
    }
}

impl From<usize> for Amount {
    #[inline]
    fn from(value: usize) -> Self {
        Self(BigUint::from(value))
    }
}

impl TryFrom<&str> for Amount {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_u256_quantity("amount", value).map(Self)
    }
}

impl TryFrom<String> for Amount {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        parse_u256_quantity("amount", &value).map(Self)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Serialize for Amount {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_str_radix(10))
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        parse_u256_quantity("amount", &value)
            .map(Self)
            .map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

impl Add<Self> for Amount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<Self> for Amount {
    type Output = Option<Self>;

    fn sub(self, rhs: Self) -> Self::Output {
        self.checked_sub(&rhs)
    }
}

impl AddAssign<Self> for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign<Self> for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

/// Decimal-aware token amount pairing an atomic quantity with a decimals scale.
///
/// `DecimalAmount` keeps the authoritative storage in atoms so settlement
/// arithmetic stays exact, while exposing the decimals scale for display and
/// human-oriented conversion paths. Wire formats continue to carry the
/// atomic value as a base-10 string; this type is intended for in-process
/// typing and ergonomic conversions rather than transport.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecimalAmount {
    atoms: BigUint,
    decimals: u8,
}

impl DecimalAmount {
    /// Creates a decimal-aware amount from an atomic quantity and decimals scale.
    #[inline]
    #[must_use]
    pub const fn new(atoms: BigUint, decimals: u8) -> Self {
        Self { atoms, decimals }
    }

    /// Creates an approximate decimal-aware amount from a whole-unit floating-point value.
    ///
    /// Non-finite or negative inputs clamp to zero. The conversion is lossy
    /// beyond `f64` precision and is intended for display or user-input flows
    /// rather than settlement arithmetic.
    #[must_use]
    pub fn from_whole_approx(whole_units: f64, decimals: u8) -> Self {
        if !whole_units.is_finite() || whole_units < 0.0 {
            return Self {
                atoms: BigUint::from(0u32),
                decimals,
            };
        }
        let scale = 10f64.powi(i32::from(decimals));
        let atoms_f = whole_units * scale;
        #[allow(
            clippy::cast_precision_loss,
            reason = "the clamp bounds the value by u128::MAX as f64 before the lossy truncation"
        )]
        let upper_bound = u128::MAX as f64;
        let bounded = atoms_f.clamp(0.0, upper_bound);
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "the clamp restricts the input to the non-negative u128 range before truncation"
        )]
        let atoms_u128 = bounded as u128;
        Self {
            atoms: BigUint::from(atoms_u128),
            decimals,
        }
    }

    /// Returns a borrow of the raw atomic quantity.
    #[inline]
    #[must_use]
    pub const fn atoms(&self) -> &BigUint {
        &self.atoms
    }

    /// Consumes the decimal amount and returns the raw atomic quantity.
    #[inline]
    #[must_use]
    pub fn into_atoms(self) -> BigUint {
        self.atoms
    }

    /// Returns the configured decimals scale.
    #[inline]
    #[must_use]
    pub const fn decimals(&self) -> u8 {
        self.decimals
    }

    /// Returns an approximate floating-point whole-unit value for display.
    ///
    /// The conversion is lossy beyond `f64` precision and should not be used
    /// for settlement arithmetic.
    #[must_use]
    pub fn to_f64_approx(&self) -> f64 {
        let atoms_str = self.atoms.to_str_radix(10);
        let atoms_f: f64 = atoms_str.parse().unwrap_or(f64::NAN);
        let scale = 10f64.powi(i32::from(self.decimals));
        atoms_f / scale
    }
}

/// Canonical signed integer with typed [`BigInt`] storage and a decimal-string wire form.
#[derive(Debug, Clone)]
pub struct SignedAmount {
    value: BigInt,
    canonical: Box<str>,
}

impl SignedAmount {
    /// Creates a canonical signed integer quantity.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty or cannot be parsed as a
    /// base-10 signed integer.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let value = value.into();
        let parsed = parse_signed_quantity("signed_amount", &value)?;
        Ok(Self::from_bigint(parsed))
    }

    /// Creates a signed amount from its raw arbitrary-precision integer value.
    #[must_use]
    pub fn from_bigint(value: BigInt) -> Self {
        Self {
            canonical: value.to_string().into_boxed_str(),
            value,
        }
    }

    /// Returns the zero quantity.
    #[must_use]
    pub fn zero() -> Self {
        Self::from_bigint(BigInt::from(0u32))
    }

    /// Returns a borrow of the raw `BigInt` quantity.
    #[inline]
    #[must_use]
    pub const fn as_bigint(&self) -> &BigInt {
        &self.value
    }

    /// Consumes the signed amount and returns the raw `BigInt` quantity.
    #[inline]
    #[must_use]
    pub fn into_bigint(self) -> BigInt {
        self.value
    }

    /// Returns the canonical decimal string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.canonical
    }

    /// Returns the sum of two signed amounts when the underlying `BigInt`
    /// implementation accepts the operation.
    #[must_use]
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        self.value.checked_add(&other.value).map(Self::from_bigint)
    }

    /// Returns the difference of two signed amounts when the underlying `BigInt`
    /// implementation accepts the operation.
    #[must_use]
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.value.checked_sub(&other.value).map(Self::from_bigint)
    }

    /// Returns the product of two signed amounts when the underlying `BigInt`
    /// implementation accepts the operation.
    #[must_use]
    pub fn checked_mul(&self, other: &Self) -> Option<Self> {
        self.value.checked_mul(&other.value).map(Self::from_bigint)
    }

    fn refresh_canonical(&mut self) {
        self.canonical = self.value.to_string().into_boxed_str();
    }
}

impl Default for SignedAmount {
    fn default() -> Self {
        Self::zero()
    }
}

impl TryFrom<String> for SignedAmount {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for SignedAmount {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<SignedAmount> for String {
    fn from(value: SignedAmount) -> Self {
        value.canonical.into()
    }
}

impl AsRef<str> for SignedAmount {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for SignedAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl PartialEq for SignedAmount {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl Eq for SignedAmount {}

impl PartialOrd for SignedAmount {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for SignedAmount {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

impl std::hash::Hash for SignedAmount {
    #[inline]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl Serialize for SignedAmount {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for SignedAmount {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        parse_signed_quantity("signed_amount", &value)
            .map(Self::from_bigint)
            .map_err(|err| serde::de::Error::custom(err.to_string()))
    }
}

impl Add<Self> for SignedAmount {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::from_bigint(self.value + rhs.value)
    }
}

impl Sub<Self> for SignedAmount {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::from_bigint(self.value - rhs.value)
    }
}

impl AddAssign<Self> for SignedAmount {
    fn add_assign(&mut self, rhs: Self) {
        self.value += rhs.value;
        self.refresh_canonical();
    }
}

impl SubAssign<Self> for SignedAmount {
    fn sub_assign(&mut self, rhs: Self) {
        self.value -= rhs.value;
        self.refresh_canonical();
    }
}

/// Sell or buy side of a trade.
///
/// Encoded as `keccak256("buy")` / `keccak256("sell")` in the EIP-712
/// `Order` type. The set of variants is fixed by the protocol; adding a third
/// variant would change the protocol, not the SDK. Classified as
/// `protocol-fixed-exhaustive` in the workspace enum policy manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderKind {
    /// Buy an exact amount of the buy token.
    Buy,
    /// Sell an exact amount of the sell token.
    Sell,
}

/// Source from which the `sellAmount` is drawn upon order fulfillment.
///
/// This mirrors the services `SellTokenSource` enum byte-for-byte on the wire.
/// Orders model the sell-side allowance path independently of the buy-side
/// payout path, which is typed as [`BuyTokenDestination`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum SellTokenSource {
    /// Sell tokens are drawn through the regular ERC-20 allowance granted to
    /// the vault relayer.
    #[default]
    Erc20,
    /// Sell tokens are drawn through the Balancer vault relayer using an
    /// external ERC-20 allowance on the vault.
    External,
    /// Sell tokens are drawn from the user's internal Balancer vault balance.
    Internal,
}

/// Destination to which the `buyAmount` is transferred upon order fulfillment.
///
/// This mirrors the services `BuyTokenDestination` enum byte-for-byte on the
/// wire. The buy-side payout path only accepts the ERC-20 and internal
/// variants; the [`SellTokenSource::External`] variant has no buy-side
/// counterpart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[non_exhaustive]
#[serde(rename_all = "snake_case")]
pub enum BuyTokenDestination {
    /// Buy tokens are paid out as a regular ERC-20 transfer.
    #[default]
    Erc20,
    /// Buy tokens are paid out as a Balancer vault internal balance credit.
    Internal,
}

/// Token metadata used by user-domain SDK surfaces.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenInfo {
    /// Numeric chain id that owns the token.
    pub chain_id: ChainId,
    /// Token contract address.
    pub address: Address,
    /// Token decimals.
    pub decimals: u8,
    /// Display symbol.
    pub symbol: String,
    /// Display name.
    pub name: String,
    /// Optional logo URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logo_url: Option<String>,
}

impl TokenInfo {
    /// Creates token metadata from the canonical display fields.
    #[inline]
    #[must_use]
    pub const fn new(
        chain_id: ChainId,
        address: Address,
        decimals: u8,
        symbol: String,
        name: String,
        logo_url: Option<String>,
    ) -> Self {
        Self {
            chain_id,
            address,
            decimals,
            symbol,
            name,
            logo_url,
        }
    }
}

/// Compares two addresses using case-insensitive normalization.
///
/// Equivalent to `left == right`; kept as a named helper for call sites that
/// want to make the case-insensitive intent explicit.
#[inline]
#[must_use]
pub fn addresses_equal(left: &Address, right: &Address) -> bool {
    left == right
}

/// Builds the canonical `<chain_id>:<lowercase-address>` token identifier.
#[inline]
#[must_use]
pub fn token_id(chain_id: ChainId, address: &Address) -> String {
    format!("{chain_id}:{}", address.normalized_key())
}

/// User-domain order shape prepared for signing and trading workflows.
///
/// This is not an orderbook wire DTO or an ABI struct. Contract hashing converts
/// it into `cow_sdk_contracts::Order`, where receiver and token-balance defaults
/// are normalized for EIP-712 hashing.
///
/// Downstream crates construct orders through [`UnsignedOrder::new`] and the
/// chainable `with_*` setters rather than a struct literal so additive fields
/// remain semver-compatible.
///
/// ```compile_fail
/// use cow_sdk_core::{
///     Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource,
///     UnsignedOrder,
/// };
///
/// let _order = UnsignedOrder {
///     sell_token: Address::new("0x1111111111111111111111111111111111111111").unwrap(),
///     buy_token: Address::new("0x2222222222222222222222222222222222222222").unwrap(),
///     receiver: Address::new("0x3333333333333333333333333333333333333333").unwrap(),
///     sell_amount: Amount::new("100").unwrap(),
///     buy_amount: Amount::new("200").unwrap(),
///     valid_to: 1_700_000_000,
///     app_data: AppDataHash::new(
///         "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
///     )
///     .unwrap(),
///     fee_amount: Amount::new("5").unwrap(),
///     kind: OrderKind::Sell,
///     partially_fillable: true,
///     sell_token_balance: SellTokenSource::External,
///     buy_token_balance: BuyTokenDestination::Internal,
/// };
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnsignedOrder {
    /// Sell token address.
    pub sell_token: Address,
    /// Buy token address.
    pub buy_token: Address,
    /// Receiver of the bought tokens.
    pub receiver: Address,
    /// Exact sell amount for sell orders or maximum sell amount for buy orders.
    pub sell_amount: Amount,
    /// Exact buy amount for buy orders or minimum buy amount for sell orders.
    pub buy_amount: Amount,
    /// Expiration timestamp encoded as `uint32`.
    pub valid_to: u32,
    /// App-data hash linked to the order.
    pub app_data: AppDataHash,
    /// Fee amount encoded in sell-token units.
    pub fee_amount: Amount,
    /// Order side.
    pub kind: OrderKind,
    /// Whether the order can be partially filled.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default)]
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination.
    #[serde(default)]
    pub buy_token_balance: BuyTokenDestination,
}

impl UnsignedOrder {
    /// Creates an unsigned order from the canonical EIP-712 field set.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        sell_token: Address,
        buy_token: Address,
        receiver: Address,
        sell_amount: Amount,
        buy_amount: Amount,
        valid_to: u32,
        app_data: AppDataHash,
        fee_amount: Amount,
        kind: OrderKind,
        partially_fillable: bool,
        sell_token_balance: SellTokenSource,
        buy_token_balance: BuyTokenDestination,
    ) -> Self {
        Self {
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            fee_amount,
            kind,
            partially_fillable,
            sell_token_balance,
            buy_token_balance,
        }
    }

    /// Returns a copy of this order with a different receiver.
    #[must_use]
    pub fn with_receiver(mut self, receiver: Address) -> Self {
        self.receiver = receiver;
        self
    }

    /// Returns a copy of this order with a different app-data hash.
    #[must_use]
    pub fn with_app_data(mut self, app_data: AppDataHash) -> Self {
        self.app_data = app_data;
        self
    }

    /// Returns a copy of this order with a different fee amount.
    #[must_use]
    pub fn with_fee_amount(mut self, fee_amount: Amount) -> Self {
        self.fee_amount = fee_amount;
        self
    }

    /// Returns a copy of this order with an updated partial-fill flag.
    #[must_use]
    pub const fn with_partially_fillable(mut self, partially_fillable: bool) -> Self {
        self.partially_fillable = partially_fillable;
        self
    }

    /// Returns a copy of this order with a different sell-token balance source.
    #[must_use]
    pub const fn with_sell_token_balance(mut self, sell_token_balance: SellTokenSource) -> Self {
        self.sell_token_balance = sell_token_balance;
        self
    }

    /// Returns a copy of this order with a different buy-token balance destination.
    #[must_use]
    pub const fn with_buy_token_balance(mut self, buy_token_balance: BuyTokenDestination) -> Self {
        self.buy_token_balance = buy_token_balance;
        self
    }

    /// Returns the canonical EIP-712 field ordering for orders.
    #[must_use]
    pub const fn field_names() -> &'static [&'static str; ORDER_TYPE_FIELD_NAMES.len()] {
        &ORDER_TYPE_FIELD_NAMES
    }
}

/// Optional order envelope used by SDK consumers that need owner or uid context
/// alongside the user-domain unsigned order.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Order {
    /// Unsigned user-domain order payload.
    #[serde(flatten)]
    pub unsigned: UnsignedOrder,
    /// Optional order owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    /// Optional persisted order UID.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<OrderUid>,
}

impl Order {
    /// Creates an optional order envelope around an unsigned order.
    #[inline]
    #[must_use]
    pub const fn new(
        unsigned: UnsignedOrder,
        owner: Option<Address>,
        uid: Option<OrderUid>,
    ) -> Self {
        Self {
            unsigned,
            owner,
            uid,
        }
    }
}

/// Simplified trade execution view used by SDK consumers.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Trade {
    /// Order UID that produced the trade.
    pub order_uid: OrderUid,
    /// Executed sell amount.
    pub executed_sell_amount: Amount,
    /// Executed buy amount.
    pub executed_buy_amount: Amount,
}

impl Trade {
    /// Creates a simplified trade execution view.
    #[inline]
    #[must_use]
    pub const fn new(
        order_uid: OrderUid,
        executed_sell_amount: Amount,
        executed_buy_amount: Amount,
    ) -> Self {
        Self {
            order_uid,
            executed_sell_amount,
            executed_buy_amount,
        }
    }
}

/// Backward-compatible alias for the user-domain trade model.
pub type TradeModel = Trade;

/// User-domain quote request shape with validated quantities.
///
/// This is not the orderbook HTTP wire DTO. The orderbook crate keeps the upstream
/// string-based transport contract explicit.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    /// Quote side.
    pub kind: OrderKind,
    /// Optional sell token address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token: Option<Address>,
    /// Optional buy token address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token: Option<Address>,
    /// Optional receiver address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Optional order owner address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Optional sell amount input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount: Option<Amount>,
    /// Optional buy amount input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount: Option<Amount>,
    /// Optional explicit fee amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<Amount>,
    /// Optional app-data hash reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
    /// Optional raw app-data document payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    /// Optional order expiration timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
}

impl QuoteRequest {
    /// Creates a user-domain quote request from its optional input fields.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        kind: OrderKind,
        sell_token: Option<Address>,
        buy_token: Option<Address>,
        receiver: Option<Address>,
        from: Option<Address>,
        sell_amount: Option<Amount>,
        buy_amount: Option<Amount>,
        fee_amount: Option<Amount>,
        app_data_hash: Option<AppDataHash>,
        app_data: Option<String>,
        valid_to: Option<u32>,
    ) -> Self {
        Self {
            kind,
            sell_token,
            buy_token,
            receiver,
            from,
            sell_amount,
            buy_amount,
            fee_amount,
            app_data_hash,
            app_data,
            valid_to,
        }
    }
}

/// User-domain quote response with validated quantities.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    /// Quote side.
    pub kind: OrderKind,
    /// Sell amount returned by the quote.
    pub sell_amount: Amount,
    /// Buy amount returned by the quote.
    pub buy_amount: Amount,
    /// Fee amount returned by the quote.
    pub fee_amount: Amount,
    /// Optional order UID when the quote is tied to a persisted order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_uid: Option<OrderUid>,
    /// Optional price string from the upstream API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// Optional quote identifier from the upstream API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<String>,
    /// Optional stepwise amounts-and-costs breakdown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amounts_and_costs: Option<QuoteAmountsAndCosts>,
}

impl QuoteResponse {
    /// Creates a user-domain quote response from the canonical amount fields.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        kind: OrderKind,
        sell_amount: Amount,
        buy_amount: Amount,
        fee_amount: Amount,
        order_uid: Option<OrderUid>,
        price: Option<String>,
        quote_id: Option<String>,
        amounts_and_costs: Option<QuoteAmountsAndCosts>,
    ) -> Self {
        Self {
            kind,
            sell_amount,
            buy_amount,
            fee_amount,
            order_uid,
            price,
            quote_id,
            amounts_and_costs,
        }
    }
}

/// Generic sell/buy amount pair.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amounts<T> {
    /// Sell-side amount.
    pub sell_amount: T,
    /// Buy-side amount.
    pub buy_amount: T,
}

impl<T> Amounts<T> {
    /// Creates a sell/buy amount pair.
    #[inline]
    #[must_use]
    pub const fn new(sell_amount: T, buy_amount: T) -> Self {
        Self {
            sell_amount,
            buy_amount,
        }
    }
}

/// Network-fee amounts expressed in both quote currencies.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFee<T> {
    /// Network fee expressed in sell-token units.
    pub amount_in_sell_currency: T,
    /// Network fee expressed in buy-token units.
    pub amount_in_buy_currency: T,
}

impl<T> NetworkFee<T> {
    /// Creates network-fee amounts in both quote currencies.
    #[inline]
    #[must_use]
    pub const fn new(amount_in_sell_currency: T, amount_in_buy_currency: T) -> Self {
        Self {
            amount_in_sell_currency,
            amount_in_buy_currency,
        }
    }
}

/// Generic fee component represented by amount and basis points.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeComponent<T> {
    /// Fee amount.
    pub amount: T,
    /// Fee in basis points.
    pub bps: u32,
}

impl<T> FeeComponent<T> {
    /// Creates a fee component from an amount and basis-point value.
    #[inline]
    #[must_use]
    pub const fn new(amount: T, bps: u32) -> Self {
        Self { amount, bps }
    }
}

/// Full quote cost breakdown.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Costs<T> {
    /// Network fee component.
    pub network_fee: NetworkFee<T>,
    /// Partner fee component.
    pub partner_fee: FeeComponent<T>,
    /// Protocol fee component.
    pub protocol_fee: FeeComponent<T>,
}

impl<T> Costs<T> {
    /// Creates a full quote cost breakdown.
    #[inline]
    #[must_use]
    pub const fn new(
        network_fee: NetworkFee<T>,
        partner_fee: FeeComponent<T>,
        protocol_fee: FeeComponent<T>,
    ) -> Self {
        Self {
            network_fee,
            partner_fee,
            protocol_fee,
        }
    }
}

/// Stepwise quote amounts and cost components across the quote lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteAmountsAndCosts<T = Amount> {
    /// Whether the source quote was sell-sided.
    pub is_sell: bool,
    /// Cost breakdown for the quote.
    pub costs: Costs<T>,
    /// Amounts before all fees.
    pub before_all_fees: Amounts<T>,
    /// Amounts before network costs.
    pub before_network_costs: Amounts<T>,
    /// Amounts after protocol fees.
    pub after_protocol_fees: Amounts<T>,
    /// Amounts after network costs.
    pub after_network_costs: Amounts<T>,
    /// Amounts after partner fees.
    pub after_partner_fees: Amounts<T>,
    /// Amounts after slippage.
    pub after_slippage: Amounts<T>,
    /// Amounts that should be signed.
    pub amounts_to_sign: Amounts<T>,
}

impl<T> QuoteAmountsAndCosts<T> {
    /// Creates a quote-stage breakdown from its individual stage amounts.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        is_sell: bool,
        costs: Costs<T>,
        before_all_fees: Amounts<T>,
        before_network_costs: Amounts<T>,
        after_protocol_fees: Amounts<T>,
        after_network_costs: Amounts<T>,
        after_partner_fees: Amounts<T>,
        after_slippage: Amounts<T>,
        amounts_to_sign: Amounts<T>,
    ) -> Self {
        Self {
            is_sell,
            costs,
            before_all_fees,
            before_network_costs,
            after_protocol_fees,
            after_network_costs,
            after_partner_fees,
            after_slippage,
            amounts_to_sign,
        }
    }

    /// Returns the canonical stage ordering for quote amount breakdowns.
    #[must_use]
    pub const fn stage_names() -> &'static [&'static str; QUOTE_AMOUNT_STAGE_NAMES.len()] {
        &QUOTE_AMOUNT_STAGE_NAMES
    }
}

const HEX_ALPHABET: [u8; 16] = *b"0123456789abcdef";

/// Returns the canonical `0x`-prefixed lowercase hex encoding of a 20-byte input.
#[must_use]
pub const fn hex_encode_20(bytes: [u8; 20]) -> [u8; 42] {
    let mut out = [0u8; 42];
    out[0] = b'0';
    out[1] = b'x';
    let mut i = 0;
    while i < 20 {
        let byte = bytes[i];
        out[2 + 2 * i] = HEX_ALPHABET[(byte >> 4) as usize];
        out[2 + 2 * i + 1] = HEX_ALPHABET[(byte & 0x0F) as usize];
        i += 1;
    }
    out
}

/// Returns the canonical `0x`-prefixed lowercase hex encoding of a 32-byte input.
#[must_use]
pub const fn hex_encode_32(bytes: [u8; 32]) -> [u8; 66] {
    let mut out = [0u8; 66];
    out[0] = b'0';
    out[1] = b'x';
    let mut i = 0;
    while i < 32 {
        let byte = bytes[i];
        out[2 + 2 * i] = HEX_ALPHABET[(byte >> 4) as usize];
        out[2 + 2 * i + 1] = HEX_ALPHABET[(byte & 0x0F) as usize];
        i += 1;
    }
    out
}

/// Returns the canonical `0x`-prefixed lowercase hex encoding of a 56-byte input.
#[must_use]
pub const fn hex_encode_56(bytes: [u8; 56]) -> [u8; 114] {
    let mut out = [0u8; 114];
    out[0] = b'0';
    out[1] = b'x';
    let mut i = 0;
    while i < 56 {
        let byte = bytes[i];
        out[2 + 2 * i] = HEX_ALPHABET[(byte >> 4) as usize];
        out[2 + 2 * i + 1] = HEX_ALPHABET[(byte & 0x0F) as usize];
        i += 1;
    }
    out
}

/// Decodes a `0x`-prefixed hex string literal into a fixed-length byte array at compile time.
///
/// Intended for converting the embedded protocol-address hex literals to their
/// raw byte form inside `const` initialisers.
///
/// # Panics
///
/// Panics at compile time when the input is not exactly 42 characters long,
/// is missing the `0x` prefix, or contains a non-hex character.
#[must_use]
pub const fn hex_decode_20(hex: &str) -> [u8; 20] {
    let bytes = hex.as_bytes();
    assert!(
        bytes.len() == 42,
        "hex_decode_20 requires a 42-character input"
    );
    assert!(
        bytes[0] == b'0' && bytes[1] == b'x',
        "hex_decode_20 requires a 0x prefix"
    );
    let mut out = [0u8; 20];
    let mut i = 0;
    while i < 20 {
        out[i] = (decode_nibble(bytes[2 + 2 * i]) << 4) | decode_nibble(bytes[2 + 2 * i + 1]);
        i += 1;
    }
    out
}

/// Decodes a `0x`-prefixed 32-byte hex string literal at compile time.
///
/// # Panics
///
/// Panics at compile time when the input is not exactly 66 characters long,
/// is missing the `0x` prefix, or contains a non-hex character.
#[must_use]
pub const fn hex_decode_32(hex: &str) -> [u8; 32] {
    let bytes = hex.as_bytes();
    assert!(
        bytes.len() == 66,
        "hex_decode_32 requires a 66-character input"
    );
    assert!(
        bytes[0] == b'0' && bytes[1] == b'x',
        "hex_decode_32 requires a 0x prefix"
    );
    let mut out = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        out[i] = (decode_nibble(bytes[2 + 2 * i]) << 4) | decode_nibble(bytes[2 + 2 * i + 1]);
        i += 1;
    }
    out
}

/// Decodes a `0x`-prefixed 56-byte hex string literal at compile time.
///
/// # Panics
///
/// Panics at compile time when the input is not exactly 114 characters long,
/// is missing the `0x` prefix, or contains a non-hex character.
#[must_use]
pub const fn hex_decode_56(hex: &str) -> [u8; 56] {
    let bytes = hex.as_bytes();
    assert!(
        bytes.len() == 114,
        "hex_decode_56 requires a 114-character input"
    );
    assert!(
        bytes[0] == b'0' && bytes[1] == b'x',
        "hex_decode_56 requires a 0x prefix"
    );
    let mut out = [0u8; 56];
    let mut i = 0;
    while i < 56 {
        out[i] = (decode_nibble(bytes[2 + 2 * i]) << 4) | decode_nibble(bytes[2 + 2 * i + 1]);
        i += 1;
    }
    out
}

const fn decode_nibble(c: u8) -> u8 {
    match c {
        b'0'..=b'9' => c - b'0',
        b'a'..=b'f' => c - b'a' + 10,
        b'A'..=b'F' => c - b'A' + 10,
        _ => panic!("hex nibble must be 0-9, a-f, or A-F"),
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

fn parse_u256_quantity(field: &'static str, value: &str) -> Result<BigUint, CoreError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyField { field }.into());
    }

    let parsed = value
        .strip_prefix("0x")
        .map_or_else(
            || BigUint::parse_bytes(value.as_bytes(), 10),
            |stripped| BigUint::parse_bytes(stripped.as_bytes(), 16),
        )
        .ok_or(ValidationError::InvalidNumeric { field })?;

    if parsed.bits() > U256_MAX_BITS {
        return Err(ValidationError::NumericOverflow { field }.into());
    }

    Ok(parsed)
}

fn parse_signed_quantity(field: &'static str, value: &str) -> Result<BigInt, CoreError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyField { field }.into());
    }

    BigInt::parse_bytes(value.as_bytes(), 10)
        .ok_or_else(|| ValidationError::InvalidNumeric { field }.into())
}

#[cfg(test)]
mod tests {
    use super::{
        Address, Amount, AppDataHash, BuyTokenDestination, OrderKind, SellTokenSource,
        UnsignedOrder,
    };

    #[test]
    fn unsigned_order_builder_serializes_identically_to_internal_literal_construction() {
        let sell_token = Address::new("0x1111111111111111111111111111111111111111").unwrap();
        let buy_token = Address::new("0x2222222222222222222222222222222222222222").unwrap();
        let receiver = Address::new("0x3333333333333333333333333333333333333333").unwrap();
        let sell_amount = Amount::new("100").unwrap();
        let buy_amount = Amount::new("200").unwrap();
        let valid_to = 1_700_000_000;
        let app_data =
            AppDataHash::new("0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
                .unwrap();
        let fee_amount = Amount::new("5").unwrap();

        let from_builder = UnsignedOrder::new(
            sell_token.clone(),
            buy_token.clone(),
            receiver.clone(),
            sell_amount.clone(),
            buy_amount.clone(),
            valid_to,
            app_data.clone(),
            Amount::zero(),
            OrderKind::Sell,
            false,
            SellTokenSource::Erc20,
            BuyTokenDestination::Erc20,
        )
        .with_receiver(receiver.clone())
        .with_app_data(app_data.clone())
        .with_fee_amount(fee_amount.clone())
        .with_partially_fillable(true)
        .with_sell_token_balance(SellTokenSource::External)
        .with_buy_token_balance(BuyTokenDestination::Internal);

        let from_literal = UnsignedOrder {
            sell_token,
            buy_token,
            receiver,
            sell_amount,
            buy_amount,
            valid_to,
            app_data,
            fee_amount,
            kind: OrderKind::Sell,
            partially_fillable: true,
            sell_token_balance: SellTokenSource::External,
            buy_token_balance: BuyTokenDestination::Internal,
        };

        assert_eq!(
            serde_json::to_vec(&from_builder).unwrap(),
            serde_json::to_vec(&from_literal).unwrap()
        );
    }
}
