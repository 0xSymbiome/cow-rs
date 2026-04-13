use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::AppDataError;

/// Default `appCode` value inserted by [`crate::generate_app_data_doc`].
pub const DEFAULT_APP_CODE: &str = "CoW Swap";
/// Default public IPFS gateway used for read operations.
pub const DEFAULT_IPFS_READ_URI: &str = "https://cloudflare-ipfs.com/ipfs";
/// Default Pinata base URI used for write operations.
pub const DEFAULT_IPFS_WRITE_URI: &str = "https://api.pinata.cloud";
/// Latest bundled app-data schema version.
pub const LATEST_APP_DATA_VERSION: &str = "1.14.0";
/// Alias for the latest bundled schema version.
pub const LATEST_SCHEMA_VERSION: &str = LATEST_APP_DATA_VERSION;
/// Latest supported quote metadata schema version.
pub const LATEST_QUOTE_METADATA_VERSION: &str = "1.1.0";
/// Latest supported referrer metadata schema version.
pub const LATEST_REFERRER_METADATA_VERSION: &str = "1.0.0";
/// Latest supported order-class metadata schema version.
pub const LATEST_ORDER_CLASS_METADATA_VERSION: &str = "0.3.0";
/// Latest supported UTM metadata schema version.
pub const LATEST_UTM_METADATA_VERSION: &str = "0.3.0";
/// Latest supported hooks metadata schema version.
pub const LATEST_HOOKS_METADATA_VERSION: &str = "0.2.0";
/// Latest supported signer metadata schema version.
pub const LATEST_SIGNER_METADATA_VERSION: &str = "0.1.0";
/// Latest supported widget metadata schema version.
pub const LATEST_WIDGET_METADATA_VERSION: &str = "0.1.0";
/// Latest supported partner-fee metadata schema version.
pub const LATEST_PARTNER_FEE_METADATA_VERSION: &str = "1.0.0";
/// Latest supported replaced-order metadata schema version.
pub const LATEST_REPLACED_ORDER_METADATA_VERSION: &str = "0.1.0";
/// Latest supported wrappers metadata schema version.
pub const LATEST_WRAPPERS_METADATA_VERSION: &str = "0.2.0";
/// Latest supported user-consents metadata schema version.
pub const LATEST_USER_CONSENTS_METADATA_VERSION: &str = "0.1.0";

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
            Err(AppDataError::InvalidSchemaVersion(version))
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

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::latest()
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

/// Inputs used to build an app-data document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AppDataParams {
    /// Optional application name written to the `appCode` field.
    #[serde(default, rename = "appCode", skip_serializing_if = "Option::is_none")]
    pub app_code: Option<String>,
    /// Optional environment label for distinguishing deployments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    /// Arbitrary application metadata merged into the document.
    #[serde(default)]
    pub metadata: MetadataMap,
}

/// Derived identifiers for a validated app-data document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppDataInfo {
    /// CID representation of the document.
    pub cid: String,
    /// Serialized JSON content used to derive the digest.
    #[serde(rename = "appDataContent")]
    pub app_data_content: String,
    /// `0x`-prefixed app-data digest.
    #[serde(rename = "appDataHex")]
    pub app_data_hex: String,
}

/// Schema validation result returned by [`crate::validate_app_data_doc`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether validation succeeded.
    pub success: bool,
    /// Rendered validation errors when `success` is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
}

/// IPFS configuration used by fetch and upload helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IpfsConfig {
    /// Legacy shared base URI used when `read_uri` is absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    /// Base URI used for Pinata-style write requests.
    #[serde(default, rename = "writeUri", skip_serializing_if = "Option::is_none")]
    pub write_uri: Option<String>,
    /// Base URI used for IPFS read requests.
    #[serde(default, rename = "readUri", skip_serializing_if = "Option::is_none")]
    pub read_uri: Option<String>,
    /// Pinata API key used by upload helpers.
    #[serde(
        default,
        rename = "pinataApiKey",
        skip_serializing_if = "Option::is_none"
    )]
    pub pinata_api_key: Option<String>,
    /// Pinata API secret used by upload helpers.
    #[serde(
        default,
        rename = "pinataApiSecret",
        skip_serializing_if = "Option::is_none"
    )]
    pub pinata_api_secret: Option<String>,
}

/// Raw HTTP response returned by app-data transport seams.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response body text.
    pub body: String,
}

/// Result returned by legacy Pinata upload helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpfsUploadResult {
    /// `0x`-prefixed app-data digest derived from the returned CID.
    #[serde(rename = "appData")]
    pub app_data: String,
    /// CID returned by the upload backend.
    pub cid: String,
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
        assert_eq!(SchemaVersion::default(), latest);
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
            assert_eq!(
                SchemaVersion::new(invalid).unwrap_err(),
                AppDataError::InvalidSchemaVersion(invalid.to_owned())
            );
        }

        assert!(is_non_empty_digits("123456"));
        assert!(!is_non_empty_digits(""));
        assert!(!is_non_empty_digits("12a45"));
    }

    #[test]
    fn schema_version_from_str_fails_closed_for_invalid_inputs() {
        for invalid in ["1.0", "1.0.0.1", "v1.0.0", "1.two.3", "", "1..3"] {
            assert_eq!(
                invalid.parse::<SchemaVersion>().unwrap_err(),
                AppDataError::InvalidSchemaVersion(invalid.to_owned())
            );
        }
    }
}
