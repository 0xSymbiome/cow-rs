//! App-data boundary shapes for the TypeScript-callable surface.

use cow_sdk_app_data::AppDataError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// App-data document input.
///
/// An app-data-flavour boundary type (the TypeScript declaration derive scopes it
/// to the wasm flavours that surface app-data). The app-data document lowering
/// that consumes it lives in the leaf's host-safe `helpers`, so this type carries
/// only the structural shape. The shape is always defined so the host-side
/// `helpers` can build it; only the TypeScript declaration derive is scoped to the
/// wasm-bindgen target and the app-data feature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "app-data"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "app-data"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `metadata` field is a serde_json::Value, which is not Eq, so the struct cannot derive Eq"
)]
pub struct AppDataParams {
    /// Application code.
    pub app_code: String,
    /// Metadata object.
    pub metadata: Value,
    /// Schema version.
    pub version: String,
    /// Optional environment label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub environment: Option<String>,
}

/// App-data validation result.
///
/// An app-data-flavour boundary projection of the typed
/// `Result<(), AppDataError>` the SDK validator returns: `{success, errors}`,
/// where the rendered error text names only the offending public field and
/// never the caller-supplied value. The TypeScript declaration derive scopes it
/// to the wasm flavours that surface app-data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "app-data"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "app-data"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    /// Whether validation succeeded.
    pub success: bool,
    /// Errors when validation failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
}

impl From<Result<(), AppDataError>> for ValidationResult {
    /// Projects the validator's typed result into the boundary `{success,
    /// errors}` shape, rendering the typed error to its redacted display string.
    fn from(value: Result<(), AppDataError>) -> Self {
        match value {
            Ok(()) => Self {
                success: true,
                errors: None,
            },
            Err(error) => Self {
                success: false,
                errors: Some(error.to_string()),
            },
        }
    }
}

/// App-data document output.
#[cfg(feature = "app-data")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "app-data"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "app-data"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `document` field is a serde_json::Value, which is not Eq, so the struct cannot derive Eq"
)]
pub struct AppDataDocument {
    /// App-data document.
    pub document: Value,
}

#[cfg(feature = "app-data")]
impl From<Value> for AppDataDocument {
    fn from(value: Value) -> Self {
        Self { document: value }
    }
}
