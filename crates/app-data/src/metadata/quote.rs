//! Typed `metadata.quote` slippage hint carried inside the app-data envelope.
//!
//! The reviewed quote metadata records the slippage tolerance a quote was
//! built against, expressed in basis points. [`QuoteMetadata`] narrows that
//! wire object to a typed Rust value whose derived serde impls reproduce the
//! camelCase wire form, and whose [`QuoteMetadata::validate`] enforces the
//! published `[0, 10000]` basis-point bound at the client before a document is
//! sealed — the one quote-side bound the SDK is responsible for emitting
//! correctly.

use cow_sdk_core::ValidationReason;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::AppDataError;

/// Inclusive upper bound for a slippage tolerance expressed in basis points
/// (`10000` basis points is `100%`).
const MAX_SLIPPAGE_BIPS: u32 = 10_000;

/// Typed `metadata.quote` value carrying the slippage tolerance a quote was
/// built against.
///
/// The struct head is `#[non_exhaustive]` so future additions to the reviewed
/// quote schema can be introduced as a minor change without breaking
/// downstream exhaustive matches. Unknown wire fields are ignored on
/// deserialization so a document produced by a newer schema minor still
/// parses through the typed shape.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteMetadata {
    /// Slippage tolerance in basis points, bounded to the inclusive range
    /// `[0, 10000]` by [`QuoteMetadata::validate`].
    pub slippage_bips: u32,
    /// Optional flag marking the tolerance as smart-slippage derived.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub smart_slippage: Option<bool>,
    /// Optional quote metadata schema version carried by some wire documents.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
}

impl QuoteMetadata {
    /// Creates quote metadata for a slippage tolerance in basis points after
    /// validating it against the published bound.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidAppDataProvided`] when `slippage_bips`
    /// exceeds `10000`.
    pub fn new(slippage_bips: u32) -> Result<Self, AppDataError> {
        let quote = Self {
            slippage_bips,
            smart_slippage: None,
            version: None,
        };
        quote.validate()?;
        Ok(quote)
    }

    /// Returns a copy flagged as smart-slippage derived.
    #[must_use]
    pub const fn with_smart_slippage(mut self, smart_slippage: bool) -> Self {
        self.smart_slippage = Some(smart_slippage);
        self
    }

    /// Validates the slippage tolerance against the published `[0, 10000]`
    /// basis-point bound.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidAppDataProvided`] when `slippage_bips`
    /// exceeds `10000`.
    pub const fn validate(&self) -> Result<(), AppDataError> {
        if self.slippage_bips > MAX_SLIPPAGE_BIPS {
            return Err(AppDataError::InvalidAppDataProvided {
                field: "metadata.quote.slippageBips",
                reason: ValidationReason::OutOfRange {
                    details: "slippageBips must be an integer in the inclusive range [0, 10000]",
                },
            });
        }
        Ok(())
    }

    /// Parses quote metadata from an app-data metadata value.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::Json`] when the value does not match the typed
    /// quote shape.
    pub fn from_value(value: Value) -> Result<Self, AppDataError> {
        serde_json::from_value(value).map_err(AppDataError::from)
    }
}
