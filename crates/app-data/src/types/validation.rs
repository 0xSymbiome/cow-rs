use cow_sdk_core::Redacted;
use serde::{Deserialize, Serialize};

/// Derived identifiers for a validated app-data document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
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

impl AppDataInfo {
    /// Creates derived identifiers for a validated app-data document.
    #[must_use]
    pub fn new(
        cid: impl Into<String>,
        app_data_content: impl Into<String>,
        app_data_hex: impl Into<String>,
    ) -> Self {
        Self {
            cid: cid.into(),
            app_data_content: app_data_content.into(),
            app_data_hex: app_data_hex.into(),
        }
    }
}

/// Schema validation result returned by [`crate::validate_app_data_doc`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ValidationResult {
    /// Whether validation succeeded.
    pub success: bool,
    /// Rendered validation errors when `success` is `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<Redacted<String>>,
}

impl ValidationResult {
    /// Creates a schema validation result.
    #[must_use]
    pub fn new(success: bool, errors: Option<String>) -> Self {
        Self {
            success,
            errors: errors.map(Redacted::new),
        }
    }
}
