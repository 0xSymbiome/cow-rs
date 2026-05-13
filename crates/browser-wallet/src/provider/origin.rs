use std::fmt;

use cow_sdk_core::Redacted;

use crate::BrowserWalletError;

/// Reviewed origin label for an EIP-1193 provider binding.
///
/// The value can be a browser origin or an EIP-6963 reverse-DNS identifier.
/// Its debug and display representations are redacted; use [`Origin::as_str`]
/// only when the caller deliberately needs the raw value.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Origin(String);

impl Origin {
    /// Creates a non-empty provider origin label.
    ///
    /// # Errors
    ///
    /// Returns [`BrowserWalletError`] when the origin is empty or contains
    /// control characters.
    pub fn new(value: impl Into<String>) -> Result<Self, BrowserWalletError> {
        let value = value.into();
        let trimmed = value.trim();
        if trimmed.is_empty() {
            return Err(BrowserWalletError::InvalidProviderOrigin {
                message: "provider origin must not be empty".to_owned().into(),
            });
        }
        if trimmed.chars().any(char::is_control) {
            return Err(BrowserWalletError::InvalidProviderOrigin {
                message: "provider origin must not contain control characters"
                    .to_owned()
                    .into(),
            });
        }
        if !origin_scheme_is_documented(trimmed) {
            return Err(BrowserWalletError::InvalidProviderOrigin {
                message:
                    "provider origin scheme must be http, https, test, transport, or reverse-DNS"
                        .to_owned()
                        .into(),
            });
        }
        Ok(Self(trimmed.to_owned()))
    }

    /// Returns the raw provider origin label.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

fn origin_scheme_is_documented(value: &str) -> bool {
    let Some((scheme, _)) = value.split_once(':') else {
        return true;
    };
    matches!(
        scheme.to_ascii_lowercase().as_str(),
        "http" | "https" | "test" | "transport"
    )
}

impl fmt::Debug for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Redacted::new(self.0.clone()).fmt(f)
    }
}

impl fmt::Display for Origin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Redacted::new(self.0.clone()).fmt(f)
    }
}
