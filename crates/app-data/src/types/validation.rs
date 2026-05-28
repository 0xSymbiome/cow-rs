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
///
/// On failure, [`ValidationResult::errors`] carries a path-prefixed
/// validator message that is safe-by-construction: instance values are
/// masked through the underlying validator's masking surface and
/// rejected-property-name lists are rendered as counts rather than names,
/// so the rendered text can be logged or surfaced to end users without
/// crossing the redaction boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ValidationResult {
    /// Whether validation succeeded.
    pub success: bool,
    /// Rendered validation errors when `success` is `false`. Plaintext-safe
    /// by construction; see the struct-level documentation for the masking
    /// contract.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
}

impl ValidationResult {
    /// Creates a schema validation result.
    #[must_use]
    pub const fn new(success: bool, errors: Option<String>) -> Self {
        Self { success, errors }
    }
}
