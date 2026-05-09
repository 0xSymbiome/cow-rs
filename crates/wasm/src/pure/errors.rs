use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Host-safe error type shared by pure helpers before wasm export mapping.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum PureError {
    /// A string, hex, numeric, or JSON input failed validation.
    #[error("invalid input for {field}: {message}")]
    InvalidInput {
        /// Public field name that failed validation.
        field: String,
        /// Human-readable validation failure.
        message: String,
    },
    /// A string enum value was not accepted by the SDK surface.
    #[error("unknown enum value '{value}' for {field}")]
    UnknownEnumValue {
        /// Public field name that carried the value.
        field: String,
        /// Rejected value.
        value: String,
    },
    /// A numeric chain id is not configured by the SDK.
    #[error("unsupported chain id: {chain_id}")]
    UnsupportedChain {
        /// Numeric chain id supplied by the caller.
        chain_id: u32,
    },
}

impl PureError {
    pub(crate) fn invalid(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            field: field.into(),
            message: message.into(),
        }
    }

    pub(crate) fn unknown_enum(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::UnknownEnumValue {
            field: field.into(),
            value: value.into(),
        }
    }
}
