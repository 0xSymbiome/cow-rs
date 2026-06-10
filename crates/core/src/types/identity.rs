use std::borrow::Cow;
use std::fmt::{self, Write as FmtWrite};
use std::str::FromStr;

use alloy_primitives::hex::FromHexError;
use alloy_primitives::{Address as AlloyAddress, B256, Bytes, FixedBytes};
use serde::{Deserialize, Serialize};

use crate::errors::{CoreError, ValidationError};

/// Hex character count for an EVM address without the `0x` prefix.
const EVM_ADDRESS_HEX_CHARS: usize = 40;
/// Hex character count for a 32-byte app-data hash without the `0x` prefix.
const APP_DATA_HASH_HEX_CHARS: usize = 64;
/// Hex character count for an order UID without the `0x` prefix.
const ORDER_UID_HEX_CHARS: usize = 112;
/// Hex character count for a 32-byte hash without the `0x` prefix.
const HASH32_HEX_CHARS: usize = 64;

/// Numeric EVM chain id.
pub type ChainId = u64;

// --- Address ---------------------------------------------------------------

/// Validated EVM address.
///
/// The wire form is the protocol-canonical `0x`-prefixed 42-character
/// lowercase hexadecimal string. The newtype is `#[repr(transparent)]` over
/// [`alloy_primitives::Address`], so the in-memory layout is bit-for-bit
/// identical to the alloy primitive and conversion at the alloy seam is free
/// at runtime through [`Address::as_alloy`] (borrowed), [`Address::into_alloy`]
/// (owned), or [`From`] / [`Into`].
///
/// `Address` carries cow-owned [`fmt::Display`], [`Serialize`], and
/// [`Deserialize`] impls because alloy's default `Display` for
/// [`alloy_primitives::Address`] emits the EIP-55 mixed-case checksum form,
/// while the cow protocol wire form is lowercase. The cow `Display` impl
/// writes `format!("{:#x}", self.0)` which routes through alloy's
/// [`fmt::LowerHex`] impl and emits lowercase 0x-prefixed hex.
///
/// [`PartialEq`], [`Eq`], [`Hash`](std::hash::Hash), [`PartialOrd`], and
/// [`Ord`] derive from the inner alloy primitive, which compares addresses on
/// the packed 20-byte representation.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default)]
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[cfg_attr(
    target_family = "wasm",
    tsify(into_wasm_abi, from_wasm_abi, type = "string")
)]
pub struct Address(
    // Private inner: the constructors (`new` / `from_*` / `From`) and the
    // `as_alloy` / `into_alloy` accessors are the entire contract, so a future
    // runtime invariant can land without breaking consumers (ADR 0052).
    AlloyAddress,
);

impl Address {
    /// Raw decoded byte length of an EVM address.
    pub const BYTE_LENGTH: usize = 20;

    /// Canonical zero address (`0x00…00`).
    pub const ZERO: Self = Self(AlloyAddress::ZERO);

    /// Creates a validated address from a `0x`-prefixed hexadecimal string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed
    /// (lowercase), has the wrong length, or contains non-hex characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(ValidationError::EmptyField { field: "address" }.into());
        }
        if !value.starts_with("0x") {
            return Err(ValidationError::InvalidHexPrefix { field: "address" }.into());
        }
        let inner = AlloyAddress::from_str(value)
            .map_err(|e| classify_alloy_hex_error("address", EVM_ADDRESS_HEX_CHARS, e))?;
        Ok(Self(inner))
    }

    /// Creates an address from its raw 20-byte representation.
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 20]) -> Self {
        Self(AlloyAddress::new(bytes))
    }

    /// Returns the canonical lowercase 0x-prefixed hex form as an owned
    /// [`String`].
    ///
    /// Follows the Rust stdlib naming convention: `to_*` returns an owned
    /// value; `as_*` returns a borrow. For the allocation-free path that
    /// writes the canonical form into a caller-provided formatter without
    /// intermediate allocation, see [`Address::write_into`].
    #[inline]
    #[must_use]
    pub fn to_hex_string(&self) -> String {
        format!("{:#x}", self.0)
    }

    /// Writes the canonical lowercase 0x-prefixed hex form into the given
    /// formatter without allocating.
    ///
    /// Equivalent to `write!(f, "{}", self)` via the cow newtype's
    /// [`fmt::Display`] impl, but named explicitly so hot-path callers
    /// (URL templating with reusable buffers, tracing field encoders,
    /// batch log formatters) can target the allocation-free path.
    ///
    /// # Errors
    ///
    /// Returns the same [`fmt::Error`] the caller's formatter raises; the
    /// cow body itself is infallible.
    #[inline]
    pub fn write_into(&self, f: &mut impl FmtWrite) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }

    /// Returns the raw 20 bytes of the address as a borrowed slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Returns the underlying packed [`alloy_primitives::Address`].
    ///
    /// Use this accessor when handing the address to an
    /// `alloy_primitives`-typed surface (signing, contract bindings, RPC
    /// adapters) without re-parsing the lowercase hex string.
    #[inline]
    #[must_use]
    pub const fn as_alloy(&self) -> &AlloyAddress {
        &self.0
    }

    /// Returns the underlying packed [`alloy_primitives::Address`] by value.
    #[inline]
    #[must_use]
    pub const fn into_alloy(self) -> AlloyAddress {
        self.0
    }

    /// Returns `true` when this is the zero address.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == AlloyAddress::ZERO
    }

    /// Returns the fixed decoded byte length of an EVM address.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        Self::BYTE_LENGTH
    }
}

// DO NOT SWAP for #[derive(Display)] or alloy's default Address Display.
//
// cow-rs wire form for Address is lowercase (`0x` followed by 40
// lowercase hex digits) because every parity fixture under
// `parity/fixtures/`, every services-backend response, and every
// EIP-712 JSON-stringified payload uses lowercase. alloy's default
// Display emits the EIP-55 mixed-case checksum.
//
// Swapping to derived Display or calling `.to_checksum()` would diff
// every parity fixture on hash, falsely report mismatches against
// lowercase-emitting tools, and silently change the EIP-712 digest
// where address strings get hashed for transport.
//
// The `{:#x}` format spec routes through alloy's `LowerHex` impl,
// which emits the lowercase byte sequence we depend on; keep it.
//
// ADR: docs/adr/0052-alloy-primitives-canonical-primitive-layer.md
// (lines 96-99).
// Doctrine: docs/alloy-doctrine.md, Bucket 2 row for `Address::Display`
// lowercase emission.
// Enforced by cargo check-source-fences (xtask/src/policy/fences.rs).
impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }
}

impl Serialize for Address {
    fn serialize<S: serde::Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D: serde::Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        let value = <Cow<'_, str>>::deserialize(de)?;
        Self::new(value.as_ref()).map_err(serde::de::Error::custom)
    }
}

impl From<AlloyAddress> for Address {
    #[inline]
    fn from(value: AlloyAddress) -> Self {
        Self(value)
    }
}

impl From<Address> for AlloyAddress {
    #[inline]
    fn from(value: Address) -> Self {
        value.0
    }
}

impl FromStr for Address {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for Address {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl TryFrom<&str> for Address {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

// --- HexData ---------------------------------------------------------------

/// Validated hex payload used for calldata and byte blobs.
///
/// The wire form is the protocol-canonical `0x`-prefixed lowercase
/// hexadecimal string. The newtype is `#[repr(transparent)]` over
/// [`alloy_primitives::Bytes`] and forwards `Display`/`Serialize`/
/// `Deserialize` to the inner alloy type, whose canonical defaults already
/// emit the cow lowercase wire form. Odd-length inputs are left-padded with
/// one zero nibble during construction so the stored value remains
/// byte-aligned hex.
#[repr(transparent)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize)]
#[serde(transparent)]
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[cfg_attr(
    target_family = "wasm",
    tsify(into_wasm_abi, from_wasm_abi, type = "string")
)]
pub struct HexData(
    // Private inner: the constructors (`new` / `from_*` / `From`) and the
    // `as_alloy` / `into_alloy` accessors are the entire contract, so a future
    // runtime invariant can land without breaking consumers (ADR 0052).
    Bytes,
);

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
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(ValidationError::EmptyField { field: "hex_data" }.into());
        }
        let stripped = value
            .strip_prefix("0x")
            .ok_or(ValidationError::InvalidHexPrefix { field: "hex_data" })?;
        let normalized: Cow<'_, str> = if stripped.len() % 2 == 1 {
            Cow::Owned(format!("0{stripped}"))
        } else {
            Cow::Borrowed(stripped)
        };
        let bytes = alloy_primitives::hex::decode(normalized.as_ref())
            .map_err(|_| ValidationError::InvalidHexCharacters { field: "hex_data" })?;
        Ok(Self(Bytes::from(bytes)))
    }

    /// Creates hex data from a raw byte buffer (no hex parsing).
    #[inline]
    #[must_use]
    pub fn from_bytes(bytes: impl Into<Bytes>) -> Self {
        Self(bytes.into())
    }

    /// Returns the canonical empty payload (`"0x"`).
    #[inline]
    #[must_use]
    pub const fn empty() -> Self {
        Self(Bytes::new())
    }

    /// Returns the canonical lowercase 0x-prefixed hex form as an owned
    /// [`String`].
    ///
    /// Follows the Rust stdlib naming convention: `to_*` returns an owned
    /// value; `as_*` returns a borrow. For the allocation-free path that
    /// writes the canonical form into a caller-provided formatter without
    /// intermediate allocation, see [`HexData::write_into`].
    #[inline]
    #[must_use]
    pub fn to_hex_string(&self) -> String {
        format!("{:#x}", self.0)
    }

    /// Writes the canonical lowercase 0x-prefixed hex form into the given
    /// formatter without allocating.
    ///
    /// # Errors
    ///
    /// Returns the same [`fmt::Error`] the caller's formatter raises; the
    /// cow body itself is infallible.
    #[inline]
    pub fn write_into(&self, f: &mut impl FmtWrite) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }

    /// Returns the raw bytes of the payload as a borrowed slice.
    #[inline]
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.0.as_ref()
    }

    /// Returns the underlying [`alloy_primitives::Bytes`] payload.
    #[inline]
    #[must_use]
    pub const fn as_alloy(&self) -> &Bytes {
        &self.0
    }

    /// Returns the underlying [`alloy_primitives::Bytes`] payload by value.
    #[inline]
    #[must_use]
    pub fn into_alloy(self) -> Bytes {
        self.0
    }

    /// Returns `true` when the payload is empty.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Returns the decoded byte length of the payload.
    #[inline]
    #[must_use]
    pub fn byte_length(&self) -> usize {
        self.0.len()
    }
}

impl fmt::Display for HexData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<Bytes> for HexData {
    #[inline]
    fn from(value: Bytes) -> Self {
        Self(value)
    }
}

impl From<HexData> for Bytes {
    #[inline]
    fn from(value: HexData) -> Self {
        value.0
    }
}

impl FromStr for HexData {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for HexData {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl TryFrom<&str> for HexData {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

// --- AppDataHash ----------------------------------------------------------

/// Validated 32-byte app-data hash.
///
/// The wire form is the protocol-canonical `0x`-prefixed 66-character
/// lowercase hexadecimal string. The newtype is `#[repr(transparent)]`
/// over [`alloy_primitives::B256`], so the in-memory layout is
/// bit-for-bit identical to the alloy primitive and conversion at the
/// alloy seam is free at runtime through [`AppDataHash::as_alloy`]
/// (borrowed), [`AppDataHash::into_alloy`] (owned), or [`From`] /
/// [`Into`].
///
/// `AppDataHash` forwards [`Serialize`] / [`Deserialize`] to the inner
/// [`alloy_primitives::B256`] via `#[serde(transparent)]` because the
/// alloy lowercase 0x-prefixed default already matches the cow wire
/// form. [`fmt::Display`] is a one-line delegate to the inner primitive
/// for the same reason.
///
/// Equality, hash, and ordering derive from the packed 32-byte
/// representation, which is equivalent to the documented
/// case-insensitive comparison contract because every valid value parses
/// to the same bytes regardless of input casing.
#[doc(alias = "app-data")]
#[doc(alias = "AppData")]
#[repr(transparent)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[cfg_attr(
    target_family = "wasm",
    tsify(into_wasm_abi, from_wasm_abi, type = "string")
)]
pub struct AppDataHash(
    // Private inner: the constructors (`new` / `from_*` / `From`) and the
    // `as_alloy` / `into_alloy` accessors are the entire contract, so a future
    // runtime invariant can land without breaking consumers (ADR 0052).
    B256,
);

impl AppDataHash {
    /// Raw decoded byte length of an app-data hash.
    pub const BYTE_LENGTH: usize = 32;

    /// Canonical zero app-data hash (32 zero bytes).
    pub const ZERO: Self = Self(B256::ZERO);

    /// Creates a validated app-data hash from a `0x`-prefixed 32-byte hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed
    /// (lowercase), has the wrong length, or contains non-hex characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(ValidationError::EmptyField {
                field: "app_data_hash",
            }
            .into());
        }
        if !value.starts_with("0x") {
            return Err(ValidationError::InvalidHexPrefix {
                field: "app_data_hash",
            }
            .into());
        }
        let inner = B256::from_str(value)
            .map_err(|e| classify_alloy_hex_error("app_data_hash", APP_DATA_HASH_HEX_CHARS, e))?;
        Ok(Self(inner))
    }

    /// Creates an app-data hash from its raw 32-byte representation.
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(B256::new(bytes))
    }

    /// Computes the canonical [`AppDataHash`] for a serialized app-data document.
    ///
    /// Returns `AppDataHash(keccak256(full_app_data.as_bytes()))`. The hashing
    /// is byte-wise: the caller is responsible for serializing the document to
    /// its canonical form before passing the string. Two semantically-equal
    /// JSON documents with different key orderings hash to different values.
    /// Use the canonical serializer in `cow-sdk-app-data` to produce the
    /// canonical form before computing this hash.
    ///
    /// Equivalent to but cheaper than going through hex encoding and re-parsing
    /// via [`AppDataHash::new`].
    #[inline]
    #[must_use]
    pub fn from_full_app_data(full_app_data: &str) -> Self {
        Self(alloy_primitives::keccak256(full_app_data.as_bytes()))
    }

    /// Returns the canonical lowercase 0x-prefixed hex form as an owned
    /// [`String`].
    ///
    /// Follows the Rust stdlib naming convention: `to_*` returns an owned
    /// value; `as_*` returns a borrow. For the allocation-free path that
    /// writes the canonical form into a caller-provided formatter without
    /// intermediate allocation, see [`AppDataHash::write_into`].
    #[inline]
    #[must_use]
    pub fn to_hex_string(&self) -> String {
        format!("{:#x}", self.0)
    }

    /// Writes the canonical lowercase 0x-prefixed hex form into the given
    /// formatter without allocating.
    ///
    /// # Errors
    ///
    /// Returns the same [`fmt::Error`] the caller's formatter raises; the
    /// cow body itself is infallible.
    #[inline]
    pub fn write_into(&self, f: &mut impl FmtWrite) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }

    /// Returns the raw 32 bytes of the hash as a borrowed slice.
    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Returns the underlying packed [`alloy_primitives::B256`].
    ///
    /// Use this accessor when handing the hash to an
    /// `alloy_primitives`-typed surface (contract bindings, EIP-712 hash
    /// composition) without re-parsing the lowercase hex string.
    #[inline]
    #[must_use]
    pub const fn as_alloy(&self) -> &B256 {
        &self.0
    }

    /// Returns the underlying packed [`alloy_primitives::B256`] by value.
    #[inline]
    #[must_use]
    pub const fn into_alloy(self) -> B256 {
        self.0
    }

    /// Returns `true` when this is the zero app-data hash.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == B256::ZERO
    }

    /// Returns the fixed decoded byte length of an app-data hash.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        Self::BYTE_LENGTH
    }
}

impl From<B256> for AppDataHash {
    #[inline]
    fn from(value: B256) -> Self {
        Self(value)
    }
}

impl From<AppDataHash> for B256 {
    #[inline]
    fn from(value: AppDataHash) -> Self {
        value.0
    }
}

impl FromStr for AppDataHash {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for AppDataHash {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl TryFrom<&str> for AppDataHash {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl fmt::Display for AppDataHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

/// Backward-compatible alias for the app-data hash hex representation.
pub type AppDataHex = AppDataHash;

// --- Hash32 -----------------------------------------------------------------

/// Generic validated 32-byte hash wrapper for user-domain and contract surfaces.
///
/// The wire form is the protocol-canonical `0x`-prefixed 66-character
/// lowercase hexadecimal string. The newtype is `#[repr(transparent)]` over
/// [`alloy_primitives::B256`] and forwards `Display`/`Serialize`/
/// `Deserialize` to the inner alloy type, whose canonical defaults already
/// emit the cow lowercase wire form.
#[repr(transparent)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[cfg_attr(
    target_family = "wasm",
    tsify(into_wasm_abi, from_wasm_abi, type = "string")
)]
pub struct Hash32(
    // Private inner: the constructors (`new` / `from_*` / `From`) and the
    // `as_alloy` / `into_alloy` accessors are the entire contract, so a future
    // runtime invariant can land without breaking consumers (ADR 0052).
    B256,
);

impl Hash32 {
    /// Raw decoded byte length of a 32-byte hash.
    pub const BYTE_LENGTH: usize = 32;

    /// Canonical zero 32-byte hash (32 zero bytes).
    pub const ZERO: Self = Self(B256::ZERO);

    /// Creates a validated 32-byte hash from a `0x`-prefixed hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed
    /// (lowercase), has the wrong length, or contains non-hex characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(ValidationError::EmptyField { field: "hash32" }.into());
        }
        if !value.starts_with("0x") {
            return Err(ValidationError::InvalidHexPrefix { field: "hash32" }.into());
        }
        let inner = B256::from_str(value)
            .map_err(|e| classify_alloy_hex_error("hash32", HASH32_HEX_CHARS, e))?;
        Ok(Self(inner))
    }

    /// Creates a 32-byte hash from its raw 32-byte representation.
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(B256::new(bytes))
    }

    /// Creates the 32-byte topic form of an indexed `address` event argument.
    ///
    /// An EVM indexed address is encoded as a 32-byte topic with the 20 address
    /// bytes right-aligned and the high 12 bytes zeroed. Use it to filter an
    /// indexed-address argument server-side — for example the `owner` of a `CoW`
    /// settlement `Trade` event:
    /// `LogQuery::new(from, to).with_topic1(Hash32::from_indexed_address(&owner))`.
    #[inline]
    #[must_use]
    pub fn from_indexed_address(address: &Address) -> Self {
        Self(B256::left_padding_from(address.as_alloy().as_slice()))
    }

    /// Returns the canonical lowercase 0x-prefixed hex form as an owned
    /// [`String`].
    #[inline]
    #[must_use]
    pub fn to_hex_string(&self) -> String {
        format!("{:#x}", self.0)
    }

    /// Writes the canonical lowercase 0x-prefixed hex form into the given
    /// formatter without allocating.
    ///
    /// # Errors
    ///
    /// Returns the same [`fmt::Error`] the caller's formatter raises; the
    /// cow body itself is infallible.
    #[inline]
    pub fn write_into(&self, f: &mut impl FmtWrite) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }

    /// Returns the raw 32 bytes of the hash as a borrowed slice.
    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Returns the underlying [`alloy_primitives::B256`] hash.
    #[inline]
    #[must_use]
    pub const fn as_alloy(&self) -> &B256 {
        &self.0
    }

    /// Returns the underlying [`alloy_primitives::B256`] hash by value.
    #[inline]
    #[must_use]
    pub const fn into_alloy(self) -> B256 {
        self.0
    }

    /// Returns `true` when this is the zero hash.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == B256::ZERO
    }

    /// Returns the fixed decoded byte length of a 32-byte hash.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        Self::BYTE_LENGTH
    }
}

impl fmt::Display for Hash32 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<B256> for Hash32 {
    #[inline]
    fn from(value: B256) -> Self {
        Self(value)
    }
}

impl From<Hash32> for B256 {
    #[inline]
    fn from(value: Hash32) -> Self {
        value.0
    }
}

impl FromStr for Hash32 {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for Hash32 {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl TryFrom<&str> for Hash32 {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

/// Transaction hash alias.
pub type TransactionHash = Hash32;
/// Block hash alias.
pub type BlockHash = Hash32;
/// Order digest alias.
pub type OrderDigest = Hash32;

// --- OrderUid ---------------------------------------------------------------

/// Validated `CoW` order UID.
///
/// The wire form is the protocol-canonical `0x`-prefixed 114-character
/// lowercase hexadecimal string. The newtype is `#[repr(transparent)]` over
/// [`alloy_primitives::FixedBytes<56>`] and forwards `Display`/`Serialize`/
/// `Deserialize` to the inner alloy type, whose canonical defaults already
/// emit the cow lowercase wire form.
#[doc(alias = "UID")]
#[doc(alias = "Uid")]
#[doc(alias = "order-id")]
#[repr(transparent)]
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
#[cfg_attr(target_family = "wasm", derive(tsify::Tsify))]
#[cfg_attr(
    target_family = "wasm",
    tsify(into_wasm_abi, from_wasm_abi, type = "string")
)]
pub struct OrderUid(
    // Private inner: the constructors (`new` / `from_*` / `From`) and the
    // `as_alloy` / `into_alloy` accessors are the entire contract, so a future
    // runtime invariant can land without breaking consumers (ADR 0052).
    FixedBytes<56>,
);

impl OrderUid {
    /// Raw decoded byte length of an order UID.
    pub const BYTE_LENGTH: usize = 56;

    /// Canonical zero order UID (56 zero bytes).
    pub const ZERO: Self = Self(FixedBytes::<56>::ZERO);

    /// Creates a validated order UID from a `0x`-prefixed 56-byte hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed
    /// (lowercase), has the wrong length, or contains non-hex characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(ValidationError::EmptyField { field: "order_uid" }.into());
        }
        if !value.starts_with("0x") {
            return Err(ValidationError::InvalidHexPrefix { field: "order_uid" }.into());
        }
        let inner = FixedBytes::<56>::from_str(value)
            .map_err(|e| classify_alloy_hex_error("order_uid", ORDER_UID_HEX_CHARS, e))?;
        Ok(Self(inner))
    }

    /// Creates an order UID from its raw 56-byte representation.
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 56]) -> Self {
        Self(FixedBytes::<56>::new(bytes))
    }

    /// Returns the canonical lowercase 0x-prefixed hex form as an owned
    /// [`String`].
    #[inline]
    #[must_use]
    pub fn to_hex_string(&self) -> String {
        format!("{:#x}", self.0)
    }

    /// Writes the canonical lowercase 0x-prefixed hex form into the given
    /// formatter without allocating.
    ///
    /// # Errors
    ///
    /// Returns the same [`fmt::Error`] the caller's formatter raises; the
    /// cow body itself is infallible.
    #[inline]
    pub fn write_into(&self, f: &mut impl FmtWrite) -> fmt::Result {
        write!(f, "{:#x}", self.0)
    }

    /// Returns the raw 56 bytes of the UID as a borrowed slice.
    #[inline]
    #[must_use]
    pub const fn as_slice(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Returns the underlying [`alloy_primitives::FixedBytes<56>`] UID.
    #[inline]
    #[must_use]
    pub const fn as_alloy(&self) -> &FixedBytes<56> {
        &self.0
    }

    /// Returns the underlying [`alloy_primitives::FixedBytes<56>`] UID by value.
    #[inline]
    #[must_use]
    pub const fn into_alloy(self) -> FixedBytes<56> {
        self.0
    }

    /// Returns `true` when this is the zero UID.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == FixedBytes::<56>::ZERO
    }

    /// Returns the fixed decoded byte length of an order UID.
    #[inline]
    #[must_use]
    pub const fn byte_length(&self) -> usize {
        Self::BYTE_LENGTH
    }
}

impl fmt::Display for OrderUid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

impl From<FixedBytes<56>> for OrderUid {
    #[inline]
    fn from(value: FixedBytes<56>) -> Self {
        Self(value)
    }
}

impl From<OrderUid> for FixedBytes<56> {
    #[inline]
    fn from(value: OrderUid) -> Self {
        value.0
    }
}

impl FromStr for OrderUid {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

impl TryFrom<String> for OrderUid {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl TryFrom<&str> for OrderUid {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

// --- Private validation helpers --------------------------------------------

/// Maps an [`alloy_primitives::hex::FromHexError`] returned by a
/// fixed-width hex parser onto the cow [`ValidationError`] taxonomy.
///
/// The classifier intentionally constructs the cow variant by `match`
/// rather than lifting the alloy error through `#[from]`/`?`. The
/// alloy `FromHexError::InvalidHexCharacter { c: char, index: usize }`
/// payload carries one byte of the user-supplied input plus its offset;
/// the cow error boundary drops both fields so neither `Display` nor
/// `Debug` rendering can leak any portion of a caller-supplied secret.
/// The `crates/sdk/tests/error_redaction_contract.rs` redaction sentinel
/// pins this contract by constructing a hex-rejecting input through
/// [`Address::new`] and asserting the rendered error contains neither
/// the offending character nor the literal `"index"`.
const fn classify_alloy_hex_error(
    field: &'static str,
    expected: usize,
    error: FromHexError,
) -> ValidationError {
    match error {
        FromHexError::InvalidHexCharacter { .. } => ValidationError::InvalidHexCharacters { field },
        FromHexError::OddLength | FromHexError::InvalidStringLength => {
            ValidationError::InvalidHexLength { field, expected }
        }
    }
}
