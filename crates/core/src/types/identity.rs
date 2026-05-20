use std::borrow::Cow;
use std::fmt::{self, Write as FmtWrite};
use std::str::FromStr;

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
    /// Escape hatch only: prefer [`Address::as_alloy`] (borrowed) or
    /// [`Address::into_alloy`] (owned) for forward compatibility per
    /// ADR 0052. The `.0` field is `pub` to match the canonical
    /// [`alloy_primitives::wrap_fixed_bytes!`] pattern and to keep the
    /// `#[repr(transparent)]` bit-for-bit layout contract visible at the
    /// type system, but it is not part of the long-term API contract. A
    /// future cascade may seal this field through a documented deprecation
    /// cycle if a runtime validation invariant requires it; consumers who
    /// rely on `.0` accept the forward-compatibility risk.
    pub AlloyAddress,
);

impl Address {
    /// Raw decoded byte length of an EVM address.
    pub const BYTE_LENGTH: usize = 20;

    /// Creates a validated address from a `0x`-prefixed hexadecimal string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        validate_hex_field("address", value, EVM_ADDRESS_HEX_CHARS)?;
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
        Ok(Self(AlloyAddress::from(array)))
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

    /// Returns the zero address.
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self(AlloyAddress::ZERO)
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

    /// Returns the lowercase key form used for case-insensitive comparisons.
    ///
    /// The cow `Address` already serializes to the canonical lowercase form,
    /// so this accessor returns the same string as [`Address::to_hex_string`]
    /// and is preserved for callers that historically routed through it.
    #[inline]
    #[must_use]
    pub fn normalized_key(&self) -> String {
        self.to_hex_string()
    }
}

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
    /// Escape hatch only: prefer [`HexData::as_alloy`] (borrowed) or
    /// [`HexData::into_alloy`] (owned) for forward compatibility per
    /// ADR 0052. The `.0` field is `pub` to match the canonical
    /// [`alloy_primitives`] pattern and to keep the
    /// `#[repr(transparent)]` bit-for-bit layout contract visible at the
    /// type system, but it is not part of the long-term API contract. A
    /// future cascade may seal this field through a documented deprecation
    /// cycle if a runtime validation invariant requires it; consumers who
    /// rely on `.0` accept the forward-compatibility risk.
    pub Bytes,
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
        let bytes = hex::decode(normalized.as_ref())
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
    /// Escape hatch only: prefer [`AppDataHash::as_alloy`] (borrowed) or
    /// [`AppDataHash::into_alloy`] (owned) for forward compatibility per
    /// ADR 0052. The `.0` field is `pub` to match the canonical
    /// [`alloy_primitives::wrap_fixed_bytes!`] pattern and to keep the
    /// `#[repr(transparent)]` bit-for-bit layout contract visible at the
    /// type system, but it is not part of the long-term API contract. A
    /// future cascade may seal this field through a documented
    /// deprecation cycle if a runtime validation invariant requires it;
    /// consumers who rely on `.0` accept the forward-compatibility risk.
    pub B256,
);

impl AppDataHash {
    /// Raw decoded byte length of an app-data hash.
    pub const BYTE_LENGTH: usize = 32;

    /// Creates a validated app-data hash from a `0x`-prefixed 32-byte hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        validate_hex_field("app_data_hash", value, APP_DATA_HASH_HEX_CHARS)?;
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
        Ok(Self(B256::from(array)))
    }

    /// Creates an app-data hash from its raw 32-byte representation.
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(B256::new(bytes))
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

    /// Returns the zero app-data hash.
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self(B256::ZERO)
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

    /// Returns the canonical `CIDv1` multibase representation of this
    /// app-data hash.
    ///
    /// The cow protocol emits app-data CIDs as the `cidv1 + raw + keccak-256`
    /// triple, multibase-encoded in `base16` lowercase per ADR 0044.
    /// The serialised form is `f` (the base16-lowercase multibase prefix)
    /// followed by `01 55 1b 20` (CID version 1, raw codec, keccak-256
    /// multihash code, 32-byte digest length) and the 32 bytes of the
    /// inner [`alloy_primitives::B256`] hash. The cow protocol layer
    /// integration at `crates/app-data/src/cid.rs` produces the same
    /// string for the same input digest.
    #[must_use]
    pub fn to_cid(&self) -> String {
        let mut cid_bytes = [0u8; 4 + Self::BYTE_LENGTH];
        cid_bytes[0] = 0x01; // CID version 1
        cid_bytes[1] = 0x55; // raw multicodec
        cid_bytes[2] = 0x1b; // keccak-256 multihash code
        cid_bytes[3] = 0x20; // 32-byte multihash digest length
        cid_bytes[4..].copy_from_slice(self.as_slice());
        format!("f{}", hex::encode(cid_bytes))
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
    /// Escape hatch only: prefer [`Hash32::as_alloy`] (borrowed) or
    /// [`Hash32::into_alloy`] (owned) for forward compatibility per
    /// ADR 0052. The `.0` field is `pub` to match the canonical
    /// [`alloy_primitives::wrap_fixed_bytes!`] pattern and to keep the
    /// `#[repr(transparent)]` bit-for-bit layout contract visible at the
    /// type system, but it is not part of the long-term API contract. A
    /// future cascade may seal this field through a documented deprecation
    /// cycle if a runtime validation invariant requires it; consumers who
    /// rely on `.0` accept the forward-compatibility risk.
    pub B256,
);

impl Hash32 {
    /// Raw decoded byte length of a 32-byte hash.
    pub const BYTE_LENGTH: usize = 32;

    /// Creates a validated 32-byte hash from a `0x`-prefixed hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        validate_hex_field("hash32", value, HASH32_HEX_CHARS)?;
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
        Ok(Self(B256::from(array)))
    }

    /// Creates a 32-byte hash from its raw 32-byte representation.
    #[inline]
    #[must_use]
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(B256::new(bytes))
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

    /// Returns the zero hash.
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self(B256::ZERO)
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
    /// Escape hatch only: prefer [`OrderUid::as_alloy`] (borrowed) or
    /// [`OrderUid::into_alloy`] (owned) for forward compatibility per
    /// ADR 0052. The `.0` field is `pub` to match the canonical
    /// [`alloy_primitives::wrap_fixed_bytes!`] pattern and to keep the
    /// `#[repr(transparent)]` bit-for-bit layout contract visible at the
    /// type system, but it is not part of the long-term API contract. A
    /// future cascade may seal this field through a documented deprecation
    /// cycle if a runtime validation invariant requires it; consumers who
    /// rely on `.0` accept the forward-compatibility risk.
    pub FixedBytes<56>,
);

impl OrderUid {
    /// Raw decoded byte length of an order UID.
    pub const BYTE_LENGTH: usize = 56;

    /// Creates a validated order UID from a `0x`-prefixed 56-byte hex string.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed, has
    /// the wrong length, or contains non-hex characters.
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        validate_hex_field("order_uid", value, ORDER_UID_HEX_CHARS)?;
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
        Ok(Self(FixedBytes::<56>::from(array)))
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

    /// Returns the zero UID.
    #[inline]
    #[must_use]
    pub const fn zero() -> Self {
        Self(FixedBytes::<56>::ZERO)
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
