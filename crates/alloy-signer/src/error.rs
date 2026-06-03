//! Error types for the native Alloy signer adapter.

use std::{error::Error, fmt};

use cow_sdk_core::Redacted;

/// Coarse classification for [`SignerError`].
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SignerErrorClass {
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

impl SignerErrorClass {
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

impl fmt::Display for SignerErrorClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error returned by [`crate::LocalAlloyKeystoreSigner`].
#[non_exhaustive]
pub enum SignerError {
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

impl SignerError {
    /// Returns this error's coarse class.
    #[must_use]
    pub const fn class(&self) -> SignerErrorClass {
        match self {
            Self::Validation(_) => SignerErrorClass::Validation,
            Self::Signing { .. } => SignerErrorClass::Signing,
            Self::ProviderRequired { .. } => SignerErrorClass::ProviderRequired,
            Self::Unsupported(_) => SignerErrorClass::Unsupported,
            Self::Cancelled => SignerErrorClass::Cancelled,
            Self::Internal(_) => SignerErrorClass::Internal,
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

impl fmt::Debug for SignerError {
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

impl fmt::Display for SignerError {
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

impl Error for SignerError {}

impl From<cow_sdk_contracts::ContractsError> for SignerError {
    fn from(error: cow_sdk_contracts::ContractsError) -> Self {
        Self::Signing {
            detail: Redacted::new(error.to_string()),
        }
    }
}

impl From<cow_sdk_core::Cancelled> for SignerError {
    fn from(_: cow_sdk_core::Cancelled) -> Self {
        Self::Cancelled
    }
}

/// `LocalAlloyKeystoreSigner` operates on a locally-held private key
/// and never goes through an EIP-1193 provider, so no variant of
/// `SignerError` can represent a user rejection in the sense
/// EIP-1193 defines (codes `4001`, `4100`, etc.). The trait impl
/// returns `None` for every variant, which routes every leaf-signer
/// failure through the redacted `SigningError::Signer` path. If a
/// future alloy-signer variant ever represents an EIP-1193 rejection
/// surfaced by an external transport, the new code must extend this
/// impl alongside the new variant so the signing crate can re-classify
/// it. The contract is pinned by
/// `crates/alloy-signer/tests/signer_error_trait_contract.rs`.
impl cow_sdk_core::SignerError for SignerError {
    fn user_rejection_code(&self) -> Option<i32> {
        match self {
            Self::Validation(_)
            | Self::Signing { .. }
            | Self::ProviderRequired { .. }
            | Self::Unsupported(_)
            | Self::Cancelled
            | Self::Internal(_) => None,
        }
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
                SignerError::Validation("invalid".to_owned()),
                SignerErrorClass::Validation,
            ),
            (
                SignerError::Signing {
                    detail: Redacted::new("secret".to_owned()),
                },
                SignerErrorClass::Signing,
            ),
            (
                SignerError::ProviderRequired { method: "send" },
                SignerErrorClass::ProviderRequired,
            ),
            (
                SignerError::Unsupported("unsupported"),
                SignerErrorClass::Unsupported,
            ),
            (SignerError::Cancelled, SignerErrorClass::Cancelled),
            (
                SignerError::Internal("secret".to_owned()),
                SignerErrorClass::Internal,
            ),
        ];

        for (error, expected) in cases {
            assert_eq!(error.class(), expected);
            assert_eq!(error.class().to_string(), expected.as_str());
        }
    }
}
