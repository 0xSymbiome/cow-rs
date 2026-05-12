use std::{
    fmt,
    ops::{Add, AddAssign, Sub, SubAssign},
};

use num_bigint::{BigInt, BigUint};
use serde::{Deserialize, Serialize};

use super::hex::U256_MAX_BITS;
use crate::errors::{CoreError, ValidationError};
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
