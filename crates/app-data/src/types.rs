use std::{fmt, str::FromStr};

use cow_sdk_core::Address;
use serde::ser::SerializeStruct;
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

const REDACTED_SECRET: &str = "<redacted>";

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
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `metadata: MetadataMap` field is a `serde_json::Map<String, serde_json::Value>` alias, and `serde_json::Value` does not implement `Eq`"
)]
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

/// Typed partner-fee metadata accepted by app-data and trading helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PartnerFee {
    /// Single fee policy object.
    Single(PartnerFeePolicy),
    /// Ordered fee policy list.
    Multiple(Vec<PartnerFeePolicy>),
}

impl PartnerFee {
    /// Returns the first supported volume-basis-point fee in this value, if one exists.
    #[must_use]
    pub fn volume_bps(&self) -> Option<u32> {
        match self {
            Self::Single(policy) => policy.volume_bps(),
            Self::Multiple(policies) => policies.iter().find_map(PartnerFeePolicy::volume_bps),
        }
    }

    /// Serializes this typed partner-fee payload into the app-data metadata shape.
    ///
    /// # Panics
    ///
    /// Panics only if the compile-time partner-fee schema types stop being
    /// serializable to JSON.
    #[must_use]
    pub fn to_value(&self) -> Value {
        serde_json::to_value(self).expect("partner-fee schema types must remain serializable")
    }

    /// Parses partner-fee metadata from an app-data metadata value.
    ///
    /// # Errors
    ///
    /// Returns the underlying serde error when the JSON value does not match
    /// the supported partner-fee schema shape.
    pub fn from_value(value: Value) -> Result<Self, serde_json::Error> {
        serde_json::from_value(value)
    }
}

impl From<PartnerFeePolicy> for PartnerFee {
    fn from(value: PartnerFeePolicy) -> Self {
        Self::Single(value)
    }
}

impl From<Vec<PartnerFeePolicy>> for PartnerFee {
    fn from(value: Vec<PartnerFeePolicy>) -> Self {
        Self::Multiple(value)
    }
}

/// One typed partner-fee policy object.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PartnerFeePolicy {
    /// Fee paid from traded volume.
    Volume {
        /// Fee paid in basis points of volume.
        #[serde(rename = "volumeBps")]
        volume_bps: u32,
        /// Recipient of the partner fee.
        recipient: Address,
    },
    /// Fee paid from surplus, capped by volume.
    Surplus {
        /// Fee paid in basis points of surplus.
        #[serde(rename = "surplusBps")]
        surplus_bps: u32,
        /// Maximum fee paid in basis points of volume.
        #[serde(rename = "maxVolumeBps")]
        max_volume_bps: u32,
        /// Recipient of the partner fee.
        recipient: Address,
    },
    /// Fee paid from price improvement, capped by volume.
    PriceImprovement {
        /// Fee paid in basis points of price improvement.
        #[serde(rename = "priceImprovementBps")]
        price_improvement_bps: u32,
        /// Maximum fee paid in basis points of volume.
        #[serde(rename = "maxVolumeBps")]
        max_volume_bps: u32,
        /// Recipient of the partner fee.
        recipient: Address,
    },
}

impl PartnerFeePolicy {
    /// Creates a volume-based partner-fee policy.
    #[must_use]
    pub fn volume(volume_bps: u32, recipient: Address) -> Self {
        Self::Volume {
            volume_bps,
            recipient,
        }
    }

    /// Creates a surplus-based partner-fee policy.
    #[must_use]
    pub fn surplus(surplus_bps: u32, max_volume_bps: u32, recipient: Address) -> Self {
        Self::Surplus {
            surplus_bps,
            max_volume_bps,
            recipient,
        }
    }

    /// Creates a price-improvement-based partner-fee policy.
    #[must_use]
    pub fn price_improvement(
        price_improvement_bps: u32,
        max_volume_bps: u32,
        recipient: Address,
    ) -> Self {
        Self::PriceImprovement {
            price_improvement_bps,
            max_volume_bps,
            recipient,
        }
    }

    /// Returns the volume-basis-point fee when this policy uses the volume shape.
    #[must_use]
    pub fn volume_bps(&self) -> Option<u32> {
        match self {
            Self::Volume { volume_bps, .. } => Some(*volume_bps),
            Self::Surplus { .. } | Self::PriceImprovement { .. } => None,
        }
    }
}

/// IPFS configuration used by fetch and upload helpers.
#[derive(Clone, PartialEq, Eq, Deserialize, Default)]
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

impl fmt::Debug for IpfsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpfsConfig")
            .field("uri", &self.uri)
            .field("write_uri", &self.write_uri)
            .field("read_uri", &self.read_uri)
            .field(
                "pinata_api_key",
                &redacted_secret_option(&self.pinata_api_key),
            )
            .field(
                "pinata_api_secret",
                &redacted_secret_option(&self.pinata_api_secret),
            )
            .finish()
    }
}

impl Serialize for IpfsConfig {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut state = serializer.serialize_struct("IpfsConfig", 5)?;

        if let Some(uri) = &self.uri {
            state.serialize_field("uri", uri)?;
        }
        if let Some(write_uri) = &self.write_uri {
            state.serialize_field("writeUri", write_uri)?;
        }
        if let Some(read_uri) = &self.read_uri {
            state.serialize_field("readUri", read_uri)?;
        }
        if self.pinata_api_key.is_some() {
            state.serialize_field("pinataApiKey", REDACTED_SECRET)?;
        }
        if self.pinata_api_secret.is_some() {
            state.serialize_field("pinataApiSecret", REDACTED_SECRET)?;
        }

        state.end()
    }
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

fn redacted_secret_option(value: &Option<String>) -> Option<&'static str> {
    value.as_ref().map(|_| REDACTED_SECRET)
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

    #[test]
    fn partner_fee_roundtrips_single_and_array_shapes_and_exposes_first_volume_fee() {
        let recipient = Address::new("0x1111111111111111111111111111111111111111")
            .expect("test recipient must be valid");
        let fee = PartnerFee::from(vec![
            PartnerFeePolicy::surplus(250, 100, recipient.clone()),
            PartnerFeePolicy::volume(42, recipient.clone()),
        ]);

        let value = fee.to_value();
        let reparsed = PartnerFee::from_value(value.clone()).expect("typed partner fee re-parses");

        assert_eq!(
            value,
            serde_json::json!([
                {
                    "surplusBps": 250,
                    "maxVolumeBps": 100,
                    "recipient": recipient.as_str()
                },
                {
                    "volumeBps": 42,
                    "recipient": recipient.as_str()
                }
            ])
        );
        assert_eq!(reparsed, fee);
        assert_eq!(fee.volume_bps(), Some(42));
        assert_eq!(
            PartnerFee::from(PartnerFeePolicy::price_improvement(25, 100, recipient)).volume_bps(),
            None
        );
    }

    #[test]
    fn ipfs_config_debug_and_serialize_redact_pinata_credentials() {
        let config = IpfsConfig {
            uri: Some("https://ipfs.example".to_owned()),
            write_uri: Some("https://pinata.example".to_owned()),
            read_uri: Some("https://read.example".to_owned()),
            pinata_api_key: Some("pinata-key".to_owned()),
            pinata_api_secret: Some("pinata-secret".to_owned()),
        };

        let debug = format!("{config:?}");
        let json = serde_json::to_value(&config).expect("ipfs config serializes");

        assert!(debug.contains("IpfsConfig"));
        assert!(debug.contains(REDACTED_SECRET));
        assert!(!debug.contains("pinata-key"));
        assert!(!debug.contains("pinata-secret"));
        assert_eq!(json["pinataApiKey"], serde_json::json!(REDACTED_SECRET));
        assert_eq!(json["pinataApiSecret"], serde_json::json!(REDACTED_SECRET));
    }
}
