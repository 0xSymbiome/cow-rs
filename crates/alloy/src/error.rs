//! Error types for the native composed Alloy adapter.

use std::fmt;

use cow_sdk_core::{Redacted, TransportErrorClass};

use crate::conversion::rpc_error_to_class_and_detail;

/// Error returned by [`crate::AlloyClient`] and [`crate::AlloyClientSignerHandle`].
///
/// `Validation` and `Internal` hold caller- or upstream-authored text behind
/// [`Redacted`] so neither the `thiserror`-derived `Display` nor the hand-written
/// `Debug` can leak credential-bearing detail.
#[non_exhaustive]
#[derive(thiserror::Error)]
pub enum AlloyClientError {
    /// Caller-controlled input failed validation.
    #[error("validation error: {0}")]
    Validation(Redacted<String>),
    /// The HTTP transport, JSON-RPC envelope, or response decoding failed.
    #[error("transport error ({class}): {detail}")]
    Transport {
        /// Stable transport class for retry and telemetry decisions.
        class: TransportErrorClass,
        /// Redacted transport detail.
        detail: Redacted<String>,
    },
    /// The remote JSON-RPC server returned an error payload.
    #[error("remote error (code {code}): {message}")]
    Remote {
        /// JSON-RPC error code.
        code: i64,
        /// JSON-RPC error message.
        message: String,
    },
    /// The upstream signer failed.
    #[error("signing error: {detail}")]
    Signing {
        /// Redacted signing detail.
        detail: Redacted<String>,
    },
    /// Pending transaction registration or watch failed.
    #[error("pending transaction error: {detail}")]
    PendingTransaction {
        /// Redacted pending-transaction detail.
        detail: Redacted<String>,
    },
    /// The operation was cancelled by [`cow_sdk_core::Cancellable`].
    #[error("operation cancelled")]
    Cancelled,
    /// A local invariant or internal conversion failed.
    #[error("internal error: {0}")]
    Internal(Redacted<String>),
}

impl AlloyClientError {
    /// Wraps caller-input detail in the redacted `Validation` arm.
    pub(crate) const fn validation(message: String) -> Self {
        Self::Validation(Redacted::new(message))
    }

    /// Wraps internal detail in the redacted `Internal` arm.
    pub(crate) const fn internal(message: String) -> Self {
        Self::Internal(Redacted::new(message))
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
                Self::internal(message)
            }
            _ => Self::internal("unknown alloy transport error classification".to_owned()),
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
            Self::Cancelled => f.write_str("Cancelled"),
            Self::Internal(_) => f
                .debug_tuple("Internal")
                .field(&Redacted::new("internal detail"))
                .finish(),
        }
    }
}

/// `AlloyClient` composes a local-key signer with the alloy HTTP
/// provider and never surfaces an EIP-1193 wallet rejection: the
/// signer holds the key locally, so the user-prompt flow that
/// produces EIP-1193 4001 simply does not exist on this adapter.
/// The trait returns `None` for every variant so the signing crate
/// routes every umbrella failure through the redacted
/// `SigningError::Signer` path. New rejection-class variants must
/// extend this impl alongside the new variant.
impl cow_sdk_core::UserRejection for AlloyClientError {
    fn user_rejection_code(&self) -> Option<i32> {
        match self {
            Self::Validation(_)
            | Self::Transport { .. }
            | Self::Remote { .. }
            | Self::Signing { .. }
            | Self::PendingTransaction { .. }
            | Self::Cancelled
            | Self::Internal(_) => None,
        }
    }
}

impl From<cow_sdk_core::CoreError> for AlloyClientError {
    fn from(error: cow_sdk_core::CoreError) -> Self {
        Self::validation(error.to_string())
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

impl From<cow_sdk_alloy_provider::ProviderError> for AlloyClientError {
    fn from(error: cow_sdk_alloy_provider::ProviderError) -> Self {
        match error {
            cow_sdk_alloy_provider::ProviderError::Validation(detail) => Self::Validation(detail),
            cow_sdk_alloy_provider::ProviderError::Transport { class, detail } => {
                Self::Transport { class, detail }
            }
            cow_sdk_alloy_provider::ProviderError::Remote { code, message } => {
                Self::Remote { code, message }
            }
            cow_sdk_alloy_provider::ProviderError::Cancelled => Self::Cancelled,
            cow_sdk_alloy_provider::ProviderError::Internal(detail) => Self::Internal(detail),
            _ => Self::internal("unknown alloy provider error".to_owned()),
        }
    }
}
