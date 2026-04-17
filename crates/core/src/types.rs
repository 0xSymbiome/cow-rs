use std::fmt;

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

/// Canonical non-negative `uint256` quantity rendered as a base-10 string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Amount(String);

impl Amount {
    /// Creates a canonical non-negative `uint256` quantity.
    ///
    /// Decimal strings and `0x`-prefixed hexadecimal strings are accepted.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, cannot be parsed, or
    /// exceeds `uint256` bounds.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let parsed = parse_u256_quantity("amount", &value.into())?;
        Ok(Self(parsed.to_str_radix(10)))
    }

    /// Returns the zero quantity.
    #[must_use]
    pub fn zero() -> Self {
        Self("0".to_owned())
    }

    /// Returns the canonical decimal string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for Amount {
    fn default() -> Self {
        Self::zero()
    }
}

impl TryFrom<String> for Amount {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for Amount {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_owned())
    }
}

impl From<Amount> for String {
    fn from(value: Amount) -> Self {
        value.0
    }
}

impl From<u32> for Amount {
    fn from(value: u32) -> Self {
        Self(value.to_string())
    }
}

impl From<u64> for Amount {
    fn from(value: u64) -> Self {
        Self(value.to_string())
    }
}

impl From<usize> for Amount {
    fn from(value: usize) -> Self {
        Self(value.to_string())
    }
}

impl AsRef<str> for Amount {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Unsigned 256-bit atomic token quantity rendered in the smallest token unit.
///
/// `AtomAmount` is the forward-looking typed boundary for atomic token
/// values. The inner `BigUint` stays the authoritative storage while the
/// wire format remains the canonical base-10 string accepted by the `CoW`
/// Protocol orderbook and contract layer. For decimal-aware values that also
/// carry a scale, see [`DecimalAmount`]. Existing `Amount`-based surfaces are
/// preserved for wire compatibility; new typed code should reach for
/// `AtomAmount` and `DecimalAmount` instead.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtomAmount(BigUint);

impl AtomAmount {
    /// Returns the zero atom quantity.
    #[inline]
    #[must_use]
    pub fn zero() -> Self {
        Self(BigUint::from(0u32))
    }

    /// Creates an atomic amount from a raw `BigUint` quantity.
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
}

impl Default for AtomAmount {
    fn default() -> Self {
        Self::zero()
    }
}

impl From<BigUint> for AtomAmount {
    #[inline]
    fn from(value: BigUint) -> Self {
        Self(value)
    }
}

impl From<AtomAmount> for BigUint {
    #[inline]
    fn from(value: AtomAmount) -> Self {
        value.0
    }
}

impl From<u32> for AtomAmount {
    #[inline]
    fn from(value: u32) -> Self {
        Self(BigUint::from(value))
    }
}

impl From<u64> for AtomAmount {
    #[inline]
    fn from(value: u64) -> Self {
        Self(BigUint::from(value))
    }
}

impl From<u128> for AtomAmount {
    #[inline]
    fn from(value: u128) -> Self {
        Self(BigUint::from(value))
    }
}

impl TryFrom<&str> for AtomAmount {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_u256_quantity("atom_amount", value).map(Self)
    }
}

impl TryFrom<String> for AtomAmount {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        parse_u256_quantity("atom_amount", &value).map(Self)
    }
}

impl TryFrom<&Amount> for AtomAmount {
    type Error = CoreError;

    fn try_from(value: &Amount) -> Result<Self, Self::Error> {
        parse_u256_quantity("atom_amount", value.as_str()).map(Self)
    }
}

impl TryFrom<Amount> for AtomAmount {
    type Error = CoreError;

    fn try_from(value: Amount) -> Result<Self, Self::Error> {
        (&value).try_into()
    }
}

impl From<&AtomAmount> for Amount {
    fn from(value: &AtomAmount) -> Self {
        Self(value.0.to_str_radix(10))
    }
}

impl From<AtomAmount> for Amount {
    fn from(value: AtomAmount) -> Self {
        Self::from(&value)
    }
}

impl fmt::Display for AtomAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl Serialize for AtomAmount {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0.to_str_radix(10))
    }
}

impl<'de> Deserialize<'de> for AtomAmount {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = String::deserialize(deserializer)?;
        parse_u256_quantity("atom_amount", &value)
            .map(Self)
            .map_err(|err| serde::de::Error::custom(err.to_string()))
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

/// Canonical signed integer rendered as a base-10 string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct SignedAmount(String);

impl SignedAmount {
    /// Creates a canonical signed integer quantity.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty or cannot be parsed as a
    /// base-10 signed integer.
    pub fn new(value: impl Into<String>) -> Result<Self, CoreError> {
        let parsed = parse_signed_quantity("signed_amount", &value.into())?;
        Ok(Self(parsed.to_string()))
    }

    /// Returns the zero quantity.
    #[must_use]
    pub fn zero() -> Self {
        Self("0".to_owned())
    }

    /// Returns the canonical decimal string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
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
        value.0
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

/// Side of an order relative to the sell token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OrderKind {
    /// Buy an exact amount of the buy token.
    Buy,
    /// Sell an exact amount of the sell token.
    Sell,
}

/// Token-balance source selection used by `CoW` orders.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum OrderBalance {
    /// ERC-20 balance directly held by the owner.
    #[default]
    Erc20,
    /// External balance tracked by the settlement contract.
    External,
    /// Internal balance tracked by the settlement contract.
    Internal,
}

impl OrderBalance {
    /// Normalizes buy-balance selection to the protocol-supported value set.
    #[must_use]
    pub const fn normalize_for_buy(self) -> Self {
        match self {
            Self::Internal => Self::Internal,
            Self::Erc20 | Self::External => Self::Erc20,
        }
    }
}

/// Token metadata used by user-domain SDK surfaces.
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
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance source.
    #[serde(default)]
    pub buy_token_balance: OrderBalance,
}

impl UnsignedOrder {
    /// Returns the normalized buy-token balance that contract hashing uses.
    #[must_use]
    pub const fn normalized_buy_token_balance(&self) -> OrderBalance {
        self.buy_token_balance.normalize_for_buy()
    }

    /// Returns the canonical EIP-712 field ordering for orders.
    #[must_use]
    pub const fn field_names() -> &'static [&'static str; ORDER_TYPE_FIELD_NAMES.len()] {
        &ORDER_TYPE_FIELD_NAMES
    }
}

/// Optional order envelope used by SDK consumers that need owner or uid context
/// alongside the user-domain unsigned order.
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

/// Simplified trade execution view used by SDK consumers.
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

/// Backward-compatible alias for the user-domain trade model.
pub type TradeModel = Trade;

/// Compatibility order shape consumed by some lower-level contract helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OrderModel {
    /// Order side.
    pub kind: OrderKind,
    /// Sell token address.
    pub sell_token: Address,
    /// Buy token address.
    pub buy_token: Address,
    /// Receiver address.
    pub receiver: Address,
    /// Owner address.
    pub owner: Address,
    /// App-data hash hex string.
    pub app_data_hex: AppDataHash,
}

/// User-domain quote request shape with validated quantities.
///
/// This is not the orderbook HTTP wire DTO. The orderbook crate keeps the upstream
/// string-based transport contract explicit.
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

/// User-domain quote response with validated quantities.
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

/// Legacy serialized compatibility quote model retained for current workspace consumers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuoteModel {
    /// Quote side.
    pub kind: OrderKind,
    /// Sell amount as a stringly typed compatibility value.
    pub sell_amount: String,
    /// Buy amount as a stringly typed compatibility value.
    pub buy_amount: String,
    /// Fee amount as a stringly typed compatibility value.
    pub fee_amount: String,
    /// Optional order UID when present in compatibility paths.
    pub order_uid: Option<OrderUid>,
}

/// Generic sell/buy amount pair.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amounts<T> {
    /// Sell-side amount.
    pub sell_amount: T,
    /// Buy-side amount.
    pub buy_amount: T,
}

/// Network-fee amounts expressed in both quote currencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFee<T> {
    /// Network fee expressed in sell-token units.
    pub amount_in_sell_currency: T,
    /// Network fee expressed in buy-token units.
    pub amount_in_buy_currency: T,
}

/// Generic fee component represented by amount and basis points.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeComponent<T> {
    /// Fee amount.
    pub amount: T,
    /// Fee in basis points.
    pub bps: u32,
}

/// Full quote cost breakdown.
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
