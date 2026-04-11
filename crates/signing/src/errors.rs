use cow_sdk_contracts::{ContractsError, SigningScheme};
use cow_sdk_core::CoreError;
use thiserror::Error;

/// Errors returned by explicit signing helpers.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SigningError {
    /// Core validation failed while building a signing payload.
    #[error("core error: {0}")]
    Core(#[from] CoreError),
    /// Contract-level hashing or encoding failed.
    #[error("contracts error: {0}")]
    Contracts(#[from] ContractsError),
    /// JSON or payload serialization failed.
    #[error("serialization error: {0}")]
    Serialization(String),
    /// A signer operation returned an error.
    #[error("signer {operation} failed: {message}")]
    Signer {
        /// Signer operation being attempted.
        operation: &'static str,
        /// Signer error message.
        message: String,
    },
    /// Local signer generation only supports ECDSA-style schemes.
    #[error(
        "local signer-generated signatures only support EIP712 and ETHSIGN; received {scheme:?}"
    )]
    UnsupportedSignerGeneratedScheme {
        /// Unsupported requested signing scheme.
        scheme: SigningScheme,
    },
}
