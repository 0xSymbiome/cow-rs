use std::{fmt, str::FromStr};

use alloy_primitives::U256;
use serde::{Deserialize, Serialize};

use crate::errors::{CoreError, ValidationError};

/// Canonical non-negative `uint256` quantity.
///
/// `Amount` is the typed boundary for atomic token values on every
/// `CoW` Protocol surface: contract hashing, EIP-712 typed data,
/// orderbook DTOs, and decimal-aware display. The newtype is
/// `#[repr(transparent)]` over [`alloy_primitives::U256`], so the
/// in-memory layout is bit-for-bit identical to the alloy primitive and
/// conversion at the alloy seam is free at runtime through
/// [`Amount::as_u256`] (borrowed), [`Amount::into_u256`] (owned), or
/// [`From`] / [`Into`].
///
/// `Amount` carries cow-owned [`fmt::Display`], [`Serialize`], and
/// [`Deserialize`] impls so the wire form stays the canonical decimal
/// string the orderbook and contract layer accept. The cow-owned
/// `Deserialize` is strict-decimal fail-closed: it rejects `0x`, `0X`,
/// `0o`, `0O`, `0b`, `0B` prefixes (the four alternative radices the
/// alloy [`U256`] `FromStr` impl would otherwise accept silently) so the
/// cow JSON-decimal-only wire contract holds even when the value is fed
/// through serde rather than [`Amount::new`].
///
/// # Construction
///
/// Pick the constructor that matches the value you already hold; every
/// path lands on the same atomic `uint256`:
///
/// - Raw atomic units from an integer — [`Amount::from`] (`u32` / `u64` /
///   `u128` / `usize`) or [`Amount::from_u256`].
/// - Whole display units from a number — [`Amount::from_units`], for
///   example `Amount::from_units(1000, 6)` for 1000 USDC (no string and no
///   hand-counted zeros).
/// - Fractional or untrusted-text display units — [`Amount::parse_units`],
///   for example `Amount::parse_units("1.5", 18)` for 1.5 WETH.
/// - A decimal or `0x`-hex string of atomic units from a CLI flag,
///   environment variable, or config file — [`Amount::new`].
///
/// [`Amount::format_units`] is the inverse of the unit-scaled constructors
/// for human-readable display.
///
/// # Surface boundary
///
/// The arithmetic surface is intentionally narrower than the inner
/// [`alloy_primitives::U256`]. `Amount` does **not** expose:
///
/// - `Add` / `Sub` / `Mul` (and the `*Assign` operators): the bare
///   `+` `-` `*` operators on the inner `U256` wrap silently on
///   overflow and underflow, which is incompatible with
///   financial-amount safety — `a - b` for `a < b` would silently
///   become a value near `2^256`. Typed arithmetic is therefore
///   fallible by return: use [`Amount::checked_add`] /
///   [`Amount::checked_sub`] / [`Amount::checked_mul`] (`-> Option`),
///   or the explicit [`Amount::saturating_add`] /
///   [`Amount::saturating_sub`] / [`Amount::saturating_mul`] clamps.
///   A caller who genuinely wants wrapping reaches through
///   [`Amount::as_u256`] / [`Amount::into_u256`], making the wrapping
///   intent visible at the type boundary.
/// - `wrapping_*` / `overflowing_*`: same rationale; the wrapping and
///   `(value, overflow)` tuple forms belong at the low-level
///   primitive seam, not on the typed financial surface.
/// - Bit-counting helpers (`count_ones`, `count_zeros`,
///   `leading_zeros`, `trailing_zeros`, `is_power_of_two`,
///   `next_power_of_two`): no demand from the `CoW` Protocol
///   surfaces this type was built for. The exposed
///   [`Amount::bit_len`] covers the "how big is this number"
///   question.
///
/// The shipped surface is: [`Amount::ZERO`], [`Amount::MAX`],
/// [`Amount::new`], [`Amount::checked_add`] / [`Amount::checked_sub`]
/// / [`Amount::checked_mul`] / [`Amount::checked_pow`],
/// [`Amount::saturating_add`] / [`Amount::saturating_sub`] /
/// [`Amount::saturating_mul`] / [`Amount::saturating_pow`], and
/// [`Amount::bit_len`]. Combined with [`Amount::as_u256`] /
/// [`Amount::into_u256`] for the explicit alloy seam, this covers
/// every operation cow's own crates need to perform on a typed
/// amount.
///
/// There is no `From<String>` or `From<&str>` conversion: construct through
/// [`Amount::new`] or [`Amount::parse_units`] so malformed input fails closed
/// at the typed boundary rather than via an infallible `.into()`.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Amount(
    // Private inner: the constructors (`new` / `parse_units` / `from_units` /
    // `from_u256` / `From`) and the `as_u256` / `into_u256` accessors are the
    // entire contract, so a future runtime invariant can land without breaking
    // consumers (ADR 0052).
    U256,
);

impl Amount {
    /// Canonical zero quantity.
    pub const ZERO: Self = Self(U256::ZERO);

    /// Maximum representable quantity (`2^256 - 1`).
    pub const MAX: Self = Self(U256::MAX);

    /// Parses a canonical non-negative `uint256` quantity from a developer
    /// ergonomic string.
    ///
    /// The accepted grammar is decimal (`[0-9]+`) **or** `0x`-prefixed
    /// hexadecimal (`0x[0-9a-fA-F]+`); both uppercase (`0X`) and
    /// lowercase prefixes are honoured. Octal (`0o`) and binary (`0b`)
    /// prefixes are rejected so the cow constructor does not silently
    /// widen beyond the historical working-tree contract observed at
    /// `crates/core/src/types/amount.rs::parse_u256_quantity` lines
    /// 510-528 and asserted by the committed contract test at
    /// `crates/orderbook/tests/types_contract.rs:349`. Leading zeroes
    /// are accepted and canonicalised on the [`fmt::Display`] /
    /// [`Serialize`] return path. Negative values are rejected because
    /// [`Amount`] is unsigned.
    ///
    /// The wire-form [`Deserialize`] impl is intentionally **MORE
    /// strict** than this constructor: it is strict-decimal-only
    /// fail-closed and rejects every radix prefix (including `0x`).
    /// The asymmetry is deliberate per the cow constructor
    /// affordance: the JSON wire grammar stays decimal-only while the
    /// Rust-API constructor remains friendly to non-JSON callers
    /// (CLI, env var, programmatic).
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, has a leading
    /// minus, has an `0o` / `0b` radix prefix, contains characters
    /// outside the recognised decimal or hex digit set, or exceeds
    /// `uint256` bounds.
    // DO NOT SWAP for alloy_primitives::U256::from_str (or I256::from_str).
    //
    // alloy's `Uint::FromStr` sniffs four radix prefixes (`0x`, `0X`, `0o`,
    // `0O`, `0b`, `0B`) per ruint-1.18.0/src/string.rs:225-240. The cow
    // constructor explicitly rejects the octal and binary prefixes so a
    // config typo like "0o755" surfaces as `InvalidNumeric` instead of
    // silently parsing as 493 wei. The constructor uses
    // `U256::from_str_radix(_, 10)` and `U256::from_str_radix(hex, 16)` to
    // pick the radix explicitly; do not collapse onto `U256::from_str`.
    //
    // ADR: docs/adr/0052-alloy-primitives-canonical-primitive-layer.md
    // (lines 99-104).
    // Doctrine: docs/alloy-doctrine.md, Bucket 2 row for `Amount::new`
    // lenient constructor.
    // Enforced by cargo check-source-fences (xtask/src/policy/fences.rs).
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        if value.is_empty() {
            return Err(ValidationError::EmptyField { field: "amount" }.into());
        }
        if value.starts_with('-') || value.starts_with('+') {
            return Err(ValidationError::InvalidNumeric { field: "amount" }.into());
        }
        // Reject the alloy `ruint::Uint::FromStr` octal / binary radix
        // prefixes so the lenient constructor surface stays narrower
        // than alloy's default sniffer.
        let bytes = value.as_bytes();
        if bytes.len() >= 2 && bytes[0] == b'0' {
            match bytes[1] {
                b'o' | b'O' | b'b' | b'B' => {
                    return Err(ValidationError::InvalidNumeric { field: "amount" }.into());
                }
                _ => {}
            }
        }
        let parsed = if let Some(hex) = value
            .strip_prefix("0x")
            .or_else(|| value.strip_prefix("0X"))
        {
            if hex.is_empty() {
                return Err(ValidationError::InvalidNumeric { field: "amount" }.into());
            }
            U256::from_str_radix(hex, 16)
        } else {
            U256::from_str_radix(value, 10)
        }
        .map_err(|_| ValidationError::InvalidNumeric { field: "amount" })?;
        Ok(Self(parsed))
    }

    /// Creates an amount from a raw [`alloy_primitives::U256`] value.
    #[inline]
    #[must_use]
    pub const fn from_u256(atoms: U256) -> Self {
        Self(atoms)
    }

    /// Returns a borrow of the underlying [`alloy_primitives::U256`].
    #[inline]
    #[must_use]
    pub const fn as_u256(&self) -> &U256 {
        &self.0
    }

    /// Consumes the amount and returns the underlying [`alloy_primitives::U256`].
    #[inline]
    #[must_use]
    pub const fn into_u256(self) -> U256 {
        self.0
    }

    /// Returns the canonical base-10 decimal string form of this amount
    /// as an owned [`String`].
    ///
    /// Follows the Rust stdlib naming convention: `to_*` returns an
    /// owned value. The returned string matches the byte sequence the
    /// cow newtype emits through its [`fmt::Display`] and
    /// [`Serialize`] impls, so callers that need the wire form without
    /// routing through `serde_json` can use this accessor directly.
    /// The `decimal` qualifier in the method name distinguishes it
    /// from the byte-typed `to_hex_string` accessor on
    /// [`AppDataHash`](crate::AppDataHash) and the other identity
    /// newtypes.
    #[inline]
    #[must_use]
    pub fn to_decimal_string(&self) -> String {
        self.0.to_string()
    }

    /// Returns `true` when this amount equals the zero quantity.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == U256::ZERO
    }

    /// Returns the checked sum of two amounts; `None` on `uint256` overflow.
    #[inline]
    #[must_use]
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Returns the checked difference of two amounts; `None` on underflow.
    #[inline]
    #[must_use]
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    /// Returns the checked product of two amounts; `None` on `uint256` overflow.
    #[inline]
    #[must_use]
    pub fn checked_mul(&self, other: &Self) -> Option<Self> {
        self.0.checked_mul(other.0).map(Self)
    }

    /// Returns the saturating sum of two amounts (clamps at `U256::MAX`).
    #[inline]
    #[must_use]
    pub const fn saturating_add(&self, other: &Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Returns the saturating difference of two amounts (clamps at zero).
    #[inline]
    #[must_use]
    pub const fn saturating_sub(&self, other: &Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Returns the saturating product of two amounts (clamps at `U256::MAX`).
    #[inline]
    #[must_use]
    pub fn saturating_mul(&self, other: &Self) -> Self {
        Self(self.0.saturating_mul(other.0))
    }

    /// Raises `self` to the power `exp`, returning `None` on overflow.
    #[inline]
    #[must_use]
    pub fn checked_pow(&self, exp: &Self) -> Option<Self> {
        self.0.checked_pow(exp.0).map(Self)
    }

    /// Raises `self` to the power `exp`, saturating to [`Amount::MAX`] on overflow.
    #[inline]
    #[must_use]
    pub fn saturating_pow(&self, exp: &Self) -> Self {
        Self(self.0.saturating_pow(exp.0))
    }

    /// Returns the number of significant bits needed to represent `self`.
    ///
    /// Equivalent to `ceil(log2(self + 1))` for non-zero values; returns 0
    /// for [`Amount::ZERO`]. Always ≤ 256 for the 256-bit storage, so the
    /// widening from the inner `usize` to `u64` is lossless on every
    /// supported target.
    #[inline]
    #[must_use]
    pub const fn bit_len(&self) -> u64 {
        self.0.bit_len() as u64
    }

    /// Parses an exact token amount from a human-readable decimal string and
    /// the token's `decimals` scale (for example `Amount::parse_units("1.5", 18)`
    /// builds 1.5 WETH in atomic units).
    ///
    /// This is the typed, exact analogue of the `parseUnits` helper from
    /// viem/ethers: the decimal string is scaled by `10^decimals` using
    /// integer arithmetic (never floating point), so the result is exact.
    /// Fractional digits beyond `decimals` are truncated, matching the
    /// orderbook's atomic-unit contract. For a whole-number amount you
    /// already hold as an integer, [`Amount::from_units`] avoids the string
    /// entirely; the companion [`Amount::format_units`] is the inverse.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when `value` is empty or whitespace, carries a
    /// leading sign (`+` / `-`, since [`Amount`] is unsigned), is not a valid
    /// decimal, or when `decimals` exceeds `77`
    /// ([`alloy_primitives::utils::Unit::MAX`]).
    // DO NOT SWAP for a bare `alloy_primitives::utils::parse_units` call.
    //
    // alloy's `parse_units` is fail-OPEN on several inputs (the relevant
    // body is `crates/primitives/src/utils/units.rs`):
    //   - `parse_units("", d)` returns `Ok(0)`, so an empty field silently
    //     becomes a zero amount instead of an error.
    //   - a leading `-` routes to the signed `I256` arm, and the subsequent
    //     `Into<U256>` returns the two's-complement bit pattern (a huge
    //     positive) with no error.
    //   - the fractional-truncation step slices the input by *byte* offset
    //     (`&amount[..(amount.len() - (dec_len - exponent))]`, units.rs:258),
    //     so a non-ASCII input whose boundary lands inside a multi-byte
    //     UTF-8 char PANICS with "byte index N is not a char boundary".
    //   - the final scaling multiply (`a_uint *= 10.pow(exp - dec_len)`,
    //     units.rs:285) is a *wrapping* `*=` (only the `pow` is checked), so
    //     a value whose true magnitude exceeds `uint256` silently WRAPS to a
    //     wrong number with no error.
    // To honour the documented fail-closed `# Errors` contract (and never
    // panic or silently wrap on untrusted input), this constructor does the
    // exact integer scaling itself with checked arithmetic rather than
    // delegating to the alloy helper: it pre-rejects empty/whitespace and a
    // leading `+` / `-` (the same guards as `Amount::new`), restricts the
    // grammar to ASCII decimal digits and a single `.` separator, truncates
    // the fractional digits beyond `decimals`, and rejects an over-`uint256`
    // result through `checked_mul`. Do not collapse onto the raw alloy call.
    pub fn parse_units(value: impl AsRef<str>, decimals: u8) -> Result<Self, CoreError> {
        let value = value.as_ref().trim();
        if value.is_empty() {
            return Err(ValidationError::EmptyField { field: "amount" }.into());
        }
        if value.starts_with('-') || value.starts_with('+') {
            return Err(ValidationError::InvalidNumeric { field: "amount" }.into());
        }
        if alloy_primitives::utils::Unit::new(decimals).is_none() {
            return Err(ValidationError::DecimalsOutOfRange {
                actual: decimals,
                max: alloy_primitives::utils::Unit::MAX.get(),
            }
            .into());
        }
        // Split into the integer and fractional halves on the single decimal
        // separator. More than one `.` is malformed.
        let mut halves = value.split('.');
        let integer_part = halves.next().unwrap_or("");
        let fractional_part = halves.next().unwrap_or("");
        if halves.next().is_some() {
            return Err(ValidationError::InvalidNumeric { field: "amount" }.into());
        }
        // Restrict the grammar to ASCII decimal digits in each half. This
        // rejects every non-decimal byte (sign, radix prefix, exponent, unit
        // suffix, non-ASCII) before any arithmetic runs.
        let is_ascii_digits = |s: &str| s.bytes().all(|b| b.is_ascii_digit());
        if !is_ascii_digits(integer_part) || !is_ascii_digits(fractional_part) {
            return Err(ValidationError::InvalidNumeric { field: "amount" }.into());
        }
        // Truncate fractional digits beyond `decimals` (the orderbook
        // atomic-unit contract). `decimals <= 77` here, so the slice index
        // is in bounds and ASCII-aligned.
        let decimals_usize = usize::from(decimals);
        let used_fractional = if fractional_part.len() > decimals_usize {
            &fractional_part[..decimals_usize]
        } else {
            fractional_part
        };
        // Build the mantissa (integer digits followed by the kept fractional
        // digits) and parse it as a `uint256`. An empty mantissa (for
        // example the bare separator ".") is rejected as non-numeric.
        let mantissa: String = format!("{integer_part}{used_fractional}");
        if mantissa.is_empty() {
            return Err(ValidationError::InvalidNumeric { field: "amount" }.into());
        }
        let mantissa_value = U256::from_str_radix(&mantissa, 10)
            .map_err(|_| ValidationError::InvalidNumeric { field: "amount" })?;
        // Left-shift the mantissa back to atomic units by the number of
        // fractional positions that were NOT supplied, using a checked
        // multiply so an over-`uint256` magnitude fails closed instead of
        // silently wrapping.
        let scale_exponent = decimals_usize - used_fractional.len();
        let scale = U256::from(10u8)
            .checked_pow(U256::from(scale_exponent))
            .ok_or(ValidationError::NumericOverflow { field: "amount" })?;
        let scaled = mantissa_value
            .checked_mul(scale)
            .ok_or(ValidationError::NumericOverflow { field: "amount" })?;
        Ok(Self(scaled))
    }

    /// Builds an exact token amount from a whole number of display units and
    /// the token's `decimals` scale (for example `Amount::from_units(1000, 6)`
    /// builds 1000 USDC in atomic units).
    ///
    /// This is the numeric, no-string companion to [`Amount::parse_units`]: the
    /// whole-unit count is scaled by `10^decimals` with checked integer
    /// arithmetic (never a string round-trip, never floating point), so the
    /// result is exact. Reach for this when the amount is a whole number you
    /// already hold as an integer; use [`Amount::parse_units`] when the amount
    /// is fractional or arrives as untrusted text, and [`Amount::from`] /
    /// [`Amount::from_u256`] when you already have raw atomic units.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when `decimals` exceeds `77`
    /// ([`alloy_primitives::utils::Unit::MAX`]) or when `whole * 10^decimals`
    /// would exceed [`alloy_primitives::U256::MAX`].
    pub fn from_units(whole: u128, decimals: u8) -> Result<Self, CoreError> {
        if alloy_primitives::utils::Unit::new(decimals).is_none() {
            return Err(ValidationError::DecimalsOutOfRange {
                actual: decimals,
                max: alloy_primitives::utils::Unit::MAX.get(),
            }
            .into());
        }
        // Scale the whole-unit count up to atomic units by `10^decimals` using
        // checked arithmetic so an over-`uint256` magnitude fails closed
        // instead of silently wrapping. `decimals <= 77` here, so the `pow`
        // itself cannot overflow, but the final multiply by a large `whole`
        // still can.
        let scale = U256::from(10u8)
            .checked_pow(U256::from(decimals))
            .ok_or(ValidationError::NumericOverflow { field: "amount" })?;
        let scaled = U256::from(whole)
            .checked_mul(scale)
            .ok_or(ValidationError::NumericOverflow { field: "amount" })?;
        Ok(Self(scaled))
    }

    /// Formats this atomic amount as a human-readable decimal string scaled by
    /// `decimals` (for example `format_units(18)` on
    /// `1_000_000_000_000_000_000` atoms returns `"1.000000000000000000"`).
    ///
    /// The inverse of [`Amount::parse_units`] for a given `decimals`: trailing
    /// zeroes in the fractional part are preserved (the fractional substring
    /// length always equals `decimals`), so the output round-trips back through
    /// `parse_units`. This intentionally differs from viem/ethers `formatUnits`,
    /// which trims to `"1.0"`. A `decimals` of `0` returns the bare integer;
    /// values above `77` are clamped to `77`
    /// ([`alloy_primitives::utils::Unit::MAX`]).
    #[must_use]
    pub fn format_units(&self, decimals: u8) -> String {
        if decimals == 0 {
            return self.0.to_string();
        }
        let unit = alloy_primitives::utils::Unit::new(decimals)
            .unwrap_or(alloy_primitives::utils::Unit::MAX);
        alloy_primitives::utils::ParseUnits::U256(self.0).format_units(unit)
    }
}

impl From<U256> for Amount {
    #[inline]
    fn from(value: U256) -> Self {
        Self(value)
    }
}

impl From<Amount> for U256 {
    #[inline]
    fn from(value: Amount) -> Self {
        value.0
    }
}

impl From<u32> for Amount {
    #[inline]
    fn from(value: u32) -> Self {
        Self(U256::from(value))
    }
}

impl From<u64> for Amount {
    #[inline]
    fn from(value: u64) -> Self {
        Self(U256::from(value))
    }
}

impl From<u128> for Amount {
    #[inline]
    fn from(value: u128) -> Self {
        Self(U256::from(value))
    }
}

impl From<usize> for Amount {
    #[inline]
    fn from(value: usize) -> Self {
        Self(U256::from(value))
    }
}

impl TryFrom<&str> for Amount {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for Amount {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl FromStr for Amount {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl fmt::Display for Amount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // alloy U256 Display is decimal natively per ruint's fmt impl.
        fmt::Display::fmt(&self.0, f)
    }
}

impl Serialize for Amount {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Amount {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <std::borrow::Cow<'_, str>>::deserialize(deserializer)?;
        // The wire form is strict-decimal-only even though `Amount::new`
        // is liberal: this validator closes the alloy `ruint` four-radix
        // sniffer gap so JSON payloads carrying `0x`/`0o`/`0b` prefixes
        // fail closed at the serde boundary.
        validate_strict_decimal_unsigned("amount", value.as_ref())
            .map_err(serde::de::Error::custom)?;
        Self::new(value.as_ref()).map_err(serde::de::Error::custom)
    }
}

fn validate_strict_decimal_unsigned(field: &'static str, value: &str) -> Result<(), CoreError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyField { field }.into());
    }
    if has_radix_prefix(value) {
        return Err(ValidationError::InvalidNumeric { field }.into());
    }
    if !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(ValidationError::InvalidNumeric { field }.into());
    }
    Ok(())
}

/// Returns `true` when the input begins with one of the alternate-radix
/// prefixes the alloy `ruint` `FromStr` impl would otherwise accept
/// silently (`0x`, `0X`, `0o`, `0O`, `0b`, `0B`).
fn has_radix_prefix(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() >= 2
        && bytes[0] == b'0'
        && matches!(bytes[1], b'x' | b'X' | b'o' | b'O' | b'b' | b'B')
}
