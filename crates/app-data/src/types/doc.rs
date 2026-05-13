use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::AppDataError;

use super::LATEST_APP_DATA_VERSION;

/// Parsed app-data JSON document.
pub type AppDataDoc = Value;
/// Mutable JSON object used for nested `metadata` sections.
pub type MetadataMap = Map<String, Value>;

/// Semantic version for bundled app-data schemas.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(String);

impl SchemaVersion {
    /// Creates a validated schema version string.
    ///
    /// # Errors
    ///
    /// Returns [`AppDataError::InvalidSchemaVersion`] when the value is not a
    /// three-part numeric semantic version.
    pub fn new(version: impl Into<String>) -> Result<Self, AppDataError> {
        let version = version.into();
        if is_semver(&version) {
            Ok(Self(version))
        } else {
            Err(AppDataError::InvalidSchemaVersion(version.into()))
        }
    }

    /// Returns the latest bundled schema version.
    #[must_use]
    pub fn latest() -> Self {
        Self(LATEST_APP_DATA_VERSION.to_string())
    }

    /// Returns the inner schema version string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl FromStr for SchemaVersion {
    type Err = AppDataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::new(s)
    }
}

fn is_semver(version: &str) -> bool {
    let mut parts = version.split('.');
    let major = parts.next().is_some_and(is_non_empty_digits);
    let minor = parts.next().is_some_and(is_non_empty_digits);
    let patch = parts.next().is_some_and(is_non_empty_digits);
    major && minor && patch && parts.next().is_none()
}

fn is_non_empty_digits(value: &str) -> bool {
    !value.is_empty() && value.chars().all(|ch| ch.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_version_latest_default_display_and_parse_stay_aligned() {
        let latest = SchemaVersion::latest();

        assert_eq!(latest.as_str(), LATEST_APP_DATA_VERSION);
        assert_eq!(latest.to_string(), LATEST_APP_DATA_VERSION);
        assert_eq!(
            LATEST_APP_DATA_VERSION.parse::<SchemaVersion>().unwrap(),
            latest
        );
    }

    #[test]
    fn schema_version_validation_accepts_triplets_and_rejects_non_semver_inputs() {
        for valid in ["0.1.0", "1.14.0", "999.0.42"] {
            assert!(is_semver(valid), "{valid}");
            assert_eq!(SchemaVersion::new(valid).unwrap().as_str(), valid);
        }

        for invalid in ["1.0", "1.0.0.1", "v1.0.0", "1.two.3", "", "1..3"] {
            assert!(!is_semver(invalid), "{invalid}");
            let error = SchemaVersion::new(invalid).unwrap_err();
            match error {
                AppDataError::InvalidSchemaVersion(ref message) => {
                    assert_eq!(message.as_inner(), invalid);
                }
                other => panic!("expected InvalidSchemaVersion, got {other:?}"),
            }
        }

        assert!(is_non_empty_digits("123456"));
        assert!(!is_non_empty_digits(""));
        assert!(!is_non_empty_digits("12a45"));
    }

    #[test]
    fn schema_version_from_str_fails_closed_for_invalid_inputs() {
        for invalid in ["1.0", "1.0.0.1", "v1.0.0", "1.two.3", "", "1..3"] {
            let error = invalid.parse::<SchemaVersion>().unwrap_err();
            match error {
                AppDataError::InvalidSchemaVersion(ref message) => {
                    assert_eq!(message.as_inner(), invalid);
                }
                other => panic!("expected InvalidSchemaVersion, got {other:?}"),
            }
        }
    }
}
