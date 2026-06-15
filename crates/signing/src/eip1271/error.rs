use cow_sdk_core::Redacted;
use thiserror::Error;

/// Error returned by custom EIP-1271 signature providers.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum Eip1271SignatureError {
    /// Provider-specific failure while producing an EIP-1271 payload.
    #[error("EIP-1271 signature provider failed during {operation}: {message}")]
    Provider {
        /// Operation that requested the provider signature.
        operation: &'static str,
        /// Redacted provider diagnostic.
        message: Redacted<String>,
    },
}

impl Eip1271SignatureError {
    /// Creates a provider failure with a redacted message.
    #[must_use]
    pub fn provider(operation: &'static str, message: impl Into<String>) -> Self {
        Self::Provider {
            operation,
            message: message.into().into(),
        }
    }
}
