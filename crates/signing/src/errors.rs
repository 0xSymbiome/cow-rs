use cow_sdk_contracts::{ContractsError, SigningScheme};
use cow_sdk_core::{Cancelled, CoreError, Redacted};
use thiserror::Error;

/// Errors returned by explicit signing helpers.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SigningError {
    /// Core validation failed while building a signing payload.
    #[error("core error: {0}")]
    Core(#[from] CoreError),
    /// Contract-level hashing or encoding failed.
    #[error("contracts error: {0}")]
    Contracts(#[from] ContractsError),
    /// JSON or payload serialization failed.
    #[error("serialization error: {0}")]
    Serialization(Redacted<String>),
    /// A signer operation returned an error.
    #[error("signer {operation} failed: {message}")]
    Signer {
        /// Signer operation being attempted.
        operation: &'static str,
        /// Signer error message.
        message: Redacted<String>,
    },
    /// The signer reported a structured user rejection of the request,
    /// typically corresponding to EIP-1193 provider error code 4001.
    ///
    /// The fields are deterministic, non-sensitive classifications:
    /// `label` names the high-level operation the user declined
    /// (`"typed-data signature"`, `"message signature"`, etc.) and
    /// `code` carries the EIP-1193 numeric code so downstream
    /// consumers can render the standard provider error class without
    /// inspecting backend-specific strings.
    #[error("User rejected {label} ({code})")]
    SignerRejection {
        /// High-level operation label derived from the signing-helper
        /// call site (e.g. `"typed-data signature"`).
        label: &'static str,
        /// EIP-1193 provider error code reported by the wallet.
        code: i32,
    },
    /// Local signer generation only supports ECDSA-style schemes.
    #[error(
        "local signer-generated signatures only support EIP712 and ETHSIGN; received {scheme:?}"
    )]
    UnsupportedSignerGeneratedScheme {
        /// Unsupported requested signing scheme.
        scheme: SigningScheme,
    },
    /// A long-running signing operation was cancelled through a cooperative cancellation token.
    #[error("operation cancelled")]
    Cancelled,
}

impl From<Cancelled> for SigningError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}
