use std::borrow::Cow;
use std::fmt;
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

/// Declares a fixed-width, `#[repr(transparent)]` hex-newtype over an alloy
/// fixed-bytes primitive whose lowercase `0x`-prefixed canonical form already
/// matches the cow wire form.
///
/// Emits the uniform constructor / accessor / conversion / `serde(transparent)`
/// surface shared by [`AppDataHash`], [`Hash32`], and [`OrderUid`]. [`Address`]
/// keeps a hand-written lowercase `serde` (alloy's default is the EIP-55
/// checksum form, which would shift the wire bytes), and [`HexData`] is
/// variable-length, so neither is built through this macro.
macro_rules! hex_newtype {
    (
        $(#[$meta:meta])*
        $name:ident($inner:ty, $bytes:literal, field = $field:literal, hex_chars = $hex_chars:expr $(,)?)
    ) => {
        $(#[$meta])*
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
        pub struct $name(
            // Private inner: the constructors (`new` / `from_*` / `From`) and the
            // `as_alloy` / `into_alloy` accessors are the entire contract, so a
            // future runtime invariant can land without breaking consumers (ADR 0052).
            $inner,
        );

        impl $name {
            /// Raw decoded byte length of this value.
            pub const BYTE_LENGTH: usize = $bytes;

            /// Canonical zero value (all bytes zero).
            pub const ZERO: Self = Self(<$inner>::ZERO);

            /// Creates a validated value from a `0x`-prefixed lowercase hex string.
            ///
            /// # Errors
            ///
            /// Returns [`CoreError`] when the input is empty, not `0x`-prefixed
            /// (lowercase), has the wrong length, or contains non-hex characters.
            pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
                let value = value.as_ref();
                if value.is_empty() {
                    return Err(ValidationError::EmptyField { field: $field }.into());
                }
                if !value.starts_with("0x") {
                    return Err(ValidationError::InvalidHexPrefix { field: $field }.into());
                }
                let inner = <$inner>::from_str(value)
                    .map_err(|e| classify_alloy_hex_error($field, $hex_chars, e))?;
                Ok(Self(inner))
            }

            /// Creates a value from its raw byte representation.
            #[inline]
            #[must_use]
            pub const fn from_bytes(bytes: [u8; $bytes]) -> Self {
                Self(<$inner>::new(bytes))
            }

            /// Returns the canonical lowercase 0x-prefixed hex form as an owned
            /// [`String`].
            ///
            /// Follows the Rust stdlib naming convention: `to_*` returns an owned
            /// value; `as_*` returns a borrow.
            #[inline]
            #[must_use]
            pub fn to_hex_string(&self) -> String {
                format!("{:#x}", self.0)
            }

            /// Returns the raw bytes as a borrowed slice.
            #[inline]
            #[must_use]
            pub const fn as_slice(&self) -> &[u8] {
                self.0.as_slice()
            }

            /// Returns the underlying packed alloy primitive.
            ///
            /// Use this accessor when handing the value to an
            /// `alloy_primitives`-typed surface without re-parsing the hex string.
            #[inline]
            #[must_use]
            pub const fn as_alloy(&self) -> &$inner {
                &self.0
            }

            /// Returns the underlying packed alloy primitive by value.
            #[inline]
            #[must_use]
            pub const fn into_alloy(self) -> $inner {
                self.0
            }

            /// Returns `true` when this is the zero value.
            #[inline]
            #[must_use]
            pub fn is_zero(&self) -> bool {
                self.0 == <$inner>::ZERO
            }
        }

        impl From<$inner> for $name {
            #[inline]
            fn from(value: $inner) -> Self {
                Self(value)
            }
        }

        impl From<$name> for $inner {
            #[inline]
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl FromStr for $name {
            type Err = CoreError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Self::new(s)
            }
        }

        impl TryFrom<String> for $name {
            type Error = CoreError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value.as_str())
            }
        }

        impl TryFrom<&str> for $name {
            type Error = CoreError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

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

    /// Creates an address from an already-validated [`alloy_primitives::Address`].
    ///
    /// Const counterpart of the `From` conversion, completing the
    /// [`Address::as_alloy`] / [`Address::into_alloy`] accessor family for
    /// const contexts such as the [`address!`](crate::address) literal macro.
    #[inline]
    #[must_use]
    pub const fn from_alloy(inner: AlloyAddress) -> Self {
        Self(inner)
    }

    /// Returns the canonical lowercase 0x-prefixed hex form as an owned
    /// [`String`].
    ///
    /// Follows the Rust stdlib naming convention: `to_*` returns an owned
    /// value; `as_*` returns a borrow.
    #[inline]
    #[must_use]
    pub fn to_hex_string(&self) -> String {
        format!("{:#x}", self.0)
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
}

// Hand-written rather than `#[derive(Display)]` or `.to_checksum()`: the cow
// wire form for Address is lowercase, but alloy's default Display emits the
// EIP-55 mixed-case checksum. Deriving it would diff every parity fixture on
// hash, misreport mismatches against lowercase-emitting tools, and shift the
// EIP-712 digest wherever address strings are hashed. The `{:#x}` spec routes
// through alloy's lowercase `LowerHex` impl.
//
// ADR 0052. Enforced by cargo check-source-fences.
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

/// Constructs a compile-time validated [`Address`] from a `0x`-prefixed
/// hexadecimal literal, mirroring `alloy_primitives::address!`.
///
/// Malformed hex, a wrong length, or a mixed-case literal reject at build
/// time, so well-known addresses can live in `const` items without a
/// runtime [`Address::new`] call and `?` at every use site. The literal
/// must use the protocol-canonical lowercase wire form: an EIP-55 checksum
/// cannot be verified during const evaluation (it would take a Keccak-256
/// pass), so rather than accepting a checksummed-looking literal whose
/// checksum may be wrong, the macro fails closed and asks for the case-free
/// form. Lowercasing drops no information — the checksum is derived from
/// the hex digits alone.
///
/// The macro takes exactly one string literal; for the zero address use
/// [`Address::ZERO`] instead of an all-zero literal.
///
/// ```
/// use cow_sdk_core::{Address, address};
///
/// const VAULT_RELAYER: Address = address!("0xc92e8bdf79f0507f65a392b0ab4667716bfe0110");
/// assert_eq!(
///     VAULT_RELAYER.to_hex_string(),
///     "0xc92e8bdf79f0507f65a392b0ab4667716bfe0110",
/// );
/// ```
#[macro_export]
macro_rules! address {
    ($hex:literal) => {{
        const _: () = $crate::__private::assert_lowercase_address_literal($hex);
        $crate::Address::from_alloy($crate::__private::alloy_primitives::address!($hex))
    }};
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
    /// value; `as_*` returns a borrow.
    #[inline]
    #[must_use]
    pub fn to_hex_string(&self) -> String {
        format!("{:#x}", self.0)
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

hex_newtype! {
    /// Validated 32-byte app-data hash.
    ///
    /// The wire form is the protocol-canonical `0x`-prefixed 66-character
    /// lowercase hexadecimal string. The newtype is `#[repr(transparent)]` over
    /// [`alloy_primitives::B256`] and forwards `Display`/`Serialize`/
    /// `Deserialize` to the inner alloy type, whose lowercase 0x-prefixed
    /// default already matches the cow wire form.
    #[doc(alias = "app-data")]
    #[doc(alias = "AppData")]
    AppDataHash(B256, 32, field = "app_data_hash", hex_chars = APP_DATA_HASH_HEX_CHARS)
}

impl AppDataHash {
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
}

// --- Hash32 -----------------------------------------------------------------

hex_newtype! {
    /// Generic validated 32-byte hash wrapper for user-domain and contract surfaces.
    ///
    /// The wire form is the protocol-canonical `0x`-prefixed 66-character
    /// lowercase hexadecimal string. The newtype is `#[repr(transparent)]` over
    /// [`alloy_primitives::B256`] and forwards `Display`/`Serialize`/
    /// `Deserialize` to the inner alloy type, whose canonical defaults already
    /// emit the cow lowercase wire form.
    Hash32(B256, 32, field = "hash32", hex_chars = HASH32_HEX_CHARS)
}

impl Hash32 {
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
}

/// Transaction hash alias.
pub type TransactionHash = Hash32;
/// Block hash alias.
pub type BlockHash = Hash32;
/// Order digest alias.
pub type OrderDigest = Hash32;

// --- OrderUid ---------------------------------------------------------------

hex_newtype! {
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
    OrderUid(FixedBytes<56>, 56, field = "order_uid", hex_chars = ORDER_UID_HEX_CHARS)
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
