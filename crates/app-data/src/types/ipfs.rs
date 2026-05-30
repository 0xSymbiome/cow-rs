use std::fmt;

use cow_sdk_core::Redacted;
use serde::{Deserialize, Serialize};

/// IPFS configuration used by fetch and upload helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IpfsConfig {
    /// Legacy shared base URI used when `read_uri` is absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Redacted<String>>,
    /// Base URI used for Pinata-style write requests.
    #[serde(default, rename = "writeUri", skip_serializing_if = "Option::is_none")]
    pub write_uri: Option<Redacted<String>>,
    /// Base URI used for IPFS read requests.
    #[serde(default, rename = "readUri", skip_serializing_if = "Option::is_none")]
    pub read_uri: Option<Redacted<String>>,
    /// Pinata API key used by upload helpers.
    #[serde(
        default,
        rename = "pinataApiKey",
        skip_serializing_if = "Option::is_none"
    )]
    pub pinata_api_key: Option<Redacted<String>>,
    /// Pinata API secret used by upload helpers.
    #[serde(
        default,
        rename = "pinataApiSecret",
        skip_serializing_if = "Option::is_none"
    )]
    pub pinata_api_secret: Option<Redacted<String>>,
}

impl fmt::Display for IpfsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpfsConfig")
            .field("uri", &self.uri)
            .field("write_uri", &self.write_uri)
            .field("read_uri", &self.read_uri)
            .field("pinata_api_key", &self.pinata_api_key)
            .field("pinata_api_secret", &self.pinata_api_secret)
            .finish()
    }
}

/// Raw HTTP response returned by app-data transport seams.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct TransportResponse {
    /// HTTP status code.
    pub status: u16,
    /// Response body text.
    pub body: String,
}

impl TransportResponse {
    /// Creates a raw HTTP response for app-data transport seams.
    #[must_use]
    pub fn new(status: u16, body: impl Into<String>) -> Self {
        Self {
            status,
            body: body.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipfs_config_debug_and_serialize_redact_pinata_credentials() {
        let config = IpfsConfig {
            uri: Some("https://ipfs.example".to_owned().into()),
            write_uri: Some("https://pinata.example".to_owned().into()),
            read_uri: Some("https://read.example".to_owned().into()),
            pinata_api_key: Some("pinata-key".to_owned().into()),
            pinata_api_secret: Some("pinata-secret".to_owned().into()),
        };

        let debug = format!("{config:?}");
        let json = serde_json::to_value(&config).expect("ipfs config serializes");

        assert!(debug.contains("IpfsConfig"));
        assert!(debug.contains(cow_sdk_core::REDACTED_PLACEHOLDER));
        assert!(!debug.contains("ipfs.example"));
        assert!(!debug.contains("pinata.example"));
        assert!(!debug.contains("read.example"));
        assert!(!debug.contains("pinata-key"));
        assert!(!debug.contains("pinata-secret"));
        assert_eq!(
            json["pinataApiKey"],
            serde_json::json!(cow_sdk_core::REDACTED_PLACEHOLDER)
        );
        assert_eq!(
            json["pinataApiSecret"],
            serde_json::json!(cow_sdk_core::REDACTED_PLACEHOLDER)
        );
        assert_eq!(
            json["uri"],
            serde_json::json!(cow_sdk_core::REDACTED_PLACEHOLDER)
        );
        assert_eq!(
            json["writeUri"],
            serde_json::json!(cow_sdk_core::REDACTED_PLACEHOLDER)
        );
        assert_eq!(
            json["readUri"],
            serde_json::json!(cow_sdk_core::REDACTED_PLACEHOLDER)
        );
    }
}
