use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::AppDataError;

pub const DEFAULT_APP_CODE: &str = "CoW Swap";
pub const DEFAULT_IPFS_READ_URI: &str = "https://cloudflare-ipfs.com/ipfs";
pub const DEFAULT_IPFS_WRITE_URI: &str = "https://api.pinata.cloud";
pub const LATEST_APP_DATA_VERSION: &str = "1.14.0";
pub const LATEST_SCHEMA_VERSION: &str = LATEST_APP_DATA_VERSION;
pub const LATEST_QUOTE_METADATA_VERSION: &str = "1.1.0";
pub const LATEST_REFERRER_METADATA_VERSION: &str = "1.0.0";
pub const LATEST_ORDER_CLASS_METADATA_VERSION: &str = "0.3.0";
pub const LATEST_UTM_METADATA_VERSION: &str = "0.3.0";
pub const LATEST_HOOKS_METADATA_VERSION: &str = "0.2.0";
pub const LATEST_SIGNER_METADATA_VERSION: &str = "0.1.0";
pub const LATEST_WIDGET_METADATA_VERSION: &str = "0.1.0";
pub const LATEST_PARTNER_FEE_METADATA_VERSION: &str = "1.0.0";
pub const LATEST_REPLACED_ORDER_METADATA_VERSION: &str = "0.1.0";
pub const LATEST_WRAPPERS_METADATA_VERSION: &str = "0.2.0";
pub const LATEST_USER_CONSENTS_METADATA_VERSION: &str = "0.1.0";

pub type AppDataDoc = Value;
pub type MetadataMap = Map<String, Value>;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SchemaVersion(String);

impl SchemaVersion {
    pub fn new(version: impl Into<String>) -> Result<Self, AppDataError> {
        let version = version.into();
        if is_semver(&version) {
            Ok(Self(version))
        } else {
            Err(AppDataError::InvalidSchemaVersion(version))
        }
    }

    pub fn latest() -> Self {
        Self(LATEST_APP_DATA_VERSION.to_string())
    }

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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct AppDataParams {
    #[serde(default, rename = "appCode", skip_serializing_if = "Option::is_none")]
    pub app_code: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
    #[serde(default)]
    pub metadata: MetadataMap,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppDataInfo {
    pub cid: String,
    #[serde(rename = "appDataContent")]
    pub app_data_content: String,
    #[serde(rename = "appDataHex")]
    pub app_data_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ValidationResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct IpfsConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<String>,
    #[serde(default, rename = "writeUri", skip_serializing_if = "Option::is_none")]
    pub write_uri: Option<String>,
    #[serde(default, rename = "readUri", skip_serializing_if = "Option::is_none")]
    pub read_uri: Option<String>,
    #[serde(
        default,
        rename = "pinataApiKey",
        skip_serializing_if = "Option::is_none"
    )]
    pub pinata_api_key: Option<String>,
    #[serde(
        default,
        rename = "pinataApiSecret",
        skip_serializing_if = "Option::is_none"
    )]
    pub pinata_api_secret: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportResponse {
    pub status: u16,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct IpfsUploadResult {
    #[serde(rename = "appData")]
    pub app_data: String,
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
