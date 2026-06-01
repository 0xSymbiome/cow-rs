use std::fmt;

use cow_sdk_core::Redacted;
use serde::{Deserialize, Serialize};

/// IPFS configuration used by app-data read helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IpfsConfig {
    /// Legacy shared base URI used when `read_uri` is absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Redacted<String>>,
    /// Base URI used for IPFS read requests.
    #[serde(default, rename = "readUri", skip_serializing_if = "Option::is_none")]
    pub read_uri: Option<Redacted<String>>,
}

impl fmt::Display for IpfsConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("IpfsConfig")
            .field("uri", &self.uri)
            .field("read_uri", &self.read_uri)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipfs_config_debug_and_serialize_redact_configured_uris() {
        let config = IpfsConfig {
            uri: Some("https://ipfs.example".to_owned().into()),
            read_uri: Some("https://read.example".to_owned().into()),
        };

        let debug = format!("{config:?}");
        let json = serde_json::to_value(&config).expect("ipfs config serializes");

        assert!(debug.contains("IpfsConfig"));
        assert!(debug.contains(cow_sdk_core::REDACTED_PLACEHOLDER));
        assert!(!debug.contains("ipfs.example"));
        assert!(!debug.contains("read.example"));
        assert_eq!(
            json["uri"],
            serde_json::json!(cow_sdk_core::REDACTED_PLACEHOLDER)
        );
        assert_eq!(
            json["readUri"],
            serde_json::json!(cow_sdk_core::REDACTED_PLACEHOLDER)
        );
    }
}
