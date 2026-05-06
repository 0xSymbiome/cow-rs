//! Error types for the native Alloy signer adapter.

use std::{error::Error, fmt};

use cow_sdk_core::Redacted;

/// Coarse classification for [`AsyncSignerError`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AsyncSignerErrorClass {
    /// Caller-controlled input failed validation.
    Validation,
    /// The upstream signing backend failed.
    Signing,
    /// The requested method requires provider context unavailable to this leaf signer.
    ProviderRequired,
    /// The method is disabled by feature selection or unsupported capability.
    Unsupported,
    /// The future was cancelled by a consumer-provided cancellation token.
    Cancelled,
    /// A local invariant or internal conversion failed.
    Internal,
}

impl AsyncSignerErrorClass {
    /// Returns the stable lowercase class label.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Validation => "validation",
            Self::Signing => "signing",
            Self::ProviderRequired => "provider_required",
            Self::Unsupported => "unsupported",
            Self::Cancelled => "cancelled",
            Self::Internal => "internal",
        }
    }
}

impl fmt::Display for AsyncSignerErrorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error returned by [`crate::LocalAlloyKeystoreSigner`].
#[non_exhaustive]
pub enum AsyncSignerError {
    /// Caller-controlled input failed validation.
    Validation(String),
    /// The upstream signer failed.
    Signing {
        /// Redacted signing detail.
        detail: Redacted<String>,
    },
    /// A provider is required for this method.
    ProviderRequired {
        /// Method that requires provider context.
        method: &'static str,
    },
    /// The requested operation is unsupported.
    Unsupported(&'static str),
    /// The operation was cancelled by [`cow_sdk_core::Cancellable`].
    Cancelled,
    /// A local invariant or internal conversion failed.
    Internal(String),
}

impl AsyncSignerError {
    /// Returns this error's coarse class.
    #[must_use]
    pub const fn class(&self) -> AsyncSignerErrorClass {
        match self {
            Self::Validation(_) => AsyncSignerErrorClass::Validation,
            Self::Signing { .. } => AsyncSignerErrorClass::Signing,
            Self::ProviderRequired { .. } => AsyncSignerErrorClass::ProviderRequired,
            Self::Unsupported(_) => AsyncSignerErrorClass::Unsupported,
            Self::Cancelled => AsyncSignerErrorClass::Cancelled,
            Self::Internal(_) => AsyncSignerErrorClass::Internal,
        }
    }

    /// Inter-crate seam constructor; not part of the semver-stable consumer
    /// API. Sibling adapter crates use this to lift Alloy signer errors into
    /// the signer's typed error surface. The argument shape may change in any
    /// minor release.
    #[doc(hidden)]
    #[must_use]
    pub fn from_alloy_signer(error: &alloy_signer::Error) -> Self {
        Self::Signing {
            detail: Redacted::new(error.to_string()),
        }
    }
}

impl fmt::Debug for AsyncSignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(_) => f
                .debug_tuple("Validation")
                .field(&Redacted::new("validation detail"))
                .finish(),
            Self::Signing { detail } => f.debug_struct("Signing").field("detail", detail).finish(),
            Self::ProviderRequired { method } => f
                .debug_struct("ProviderRequired")
                .field("method", method)
                .finish(),
            Self::Unsupported(message) => f.debug_tuple("Unsupported").field(message).finish(),
            Self::Cancelled => f.write_str("Cancelled"),
            Self::Internal(_) => f
                .debug_tuple("Internal")
                .field(&Redacted::new("internal detail"))
                .finish(),
        }
    }
}

impl fmt::Display for AsyncSignerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Validation(_) => f.write_str("validation error: [redacted]"),
            Self::Signing { detail } => write!(f, "signing error: {detail}"),
            Self::ProviderRequired { method } => {
                write!(f, "the {method} method requires a provider")
            }
            Self::Unsupported(message) => write!(f, "unsupported: {message}"),
            Self::Cancelled => f.write_str("operation cancelled"),
            Self::Internal(_) => f.write_str("internal error: [redacted]"),
        }
    }
}

impl Error for AsyncSignerError {}

impl From<cow_sdk_contracts::ContractsError> for AsyncSignerError {
    fn from(error: cow_sdk_contracts::ContractsError) -> Self {
        Self::Signing {
            detail: Redacted::new(error.to_string()),
        }
    }
}

impl From<cow_sdk_core::Cancelled> for AsyncSignerError {
    fn from(_: cow_sdk_core::Cancelled) -> Self {
        Self::Cancelled
    }
}

#[cfg(test)]
mod tests {
    use cow_sdk_core::Redacted;

    use super::*;

    #[test]
    fn class_returns_expected_discriminant_for_every_variant() {
        let cases = [
            (
                AsyncSignerError::Validation("invalid".to_owned()),
                AsyncSignerErrorClass::Validation,
            ),
            (
                AsyncSignerError::Signing {
                    detail: Redacted::new("secret".to_owned()),
                },
                AsyncSignerErrorClass::Signing,
            ),
            (
                AsyncSignerError::ProviderRequired { method: "send" },
                AsyncSignerErrorClass::ProviderRequired,
            ),
            (
                AsyncSignerError::Unsupported("unsupported"),
                AsyncSignerErrorClass::Unsupported,
            ),
            (
                AsyncSignerError::Cancelled,
                AsyncSignerErrorClass::Cancelled,
            ),
            (
                AsyncSignerError::Internal("secret".to_owned()),
                AsyncSignerErrorClass::Internal,
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.class(), expected);
            assert_eq!(error.class().to_string(), expected.as_str());
        }
    }

    #[test]
    fn display_redacts_signing_detail() {
        let error = AsyncSignerError::Signing {
            detail: Redacted::new("private-key-fragment".to_owned()),
        };

        let display = error.to_string();
        let debug = format!("{error:?}");
        assert!(display.contains("[redacted]"));
        assert!(debug.contains("[redacted]"));
        assert!(!display.contains("private-key-fragment"));
        assert!(!debug.contains("private-key-fragment"));
    }

    #[test]
    fn provider_required_includes_method_name() {
        let error = AsyncSignerError::ProviderRequired {
            method: "estimate_gas",
        };

        assert!(error.to_string().contains("estimate_gas"));
    }

    #[test]
    fn unsupported_includes_static_message() {
        let error = AsyncSignerError::Unsupported("typed data disabled");

        assert!(error.to_string().contains("typed data disabled"));
    }
}
