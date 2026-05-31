use thiserror::Error;

use crate::{cancellation::Cancelled, config::CowEnv, redaction::Redacted};

/// Validation failures for typed user input and configuration values.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// A required string or collection field was empty after validation.
    #[error("{field} must not be empty")]
    EmptyField {
        /// Identifies the invalid field.
        field: &'static str,
    },
    /// A value could not be serialized into a valid HTTP header.
    #[error("{field} must be a valid HTTP header value")]
    InvalidHttpHeaderValue {
        /// Identifies the invalid field.
        field: &'static str,
    },
    /// A hexadecimal value did not include the required `0x` prefix.
    #[error("{field} must be 0x-prefixed hexadecimal data")]
    InvalidHexPrefix {
        /// Identifies the invalid field.
        field: &'static str,
    },
    /// A fixed-length hexadecimal value had the wrong number of hex characters.
    #[error("{field} must contain exactly {expected} hex characters")]
    InvalidHexLength {
        /// Identifies the invalid field.
        field: &'static str,
        /// Required number of hex characters excluding the `0x` prefix.
        expected: usize,
    },
    /// A hexadecimal value contained non-hex characters.
    #[error("{field} contains non-hex characters")]
    InvalidHexCharacters {
        /// Identifies the invalid field.
        field: &'static str,
    },
    /// A decimal or hexadecimal numeric value could not be parsed.
    #[error("{field} must be a non-negative integer quantity")]
    InvalidNumeric {
        /// Identifies the invalid field.
        field: &'static str,
    },
    /// A parsed numeric value exceeded the supported `uint256` range.
    #[error("{field} exceeds uint256 bounds")]
    NumericOverflow {
        /// Identifies the invalid field.
        field: &'static str,
    },
    /// A chain id was not part of the supported `CoW` Protocol network set.
    #[error("unsupported chain id {chain_id}")]
    UnsupportedChain {
        /// Unsupported numeric chain id supplied by the caller.
        chain_id: u64,
    },
    /// A `valid_to` duration fell outside the supported relative-window range.
    #[error(
        "valid_to duration {actual_seconds} is outside the supported relative range {min}..={max}"
    )]
    ValidToOutOfRange {
        /// Requested duration in seconds.
        actual_seconds: u64,
        /// Minimum supported duration in seconds.
        min: u32,
        /// Maximum supported duration in seconds.
        max: u32,
    },
    /// A `DecimalAmount` decimals scale was above the maximum representable value.
    ///
    /// The maximum is `77` because `10^77 < 2^256 - 1 < 10^78`, so any
    /// `decimals` value above `77` would make `10^decimals` overflow the
    /// inner `uint256` storage used by `DecimalAmount::to_decimal_string`.
    /// Every ERC-20 token across the supported chains ships
    /// `decimals <= 18`, so the bound is structurally satisfied in
    /// practice; the explicit error replaces a previous runtime panic
    /// path with construction-time fail-closed validation.
    #[error("DecimalAmount decimals scale {actual} exceeds the maximum representable value {max}")]
    DecimalsOutOfRange {
        /// The decimals scale that was rejected.
        actual: u8,
        /// The maximum representable decimals scale.
        max: u8,
    },
}

/// Top-level core crate error.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CoreError {
    /// Validation failed for a typed user input or configuration value.
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),
    /// The selected chain/environment pair did not resolve to a base URL.
    #[error(
        "missing API base URL for chain id {chain_id} in {env} environment (partner_api={partner_api})"
    )]
    MissingBaseUrl {
        /// Numeric chain id that could not be resolved.
        chain_id: u64,
        /// Environment name used during resolution.
        env: CowEnv,
        /// Whether partner API URLs were being requested.
        partner_api: bool,
    },
    /// A JSON or ABI-adjacent serialization step failed.
    #[error("serialization error: {0}")]
    Serialization(Redacted<String>),
    /// A downstream transport implementation violated the core contract.
    #[error("transport contract violation: {0}")]
    TransportContract(Redacted<String>),
    /// A long-running operation was cancelled through a cooperative cancellation token.
    #[error("operation was cancelled")]
    Cancelled,
    /// A CID input failed to parse or did not match the canonical app-data shape.
    #[error("invalid CID")]
    InvalidCid,
}

impl From<Cancelled> for CoreError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}

/// Coarse-grained failure classification shared across the workspace error
/// family.
///
/// Every public error type that the `cow-sdk` facade aggregates exposes a
/// `class(&self) -> ErrorClass` accessor that resolves to one of these
/// buckets, so downstream telemetry and retry layers can partition failures
/// without pattern-matching every nested variant by hand. Retry policies
/// typically retry only [`ErrorClass::Transport`] and [`ErrorClass::Remote`];
/// the other classes signal caller-side or protocol-level conditions that
/// benefit from different recovery paths.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ErrorClass {
    /// Caller-side input failed a client-side validation boundary.
    Validation,
    /// A transport-layer failure occurred before a complete response was received.
    Transport,
    /// The remote endpoint returned a structured error response.
    Remote,
    /// The remote endpoint signalled rate limiting (HTTP 429) and the
    /// transport layer's retry budget was exhausted before it cleared.
    ///
    /// Transport retries already honor `Retry-After`, so reaching this class
    /// means the throttle outlived the retry policy rather than a transient
    /// spike the client absorbed.
    RateLimited,
    /// A signing, provider, or cryptographic helper surfaced an error.
    Signing,
    /// A long-running operation was cancelled through a cooperative token.
    Cancelled,
    /// An internal invariant or helper contract was violated.
    Internal,
}

impl CoreError {
    /// Returns the coarse-grained [`ErrorClass`] for this error.
    #[must_use]
    pub const fn class(&self) -> ErrorClass {
        match self {
            Self::Validation(_) | Self::MissingBaseUrl { .. } => ErrorClass::Validation,
            Self::Cancelled => ErrorClass::Cancelled,
            // Serialization, transport-contract, and CID failures plus any
            // future additive variants signal invariant violations.
            _ => ErrorClass::Internal,
        }
    }
}
