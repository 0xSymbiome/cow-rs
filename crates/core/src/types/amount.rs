use std::{
    fmt,
    ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign},
    str::FromStr,
};

use alloy_primitives::{I256, U256};
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
/// For decimal-aware values that also carry a scale, see
/// [`DecimalAmount`]. For signed quantities, see [`SignedAmount`].
///
/// # Surface boundary
///
/// The arithmetic surface is intentionally narrower than the inner
/// [`alloy_primitives::U256`]. `Amount` does **not** expose:
///
/// - `wrapping_add` / `wrapping_sub` / `wrapping_mul` / `wrapping_pow`:
///   silent overflow is incompatible with financial-amount safety.
///   Callers needing wrapping arithmetic drop into the inner
///   primitive through [`Amount::as_u256`] or [`Amount::into_u256`],
///   making the wrapping intent visible at the type boundary.
/// - `overflowing_add` / `overflowing_sub` / `overflowing_mul` /
///   `overflowing_pow`: same rationale; the `(value, overflow)`
///   tuple form belongs at the low-level primitive seam, not on the
///   typed financial surface.
/// - Bit-counting helpers (`count_ones`, `count_zeros`,
///   `leading_zeros`, `trailing_zeros`, `is_power_of_two`,
///   `next_power_of_two`): no demand from the `CoW` Protocol
///   surfaces this type was built for. The exposed
///   [`Amount::bit_len`] covers the "how big is this number"
///   question.
///
/// The shipped surface is: [`Amount::ZERO`], [`Amount::MAX`],
/// [`Amount::new`], `Add` / `Sub` / `Mul` and their `Assign`
/// variants (all `#[track_caller]`), [`Amount::checked_add`] /
/// [`Amount::checked_sub`] / [`Amount::checked_mul`] /
/// [`Amount::checked_pow`], [`Amount::saturating_add`] /
/// [`Amount::saturating_sub`] / [`Amount::saturating_mul`] /
/// [`Amount::saturating_pow`], [`Amount::pow`], and
/// [`Amount::bit_len`]. Combined with [`Amount::as_u256`] /
/// [`Amount::into_u256`] for the explicit alloy seam, this covers
/// every operation cow's own crates need to perform on a typed
/// amount.
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
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Amount(
    /// Escape hatch only: prefer [`Amount::as_u256`] (borrowed) or
    /// [`Amount::into_u256`] (owned) for forward compatibility per
    /// ADR 0052. The `.0` field is `pub` to match the canonical
    /// [`alloy_primitives`] pattern and to keep the
    /// `#[repr(transparent)]` bit-for-bit layout contract visible at
    /// the type system, but it is not part of the long-term API
    /// contract. A future cascade may seal this field through a
    /// documented deprecation cycle if a runtime validation invariant
    /// requires it; consumers who rely on `.0` accept the
    /// forward-compatibility risk.
    pub U256,
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

    /// Raises `self` to the power `exp`. Debug-panics in debug builds and
    /// always panics on overflow.
    ///
    /// The body routes through [`Amount::checked_pow`] (rather than
    /// delegating directly to the inner [`alloy_primitives::U256::pow`])
    /// because the inner method is `wrapping_pow` and would silently
    /// produce wrong values on overflow. Callers needing infallible
    /// behaviour use [`Amount::checked_pow`] or
    /// [`Amount::saturating_pow`] instead.
    ///
    /// # Panics
    ///
    /// Panics if `self` raised to `exp` would exceed [`Amount::MAX`].
    #[inline]
    #[track_caller]
    #[must_use]
    pub fn pow(&self, exp: &Self) -> Self {
        // SAFETY: the documented contract is that `Amount::pow`
        // panics on overflow; routing through `checked_pow` ensures
        // the overflow path becomes an explicit panic with caller
        // location rather than the silent wrap that
        // `ruint::Uint::pow` (= `wrapping_pow`) would produce on
        // direct delegation. Callers needing infallible behaviour
        // use `Amount::checked_pow` or `Amount::saturating_pow`.
        self.checked_pow(exp).expect("Amount::pow overflow")
    }

    /// Like [`Amount::pow`], but returns `None` on overflow.
    #[inline]
    #[must_use]
    pub fn checked_pow(&self, exp: &Self) -> Option<Self> {
        self.0.checked_pow(exp.0).map(Self)
    }

    /// Like [`Amount::pow`], but saturates to [`Amount::MAX`] on overflow.
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

// The `#[track_caller]` annotation on the six `Amount` arithmetic
// operator impls below preserves the caller's panic location across
// the cow newtype boundary. The inner `alloy_primitives::U256` (which
// is `ruint::Uint<256, 4>`) already carries `#[track_caller]` on its
// arithmetic impls, but the bodies delegate to `wrapping_*` and never
// panic in any build profile, so for `Amount` the annotation is a
// chain-link guard rather than a panic-redirect today. The same impls
// on `SignedAmount` further below ARE load-bearing because
// `alloy_primitives::I256` panics on overflow in debug builds via
// `debug_assert!(!overflow)`; without `#[track_caller]` on the cow
// wrapper the reported panic location would point at this file
// instead of at the caller's expression. Annotation order is
// `#[inline]` then `#[track_caller]`, matching the stdlib
// `core::ops::arith` and alloy `Signed` precedents.
impl Add<Self> for Amount {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<Self> for Amount {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn sub(self, rhs: Self) -> Self::Output {
        // Delegates to the inner `alloy_primitives::U256` `Sub` impl,
        // which wraps on underflow in both debug and release builds.
        // The cow contract for the typed operator surface matches the
        // upstream `U256` semantics so `a + b - c` operator chains compose
        // without an intermediate `Option` boundary; callers that need
        // total subtraction semantics use [`Amount::checked_sub`] or
        // [`Amount::saturating_sub`] explicitly.
        Self(self.0 - rhs.0)
    }
}

impl Mul<Self> for Amount {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl AddAssign<Self> for Amount {
    #[inline]
    #[track_caller]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign<Self> for Amount {
    #[inline]
    #[track_caller]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl MulAssign<Self> for Amount {
    #[inline]
    #[track_caller]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
    }
}

/// Decimal-aware token amount pairing an atomic quantity with a decimals scale.
///
/// `DecimalAmount` keeps the authoritative storage in atoms so settlement
/// arithmetic stays exact, while exposing the decimals scale for display
/// and human-oriented conversion paths. Wire formats continue to carry the
/// atomic value as a base-10 string; this type is intended for in-process
/// typing and ergonomic conversions rather than transport.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecimalAmount {
    atoms: U256,
    decimals: u8,
}

impl DecimalAmount {
    /// Maximum representable decimals scale.
    ///
    /// `77` is the largest exponent for which `10^decimals` fits in the
    /// inner `uint256` storage that [`DecimalAmount::to_decimal_string`]
    /// uses to derive the integer/fractional split: `10^77 < 2^256 - 1`
    /// while `10^78` overflows. Every ERC-20 token across the
    /// supported chains ships `decimals <= 18`, so the bound is
    /// structurally satisfied in practice; the constant is the
    /// canonical accessor for boundary-aware callers and the public
    /// contract that `DecimalAmount::new`,
    /// [`DecimalAmount::from_atoms`], and
    /// [`DecimalAmount::from_whole_approx`] all enforce at
    /// construction.
    pub const MAX_DECIMALS: u8 = 77;

    /// Creates a decimal-aware amount from an atomic quantity and decimals scale.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] (via
    /// [`ValidationError::DecimalsOutOfRange`]) when `decimals` exceeds
    /// [`DecimalAmount::MAX_DECIMALS`].
    #[inline]
    pub const fn new(atoms: U256, decimals: u8) -> Result<Self, CoreError> {
        if decimals > Self::MAX_DECIMALS {
            return Err(CoreError::Validation(ValidationError::DecimalsOutOfRange {
                actual: decimals,
                max: Self::MAX_DECIMALS,
            }));
        }
        Ok(Self { atoms, decimals })
    }

    /// Creates a decimal-aware amount from a raw [`alloy_primitives::U256`]
    /// atomic quantity and a decimals scale.
    ///
    /// Equivalent to [`DecimalAmount::new`]; preserved as the named
    /// constructor for callsites that prefer the explicit
    /// "atoms + decimals" shape.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when `decimals` exceeds
    /// [`DecimalAmount::MAX_DECIMALS`].
    #[inline]
    pub const fn from_atoms(atoms: U256, decimals: u8) -> Result<Self, CoreError> {
        Self::new(atoms, decimals)
    }

    /// Creates an approximate decimal-aware amount from a whole-unit
    /// floating-point value.
    ///
    /// Non-finite or negative whole-unit inputs clamp to zero atoms (with
    /// the requested decimals scale preserved). The conversion is lossy
    /// beyond `f64` precision and is intended for display or user-input
    /// flows rather than settlement arithmetic.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when `decimals` exceeds
    /// [`DecimalAmount::MAX_DECIMALS`]. The whole-unit value itself is
    /// always accepted (non-finite and negative inputs clamp to zero
    /// atoms rather than failing).
    pub fn from_whole_approx(whole_units: f64, decimals: u8) -> Result<Self, CoreError> {
        if decimals > Self::MAX_DECIMALS {
            return Err(CoreError::Validation(ValidationError::DecimalsOutOfRange {
                actual: decimals,
                max: Self::MAX_DECIMALS,
            }));
        }
        if !whole_units.is_finite() || whole_units < 0.0 {
            return Ok(Self {
                atoms: U256::ZERO,
                decimals,
            });
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
        Ok(Self {
            atoms: U256::from(atoms_u128),
            decimals,
        })
    }

    /// Returns a borrow of the raw atomic quantity.
    #[inline]
    #[must_use]
    pub const fn atoms(&self) -> &U256 {
        &self.atoms
    }

    /// Consumes the decimal amount and returns the raw atomic quantity.
    #[inline]
    #[must_use]
    pub const fn into_atoms(self) -> U256 {
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
        let atoms_str = self.atoms.to_string();
        let atoms_f: f64 = atoms_str.parse().unwrap_or(f64::NAN);
        let scale = 10f64.powi(i32::from(self.decimals));
        atoms_f / scale
    }

    /// Returns the canonical decimal-string form of this amount with the
    /// decimal point inserted at the configured `decimals` position.
    ///
    /// The `decimals > 0` arm renders the canonical decimal-point form
    /// through [`alloy_primitives::utils::format_units`], which splits
    /// `self.atoms` against `10^self.decimals` and pads the fractional
    /// substring to length `self.decimals`. The output is exact up to
    /// the full `uint256` storage range.
    ///
    /// **Trailing zeroes in the fractional portion are preserved**: the
    /// fractional substring length always equals `self.decimals`, so the
    /// emitted string can be parsed back into the original
    /// `(atoms, decimals)` pair by any external lossless decimal parser
    /// without ambiguity. When `self.decimals == 0` the integer form is
    /// returned unchanged (no decimal point).
    ///
    /// This format intentionally differs from the JavaScript ecosystem's
    /// `formatUnits` helper (ethers, viem, and the cow-protocol services
    /// utility chain), which trims trailing zeros: `formatUnits(1e18, 18)`
    /// returns `"1.0"`, while [`DecimalAmount::to_decimal_string`] on the
    /// same value returns `"1.000000000000000000"`. The cow format
    /// preserves the full fractional precision so the
    /// `(integer, fractional)` digit pair is byte-identical to the
    /// underlying `(atoms, decimals)`.
    ///
    /// # Examples
    ///
    /// ```
    /// use alloy_primitives::U256;
    /// use cow_sdk_core::DecimalAmount;
    ///
    /// // Integer form (zero decimals) — no decimal point.
    /// let answer = DecimalAmount::from_atoms(U256::from(42u8), 0).unwrap();
    /// assert_eq!(answer.to_decimal_string(), "42");
    ///
    /// // Canonical 1 ether — trailing zeros preserved (ethers/viem
    /// // `formatUnits(1e18, 18)` would return `"1.0"`).
    /// let one_ether = DecimalAmount::from_atoms(
    ///     U256::from(1_000_000_000_000_000_000u128),
    ///     18,
    /// )
    /// .unwrap();
    /// assert_eq!(one_ether.to_decimal_string(), "1.000000000000000000");
    ///
    /// // Small atoms, large decimals — fractional length always equals decimals.
    /// let smallest = DecimalAmount::from_atoms(U256::from(1u8), 18).unwrap();
    /// assert_eq!(smallest.to_decimal_string(), "0.000000000000000001");
    /// ```
    ///
    /// # Panics
    ///
    /// Cannot panic in practice. [`DecimalAmount::new`],
    /// [`DecimalAmount::from_atoms`], and
    /// [`DecimalAmount::from_whole_approx`] all reject `decimals` values
    /// above [`DecimalAmount::MAX_DECIMALS`] at construction time, and
    /// [`DecimalAmount::MAX_DECIMALS`] equals
    /// `alloy_primitives::utils::Unit::MAX`, so every
    /// `(self.atoms, self.decimals)` pair this method observes is
    /// inside the range
    /// [`alloy_primitives::utils::format_units`] accepts. The `expect`
    /// call is belt-and-braces: it preserves an explicit panic
    /// location should a future constructor surface bypass the
    /// boundary without re-validating.
    #[must_use]
    #[track_caller]
    pub fn to_decimal_string(&self) -> String {
        if self.decimals == 0 {
            return self.atoms.to_string();
        }
        // SAFETY: `DecimalAmount::new`, `from_atoms`, and
        // `from_whole_approx` all reject `decimals > MAX_DECIMALS == 77`
        // at construction time, and `MAX_DECIMALS == 77` matches the
        // `alloy_primitives::utils::Unit::MAX` ceiling, so the
        // `format_units` call is structurally infallible. The `expect`
        // retains an explicit panic location should a future
        // constructor surface bypass the boundary without re-validating.
        // The trailing-zero preservation contract (fractional substring
        // length always equals `self.decimals`) is pinned by
        // `crates/core/tests/types_contract.rs` and the four invariants
        // in `crates/core/tests/property_contract.rs`.
        alloy_primitives::utils::format_units(self.atoms, self.decimals)
            .expect("decimals <= MAX_DECIMALS == Unit::MAX == 77; format_units cannot fail")
    }
}

/// Canonical signed integer for protocol surfaces that carry signed values
/// (such as the trade-simulation reader's token deltas).
///
/// The newtype is `#[repr(transparent)]` over [`alloy_primitives::I256`]
/// and ships cow-owned trait surfaces so the wire form stays the canonical
/// decimal string with an optional leading minus sign.
///
/// # Surface boundary
///
/// The arithmetic surface mirrors [`Amount`]'s narrow shape and is
/// intentionally smaller than the inner
/// [`alloy_primitives::Signed`]. `SignedAmount` does **not** expose:
///
/// - `wrapping_add` / `wrapping_sub` / `wrapping_mul` / `wrapping_pow`
///   / `wrapping_neg` / `wrapping_abs`: silent overflow is
///   incompatible with financial-amount safety. Callers needing
///   wrapping arithmetic drop into the inner primitive through
///   [`SignedAmount::as_i256`] or [`SignedAmount::into_i256`].
/// - `overflowing_*` variants: same rationale.
/// - Bit-counting helpers beyond [`SignedAmount::bits`]:
///   `count_ones`, `count_zeros`, `leading_zeros`,
///   `trailing_zeros`, etc. have no `CoW` Protocol use case on
///   signed token deltas.
///
/// The shipped surface is: [`SignedAmount::ZERO`],
/// [`SignedAmount::MAX`], [`SignedAmount::MIN`],
/// [`SignedAmount::new`], `Add` / `Sub` / `Mul` and their `Assign`
/// variants (all `#[track_caller]`),
/// [`SignedAmount::checked_add`] / [`SignedAmount::checked_sub`] /
/// [`SignedAmount::checked_mul`] / [`SignedAmount::checked_pow`] /
/// [`SignedAmount::checked_neg`] / [`SignedAmount::checked_abs`] /
/// [`SignedAmount::checked_unsigned_abs`],
/// [`SignedAmount::saturating_add`] /
/// [`SignedAmount::saturating_sub`] /
/// [`SignedAmount::saturating_mul`] /
/// [`SignedAmount::saturating_pow`], [`SignedAmount::pow`], and
/// [`SignedAmount::bits`]. Combined with
/// [`SignedAmount::as_i256`] / [`SignedAmount::into_i256`] for the
/// explicit alloy seam, this covers every signed-amount operation
/// cow's own crates need.
#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SignedAmount(
    /// Escape hatch only: prefer [`SignedAmount::as_i256`] (borrowed) or
    /// [`SignedAmount::into_i256`] (owned) for forward compatibility per
    /// ADR 0052. The `.0` field is `pub` to match the canonical
    /// [`alloy_primitives`] pattern and to keep the
    /// `#[repr(transparent)]` bit-for-bit layout contract visible at the
    /// type system, but it is not part of the long-term API contract. A
    /// future cascade may seal this field through a documented
    /// deprecation cycle if a runtime validation invariant requires it;
    /// consumers who rely on `.0` accept the forward-compatibility risk.
    pub I256,
);

impl SignedAmount {
    /// Canonical zero quantity.
    pub const ZERO: Self = Self(I256::ZERO);

    /// Maximum representable signed quantity (`2^255 - 1`).
    pub const MAX: Self = Self(I256::MAX);

    /// Minimum representable signed quantity (`-2^255`).
    pub const MIN: Self = Self(I256::MIN);

    /// Parses a canonical signed `int256` quantity from a strict decimal
    /// string.
    ///
    /// The accepted grammar is `-?[0-9]+` (an optional single leading
    /// minus sign, no leading plus sign, no whitespace, no radix prefix).
    /// Inputs with the `0x`, `0X`, `0o`, `0O`, `0b`, or `0B` prefix that
    /// the alloy [`I256`] `FromStr` impl would otherwise silently accept
    /// (the sign is stripped before forwarding to the underlying
    /// `Uint::FromStr` four-radix sniffer) are rejected so the cow
    /// JSON-decimal-only wire contract holds.
    ///
    /// # Errors
    ///
    /// Returns [`CoreError`] when the input is empty, has a forbidden
    /// radix prefix, contains a leading plus sign, contains non-decimal
    /// characters, or exceeds `int256` bounds.
    pub fn new(value: impl AsRef<str>) -> Result<Self, CoreError> {
        let value = value.as_ref();
        validate_strict_decimal_signed("signed_amount", value)?;
        let parsed = I256::from_dec_str(value).map_err(|_| ValidationError::NumericOverflow {
            field: "signed_amount",
        })?;
        Ok(Self(parsed))
    }

    /// Creates a signed amount from a raw [`alloy_primitives::I256`] value.
    #[inline]
    #[must_use]
    pub const fn from_i256(value: I256) -> Self {
        Self(value)
    }

    /// Returns a borrow of the underlying [`alloy_primitives::I256`].
    #[inline]
    #[must_use]
    pub const fn as_i256(&self) -> &I256 {
        &self.0
    }

    /// Consumes the signed amount and returns the underlying
    /// [`alloy_primitives::I256`].
    #[inline]
    #[must_use]
    pub const fn into_i256(self) -> I256 {
        self.0
    }

    /// Returns the canonical signed-decimal string form of this amount
    /// as an owned [`String`]. The output carries an optional leading
    /// minus sign for negative values and matches the byte sequence the
    /// cow newtype emits through its [`fmt::Display`] and
    /// [`Serialize`] impls.
    #[inline]
    #[must_use]
    pub fn to_decimal_string(&self) -> String {
        self.0.to_string()
    }

    /// Returns `true` when this amount equals the zero quantity.
    #[inline]
    #[must_use]
    pub fn is_zero(&self) -> bool {
        self.0 == I256::ZERO
    }

    /// Returns `true` when this amount is strictly less than zero.
    #[inline]
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    /// Returns the checked sum of two signed amounts; `None` on `int256` overflow.
    #[inline]
    #[must_use]
    pub fn checked_add(&self, other: &Self) -> Option<Self> {
        self.0.checked_add(other.0).map(Self)
    }

    /// Returns the checked difference of two signed amounts; `None` on `int256` overflow.
    #[inline]
    #[must_use]
    pub fn checked_sub(&self, other: &Self) -> Option<Self> {
        self.0.checked_sub(other.0).map(Self)
    }

    /// Returns the checked product of two signed amounts; `None` on `int256` overflow.
    #[inline]
    #[must_use]
    pub fn checked_mul(&self, other: &Self) -> Option<Self> {
        self.0.checked_mul(other.0).map(Self)
    }

    /// Returns the saturating sum of two signed amounts.
    #[inline]
    #[must_use]
    pub const fn saturating_add(&self, other: &Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// Returns the saturating difference of two signed amounts.
    #[inline]
    #[must_use]
    pub const fn saturating_sub(&self, other: &Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// Returns the saturating product of two signed amounts.
    #[inline]
    #[must_use]
    pub fn saturating_mul(&self, other: &Self) -> Self {
        Self(self.0.saturating_mul(other.0))
    }

    /// Returns the additive inverse of this signed amount; `None` when
    /// negating `I256::MIN`.
    #[inline]
    #[must_use]
    pub fn checked_neg(&self) -> Option<Self> {
        self.0.checked_neg().map(Self)
    }

    /// Returns the absolute value as a [`SignedAmount`]; `None` when
    /// negating [`alloy_primitives::I256::MIN`] would overflow the signed
    /// range. Mirrors the alloy [`alloy_primitives::Signed::checked_abs`]
    /// shape — same input type, same output type.
    #[inline]
    #[must_use]
    pub fn checked_abs(&self) -> Option<Self> {
        self.0.checked_abs().map(Self)
    }

    /// Returns the absolute value as an unsigned [`Amount`]; `None` when
    /// negating [`alloy_primitives::I256::MIN`] would overflow. Mirrors
    /// the alloy [`alloy_primitives::Signed::unsigned_abs`] shape:
    /// signed input, unsigned output, fallibility expressed through
    /// the `Option` boundary because [`alloy_primitives::I256::MIN`] has
    /// no representable absolute value on the signed surface but its
    /// bit-for-bit unsigned form would silently bridge into [`Amount`].
    #[inline]
    #[must_use]
    pub fn checked_unsigned_abs(&self) -> Option<Amount> {
        let absolute = self.0.checked_abs()?;
        Some(Amount(absolute.into_raw()))
    }

    /// Raises `self` to the unsigned power `exp`. Panics on overflow in
    /// debug builds (alloy `Signed::pow` is `debug_assert!(!overflow)`).
    ///
    /// Delegates to [`alloy_primitives::Signed::pow`] which already
    /// carries `#[track_caller]`, so the reported panic location is the
    /// caller's expression rather than this file. Callers needing
    /// infallible behaviour use [`SignedAmount::checked_pow`] or
    /// [`SignedAmount::saturating_pow`] instead.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if the result would overflow the signed
    /// `int256` range; in release builds the operation wraps silently
    /// the same way alloy's underlying `Signed::pow` does.
    #[inline]
    #[must_use]
    pub fn pow(&self, exp: &Amount) -> Self {
        Self(self.0.pow(exp.0))
    }

    /// Like [`SignedAmount::pow`], but returns `None` on overflow.
    #[inline]
    #[must_use]
    pub fn checked_pow(&self, exp: &Amount) -> Option<Self> {
        self.0.checked_pow(exp.0).map(Self)
    }

    /// Like [`SignedAmount::pow`], but saturates at the signed numeric
    /// bounds (positive or negative) on overflow.
    #[inline]
    #[must_use]
    pub fn saturating_pow(&self, exp: &Amount) -> Self {
        Self(self.0.saturating_pow(exp.0))
    }

    /// Returns the minimum number of bits needed to represent `self` as a
    /// signed two's-complement integer.
    ///
    /// Returns 0 for [`SignedAmount::ZERO`]. Positive values that are
    /// not negative powers of two include an extra sign bit (`+1` →
    /// `2`, `+2` → `3`, `+3` → `3`). Negative powers of two and `-1`
    /// reuse the high bit as the sign bit (`-1` → `1`, `-2` → `2`,
    /// `-4` → `3`). The return type matches alloy
    /// [`alloy_primitives::Signed::bits`] (`u32`); see the alloy doc
    /// comment for the worked example and the full edge-case table.
    #[inline]
    #[must_use]
    pub fn bits(&self) -> u32 {
        self.0.bits()
    }
}

impl From<I256> for SignedAmount {
    #[inline]
    fn from(value: I256) -> Self {
        Self(value)
    }
}

impl From<SignedAmount> for I256 {
    #[inline]
    fn from(value: SignedAmount) -> Self {
        value.0
    }
}

impl TryFrom<&str> for SignedAmount {
    type Error = CoreError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for SignedAmount {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value.as_str())
    }
}

impl FromStr for SignedAmount {
    type Err = CoreError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl fmt::Display for SignedAmount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // alloy I256 Display is decimal natively with an optional leading minus.
        fmt::Display::fmt(&self.0, f)
    }
}

impl Serialize for SignedAmount {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for SignedAmount {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = <std::borrow::Cow<'_, str>>::deserialize(deserializer)?;
        Self::new(value.as_ref()).map_err(serde::de::Error::custom)
    }
}

// `SignedAmount`'s operator impls carry `#[track_caller]` for the
// same chain-link reason as the `Amount` block above, with one
// difference: the inner `alloy_primitives::I256` arithmetic panics on
// overflow in debug builds via `handle_overflow` (which is
// `debug_assert!(!overflow)`), so the annotation is load-bearing
// today. Without it, a debug-mode overflow panic would surface with
// its `info.location()` pointing at this file rather than at the
// caller's expression.
impl Add<Self> for SignedAmount {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl Sub<Self> for SignedAmount {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl Mul<Self> for SignedAmount {
    type Output = Self;

    #[inline]
    #[track_caller]
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl AddAssign<Self> for SignedAmount {
    #[inline]
    #[track_caller]
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl SubAssign<Self> for SignedAmount {
    #[inline]
    #[track_caller]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl MulAssign<Self> for SignedAmount {
    #[inline]
    #[track_caller]
    fn mul_assign(&mut self, rhs: Self) {
        self.0 *= rhs.0;
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

fn validate_strict_decimal_signed(field: &'static str, value: &str) -> Result<(), CoreError> {
    if value.is_empty() {
        return Err(ValidationError::EmptyField { field }.into());
    }
    // The cow signed wire form accepts a single leading minus and never a
    // leading plus. Strip the minus once before forwarding to the unsigned
    // strict-decimal validator below so the radix-prefix and digit-only
    // checks fire over the absolute portion of the input.
    let body = if let Some(stripped) = value.strip_prefix('-') {
        if stripped.is_empty() {
            return Err(ValidationError::InvalidNumeric { field }.into());
        }
        stripped
    } else if value.starts_with('+') {
        return Err(ValidationError::InvalidNumeric { field }.into());
    } else {
        value
    };
    if has_radix_prefix(body) {
        return Err(ValidationError::InvalidNumeric { field }.into());
    }
    if !body.bytes().all(|byte| byte.is_ascii_digit()) {
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
