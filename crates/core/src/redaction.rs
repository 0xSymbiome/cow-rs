//! Typed redaction wrapper for secret-bearing configuration fields.
//!
//! `Redacted<T>` keeps the inner value available for deliberate use while
//! preventing accidental leakage through [`std::fmt::Debug`],
//! [`std::fmt::Display`], and [`serde::Serialize`]. Every public representation
//! emits the literal string `"[redacted]"` regardless of the wrapped payload.
//! [`Redacted::into_inner`] and [`Redacted::as_inner`] surface the underlying
//! value when the caller has explicit intent.

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// The placeholder emitted in every redacted representation.
pub const REDACTED_PLACEHOLDER: &str = "[redacted]";

/// Newtype wrapper that redacts the inner value in every non-explicit representation.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Redacted<T>(T);

impl<T> Redacted<T> {
    /// Wraps a value in the redacted newtype.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    /// Consumes the wrapper and returns the inner value.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Returns a borrow of the inner value for deliberate access.
    #[inline]
    pub const fn as_inner(&self) -> &T {
        &self.0
    }

    /// Returns a mutable borrow of the inner value for deliberate mutation.
    #[inline]
    pub const fn as_inner_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED_PLACEHOLDER)
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED_PLACEHOLDER)
    }
}

impl<T> Serialize for Redacted<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(REDACTED_PLACEHOLDER)
    }
}

impl<'de, T: Deserialize<'de>> Deserialize<'de> for Redacted<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        T::deserialize(deserializer).map(Self)
    }
}

impl<T> From<T> for Redacted<T> {
    #[inline]
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debug_and_display_emit_the_redacted_placeholder() {
        let secret = Redacted::new("super-secret-value".to_owned());
        assert_eq!(format!("{secret:?}"), REDACTED_PLACEHOLDER);
        assert_eq!(format!("{secret}"), REDACTED_PLACEHOLDER);
    }

    #[test]
    fn serialize_emits_the_redacted_placeholder_regardless_of_inner_value() {
        let secret = Redacted::new("another-secret".to_owned());
        let json = serde_json::to_string(&secret).unwrap();
        assert_eq!(json, format!("\"{REDACTED_PLACEHOLDER}\""));
    }

    #[test]
    fn into_inner_escapes_the_redaction_for_deliberate_use() {
        let secret = Redacted::new("explicit-unwrap".to_owned());
        assert_eq!(secret.into_inner(), "explicit-unwrap");
    }
}
