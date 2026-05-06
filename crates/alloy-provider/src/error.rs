//! Error types for the native Alloy provider adapter.

use std::{error::Error, fmt};

use cow_sdk_core::{Redacted, TransportErrorClass};

/// Coarse classification for [`AsyncProviderError`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AsyncProviderErrorClass {
    /// The caller supplied an invalid request.
    Validation,
    /// Transport or response decoding failed before a remote JSON-RPC error.
    Transport,
    /// The JSON-RPC peer returned a structured remote error.
    Remote,
    /// The future was cancelled by a consumer-provided cancellation token.
    Cancelled,
    /// A local invariant or unsupported upstream path was reached.
    Internal,
}

impl AsyncProviderErrorClass {
    /// Returns the stable lowercase class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::Transport => "transport",
            Self::Remote => "remote",
            Self::Cancelled => "cancelled",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for AsyncProviderErrorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error returned by [`crate::RpcAlloyProvider`].
#[non_exhaustive]
pub enum AsyncProviderError {
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
    /// The operation was cancelled by [`cow_sdk_core::Cancellable`].
    Cancelled,
    /// A local invariant or unsupported upstream path was reached.
    Internal(String),
}

impl AsyncProviderError {
    /// Returns this error's coarse class.
    #[must_use]
    pub const fn class(&self) -> AsyncProviderErrorClass {
        match self {
            Self::Validation(_) => AsyncProviderErrorClass::Validation,
            Self::Transport { .. } => AsyncProviderErrorClass::Transport,
            Self::Remote { .. } => AsyncProviderErrorClass::Remote,
            Self::Cancelled => AsyncProviderErrorClass::Cancelled,
            Self::Internal(_) => AsyncProviderErrorClass::Internal,
        }
    }

    /// Inter-crate seam constructor; not part of the semver-stable consumer
    /// API. Sibling adapter crates use this to lift Alloy transport errors into
    /// the provider's typed error surface. The argument shape may change in any
    /// minor release.
    #[doc(hidden)]
    #[must_use]
    pub fn from_alloy_transport(error: alloy_transport::TransportError) -> Self {
        match __transport_classification::rpc_error_to_class_and_detail(error) {
            __transport_classification::RpcErrorClassification::Transport { class, detail } => {
                Self::Transport { class, detail }
            }
            __transport_classification::RpcErrorClassification::Remote { code, message } => {
                Self::Remote { code, message }
            }
            __transport_classification::RpcErrorClassification::Internal(message) => {
                Self::Internal(message)
            }
        }
    }
}

impl fmt::Debug for AsyncProviderError {
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
            Self::Cancelled => f.write_str("Cancelled"),
            Self::Internal(_) => f
                .debug_tuple("Internal")
                .field(&Redacted::new("internal detail"))
                .finish(),
        }
    }
}

impl fmt::Display for AsyncProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(_) => f.write_str("validation error: [redacted]"),
            Self::Transport { class, detail } => {
                write!(f, "transport error ({class}): {detail}")
            }
            Self::Remote { code, message } => {
                write!(f, "remote error (code {code}): {message}")
            }
            Self::Cancelled => f.write_str("operation cancelled"),
            Self::Internal(_) => f.write_str("internal error: [redacted]"),
        }
    }
}

impl Error for AsyncProviderError {}

impl From<cow_sdk_core::CoreError> for AsyncProviderError {
    fn from(error: cow_sdk_core::CoreError) -> Self {
        Self::Validation(error.to_string())
    }
}

impl From<cow_sdk_core::Cancelled> for AsyncProviderError {
    fn from(_: cow_sdk_core::Cancelled) -> Self {
        Self::Cancelled
    }
}

/// Transport-classification helpers shared with sibling adapter crates.
pub(crate) mod __transport_classification {
    use alloy_json_rpc::RpcError;
    use cow_sdk_core::{Redacted, TransportErrorClass};

    /// Classified Alloy JSON-RPC or transport error detail.
    #[non_exhaustive]
    pub(crate) enum RpcErrorClassification {
        /// Transport-layer classification with redacted detail.
        Transport {
            /// Shared transport class.
            class: TransportErrorClass,
            /// Redacted detail.
            detail: Redacted<String>,
        },
        /// Remote JSON-RPC payload.
        Remote {
            /// JSON-RPC error code.
            code: i64,
            /// JSON-RPC error message.
            message: String,
        },
        /// Local invariant or unsupported upstream path.
        Internal(String),
    }

    /// Classifies every Alloy JSON-RPC error variant explicitly.
    #[must_use]
    pub(crate) fn rpc_error_to_class_and_detail(
        error: alloy_transport::TransportError,
    ) -> RpcErrorClassification {
        match error {
            RpcError::Transport(kind) => transport_kind_to_class_and_detail(kind),
            RpcError::ErrorResp(payload) => RpcErrorClassification::Remote {
                code: payload.code,
                message: payload.message.into_owned(),
            },
            RpcError::NullResp => RpcErrorClassification::Transport {
                class: TransportErrorClass::Decode,
                detail: Redacted::new("remote returned null where a value was expected".to_owned()),
            },
            RpcError::UnsupportedFeature(feature) => {
                RpcErrorClassification::Internal(format!("unsupported upstream feature: {feature}"))
            }
            RpcError::LocalUsageError(error) => {
                RpcErrorClassification::Internal(format!("local pre-processing error: {error}"))
            }
            RpcError::SerError(error) => {
                RpcErrorClassification::Internal(format!("upstream serialization error: {error}"))
            }
            RpcError::DeserError { err, .. } => RpcErrorClassification::Transport {
                class: TransportErrorClass::Decode,
                detail: Redacted::new(format!("upstream deserialization error: {err}")),
            },
        }
    }

    fn transport_kind_to_class_and_detail(
        kind: alloy_transport::TransportErrorKind,
    ) -> RpcErrorClassification {
        use alloy_transport::TransportErrorKind;

        match kind {
            TransportErrorKind::HttpError(http) => RpcErrorClassification::Transport {
                class: classify_status(http.status),
                detail: Redacted::new(format!(
                    "HTTP {} (body length {})",
                    http.status,
                    http.body.len()
                )),
            },
            TransportErrorKind::Custom(error) => RpcErrorClassification::Transport {
                class: TransportErrorClass::Other,
                detail: Redacted::new(error.to_string()),
            },
            TransportErrorKind::BackendGone => RpcErrorClassification::Transport {
                class: TransportErrorClass::Connect,
                detail: Redacted::new("backend connection task has stopped".to_owned()),
            },
            TransportErrorKind::PubsubUnavailable => RpcErrorClassification::Internal(
                "pubsub requested on a non-pubsub provider".to_owned(),
            ),
            TransportErrorKind::MissingBatchResponse(id) => {
                RpcErrorClassification::Internal(format!("missing batch response for id {id}"))
            }
            _ => RpcErrorClassification::Transport {
                class: TransportErrorClass::Other,
                detail: Redacted::new("unknown alloy transport error".to_owned()),
            },
        }
    }

    const fn classify_status(status: u16) -> TransportErrorClass {
        match status {
            408 => TransportErrorClass::Timeout,
            502..=504 => TransportErrorClass::Connect,
            400..=599 => TransportErrorClass::Status,
            _ => TransportErrorClass::Other,
        }
    }
}
