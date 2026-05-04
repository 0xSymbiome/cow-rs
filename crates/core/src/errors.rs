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
}

impl From<Cancelled> for CoreError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}
