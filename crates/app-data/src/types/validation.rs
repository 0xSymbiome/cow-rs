use serde::{Deserialize, Serialize};

/// Derived identifiers for a validated app-data document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
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
