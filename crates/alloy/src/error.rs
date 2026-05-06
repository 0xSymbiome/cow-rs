//! Error types for the native composed Alloy adapter.

use std::{error::Error, fmt};

use cow_sdk_core::{Redacted, TransportErrorClass};

use crate::conversion::rpc_error_to_class_and_detail;

/// Coarse classification for [`AlloyClientError`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AlloyClientErrorClass {
    /// Caller-controlled input failed validation.
    Validation,
    /// Transport or response decoding failed before a remote JSON-RPC error.
    Transport,
    /// The JSON-RPC peer returned a structured remote error.
    Remote,
    /// The upstream signing backend failed.
    Signing,
    /// Pending transaction registration or watch failed.
    PendingTransaction,
    /// The requested transaction-signing shape is intentionally unsupported.
    UnsupportedTransactionRequest,
    /// The future was cancelled by a consumer-provided cancellation token.
    Cancelled,
    /// A local invariant or unsupported upstream path was reached.
    Internal,
}

impl AlloyClientErrorClass {
    /// Returns the stable lowercase class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::Transport => "transport",
            Self::Remote => "remote",
            Self::Signing => "signing",
            Self::PendingTransaction => "pending_transaction",
            Self::UnsupportedTransactionRequest => "unsupported_transaction_request",
            Self::Cancelled => "cancelled",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for AlloyClientErrorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error returned by [`crate::AlloyClient`] and [`crate::AlloyClientSignerHandle`].
#[non_exhaustive]
pub enum AlloyClientError {
    /// Caller-controlled input failed validation.
    Validation(String),
    /// The HTTP transport, JSON-RPC envelope, or response decoding failed.
    Transport {
        /// Stable transport class for retry and telemetry decisions.
        class: TransportErrorClass,
        /// Redacted transport detail.
        detail: Redacted<String>,
    },
    /// The remote JSON-RPC server returned an error payload.
    Remote {
        /// JSON-RPC error code.
        code: i64,
        /// JSON-RPC error message.
        message: String,
    },
    /// The upstream signer failed.
    Signing {
        /// Redacted signing detail.
        detail: Redacted<String>,
    },
    /// Pending transaction registration or watch failed.
    PendingTransaction {
        /// Redacted pending-transaction detail.
        detail: Redacted<String>,
    },
    /// The requested transaction-signing shape is intentionally unsupported.
    UnsupportedTransactionRequest {
        /// Method that is unsupported.
        method: &'static str,
        /// Static reason describing the supported alternative.
        reason: &'static str,
    },
    /// The operation was cancelled by [`cow_sdk_core::Cancellable`].
    Cancelled,
    /// A local invariant or internal conversion failed.
    Internal(String),
}

impl AlloyClientError {
    /// Returns this error's coarse class.
    #[must_use]
    pub const fn class(&self) -> AlloyClientErrorClass {
        match self {
            Self::Validation(_) => AlloyClientErrorClass::Validation,
            Self::Transport { .. } => AlloyClientErrorClass::Transport,
            Self::Remote { .. } => AlloyClientErrorClass::Remote,
            Self::Signing { .. } => AlloyClientErrorClass::Signing,
            Self::PendingTransaction { .. } => AlloyClientErrorClass::PendingTransaction,
            Self::UnsupportedTransactionRequest { .. } => {
                AlloyClientErrorClass::UnsupportedTransactionRequest
            }
            Self::Cancelled => AlloyClientErrorClass::Cancelled,
            Self::Internal(_) => AlloyClientErrorClass::Internal,
        }
    }

    /// Inter-crate seam constructor; not part of the semver-stable consumer
    /// API. Sibling adapter crates use this to lift Alloy transport errors into
    /// the umbrella's typed error surface. The argument shape may change in any
    /// minor release.
    #[doc(hidden)]
    #[must_use]
    pub fn from_alloy_transport(error: alloy_transport::TransportError) -> Self {
        match rpc_error_to_class_and_detail(error) {
            cow_sdk_alloy_provider::__seam::RpcErrorClassification::Transport { class, detail } => {
                Self::Transport { class, detail }
            }
            cow_sdk_alloy_provider::__seam::RpcErrorClassification::Remote { code, message } => {
                Self::Remote { code, message }
            }
            cow_sdk_alloy_provider::__seam::RpcErrorClassification::Internal(message) => {
                Self::Internal(message)
            }
            _ => Self::Internal("unknown alloy transport error classification".to_owned()),
        }
    }

    /// Inter-crate seam constructor; not part of the semver-stable consumer
    /// API. Sibling adapter crates use this to lift Alloy signer errors into
    /// the umbrella's typed error surface. The argument shape may change in any
    /// minor release.
    #[doc(hidden)]
    #[must_use]
    pub fn from_alloy_signer(error: &alloy_signer::Error) -> Self {
        Self::Signing {
            detail: Redacted::new(error.to_string()),
        }
    }

    /// Inter-crate seam constructor; not part of the semver-stable consumer
    /// API. Sibling adapter crates use this to lift Alloy pending transaction
    /// errors into the umbrella's typed error surface. The argument shape may
    /// change in any minor release.
    #[doc(hidden)]
    #[must_use]
    pub fn from_pending_tx_error(error: &alloy_provider::PendingTransactionError) -> Self {
        Self::PendingTransaction {
            detail: Redacted::new(error.to_string()),
        }
    }
}

impl fmt::Debug for AlloyClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(_) => f
                .debug_tuple("Validation")
                .field(&Redacted::new("validation detail"))
                .finish(),
            Self::Transport { class, detail } => f
                .debug_struct("Transport")
                .field("class", class)
                .field("detail", detail)
                .finish(),
            Self::Remote { code, message } => f
                .debug_struct("Remote")
                .field("code", code)
                .field("message", message)
                .finish(),
            Self::Signing { detail } => f.debug_struct("Signing").field("detail", detail).finish(),
            Self::PendingTransaction { detail } => f
                .debug_struct("PendingTransaction")
                .field("detail", detail)
                .finish(),
            Self::UnsupportedTransactionRequest { method, reason } => f
                .debug_struct("UnsupportedTransactionRequest")
                .field("method", method)
                .field("reason", reason)
                .finish(),
            Self::Cancelled => f.write_str("Cancelled"),
            Self::Internal(_) => f
                .debug_tuple("Internal")
                .field(&Redacted::new("internal detail"))
                .finish(),
        }
    }
}

impl fmt::Display for AlloyClientError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(_) => f.write_str("validation error: [redacted]"),
            Self::Transport { class, detail } => {
                write!(f, "transport error ({class}): {detail}")
            }
            Self::Remote { code, message } => {
                write!(f, "remote error (code {code}): {message}")
            }
            Self::Signing { detail } => write!(f, "signing error: {detail}"),
            Self::PendingTransaction { detail } => {
                write!(f, "pending transaction error: {detail}")
            }
            Self::UnsupportedTransactionRequest { method, reason } => {
                write!(f, "the {method} method is unsupported: {reason}")
            }
            Self::Cancelled => f.write_str("operation cancelled"),
            Self::Internal(_) => f.write_str("internal error: [redacted]"),
        }
    }
}

impl Error for AlloyClientError {}

impl From<cow_sdk_core::CoreError> for AlloyClientError {
    fn from(error: cow_sdk_core::CoreError) -> Self {
        Self::Validation(error.to_string())
    }
}

impl From<cow_sdk_core::Cancelled> for AlloyClientError {
    fn from(_: cow_sdk_core::Cancelled) -> Self {
        Self::Cancelled
    }
}

impl From<cow_sdk_contracts::ContractsError> for AlloyClientError {
    fn from(error: cow_sdk_contracts::ContractsError) -> Self {
        Self::Signing {
            detail: Redacted::new(error.to_string()),
        }
    }
}

impl From<cow_sdk_alloy_provider::AsyncProviderError> for AlloyClientError {
    fn from(error: cow_sdk_alloy_provider::AsyncProviderError) -> Self {
        match error {
            cow_sdk_alloy_provider::AsyncProviderError::Validation(detail) => {
                Self::Validation(detail)
            }
            cow_sdk_alloy_provider::AsyncProviderError::Transport { class, detail } => {
                Self::Transport { class, detail }
            }
            cow_sdk_alloy_provider::AsyncProviderError::Remote { code, message } => {
                Self::Remote { code, message }
            }
            cow_sdk_alloy_provider::AsyncProviderError::Cancelled => Self::Cancelled,
            cow_sdk_alloy_provider::AsyncProviderError::Internal(detail) => Self::Internal(detail),
            _ => Self::Internal("unknown alloy provider error".to_owned()),
        }
    }
}
