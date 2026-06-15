use std::{fmt, str::FromStr};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

/// Validated application identifier written into generated app-data documents.
///
/// `AppCode` is deliberately permissive and mirrors the source-backed minimum
/// accepted by the upstream app-data schema: any non-empty UTF-8 string is
/// accepted as long as it does not contain NUL or ASCII control characters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AppCode(String);

/// Validation failures returned while constructing an [`AppCode`].
///
/// The variants intentionally do not carry the rejected input. `appCode`
/// values identify caller applications and can be built from user-controlled
/// configuration, so public diagnostics expose only the validation class.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AppCodeError {
    /// The supplied app code was empty.
    #[error("appCode must not be empty")]
    Empty,
    /// The supplied app code contained a NUL byte.
    #[error("appCode must not contain NUL bytes")]
    NulByte,
    /// The supplied app code contained an ASCII control character.
    #[error("appCode must not contain control characters")]
    ControlCharacter,
}

impl AppCode {
    /// Creates a validated app code.
    ///
    /// Accepted values are intentionally broad. Spaces, slashes, underscores,
    /// mixed case, and long UTF-8 strings are valid when the value is non-empty
    /// and contains no NUL or ASCII control characters.
    ///
    /// ```
    /// use cow_sdk_core::AppCode;
    ///
    /// assert!(AppCode::new("CoW Swap").is_ok());
    /// assert!(AppCode::new("cow-rs/wasm-console").is_ok());
    /// assert!(AppCode::new("COW_BRIDGING_REACT_EXAMPLE").is_ok());
    /// ```
    ///
    /// ```
    /// use cow_sdk_core::{AppCode, AppCodeError};
    ///
    /// assert!(matches!(AppCode::new(""), Err(AppCodeError::Empty)));
    /// assert!(matches!(
    ///     AppCode::new("cow-rs\nconsole"),
    ///     Err(AppCodeError::ControlCharacter)
    /// ));
    /// assert!(matches!(
    ///     AppCode::new("cow-rs\0console"),
    ///     Err(AppCodeError::NulByte)
    /// ));
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`AppCodeError`] when the value is empty or contains forbidden
    /// control characters.
    pub fn new(value: impl Into<String>) -> Result<Self, AppCodeError> {
        let value = value.into();
        if value.is_empty() {
            return Err(AppCodeError::Empty);
        }
        if value.as_bytes().contains(&b'\0') {
            return Err(AppCodeError::NulByte);
        }
        if value
            .chars()
            .any(|character| matches!(character, '\u{0001}'..='\u{001F}' | '\u{007F}'))
        {
            return Err(AppCodeError::ControlCharacter);
        }

        Ok(Self(value))
    }

    /// Returns the validated string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Returns the owned validated string value.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.0
    }
}

impl AsRef<str> for AppCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl fmt::Display for AppCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl FromStr for AppCode {
    type Err = AppCodeError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}

impl From<std::convert::Infallible> for AppCodeError {
    fn from(error: std::convert::Infallible) -> Self {
        match error {}
    }
}

impl TryFrom<&str> for AppCode {
    type Error = AppCodeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<String> for AppCode {
    type Error = AppCodeError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Serialize for AppCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for AppCode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::{AppCode, AppCodeError};

    #[test]
    fn accessors_return_the_inner_string() {
        let code = AppCode::new("cow-rs").expect("valid app code is accepted");
        assert_eq!(code.as_str(), "cow-rs");
        assert_eq!(<AppCode as AsRef<str>>::as_ref(&code), "cow-rs");
        assert_eq!(format!("{code}"), "cow-rs"); // Display
        assert_eq!(code.into_inner(), "cow-rs");
    }

    #[test]
    fn from_str_and_try_from_round_trip() {
        let code: AppCode = "cow-rs".parse().expect("FromStr accepts a valid app code");
        assert_eq!(code.as_str(), "cow-rs");

        let invalid: Result<AppCode, _> = "".parse();
        assert_eq!(invalid, Err(AppCodeError::Empty));

        let from_ref: AppCode = "cow-rs"
            .try_into()
            .expect("TryFrom<&str> accepts a valid app code");
        assert_eq!(from_ref.as_str(), "cow-rs");

        let from_owned: AppCode = String::from("cow-rs")
            .try_into()
            .expect("TryFrom<String> accepts a valid app code");
        assert_eq!(from_owned.as_str(), "cow-rs");
    }

    #[test]
    fn serde_round_trips_through_json() {
        let code = AppCode::new("cow-rs/wasm").expect("valid app code is accepted");
        let serialized = serde_json::to_string(&code).expect("app code serializes");
        assert_eq!(serialized, r#""cow-rs/wasm""#);

        let deserialized: AppCode =
            serde_json::from_str(&serialized).expect("app code deserializes");
        assert_eq!(deserialized, code);

        // Deserialization runs the same validation as construction.
        let empty: Result<AppCode, _> = serde_json::from_str(r#""""#);
        assert!(empty.is_err());
        let control: Result<AppCode, _> = serde_json::from_str(r#""cow\nrs""#);
        assert!(control.is_err());
    }
}
