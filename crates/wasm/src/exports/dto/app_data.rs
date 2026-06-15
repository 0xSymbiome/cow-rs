#[cfg(feature = "app-data")]
use crate::helpers as pure;
#[cfg(feature = "app-data")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "app-data")]
use serde_json::Value;
#[cfg(feature = "app-data")]
use tsify::Tsify;
#[cfg(feature = "app-data")]
use wasm_bindgen::prelude::*;

/// App-data document input.
#[cfg(feature = "app-data")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AppDataDocInput {
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

#[cfg(feature = "app-data")]
impl From<AppDataDocInput> for pure::dto::AppDataDocInput {
    fn from(value: AppDataDocInput) -> Self {
        Self {
            app_code: value.app_code,
            metadata: value.metadata,
            version: value.version,
            environment: value.environment,
        }
    }
}

/// App-data document output.
#[cfg(feature = "app-data")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AppDataDocDto {
    /// App-data document.
    pub document: Value,
}

#[cfg(feature = "app-data")]
impl From<Value> for AppDataDocDto {
    fn from(value: Value) -> Self {
        Self { document: value }
    }
}

/// App-data info output.
#[cfg(feature = "app-data")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AppDataInfoDto {
    /// CID representation.
    pub cid: String,
    /// Deterministic app-data content.
    pub app_data_content: String,
    /// App-data hash.
    pub app_data_hex: String,
}

#[cfg(feature = "app-data")]
impl From<pure::dto::AppDataInfoDto> for AppDataInfoDto {
    fn from(value: pure::dto::AppDataInfoDto) -> Self {
        Self {
            cid: value.cid,
            app_data_content: value.app_data_content,
            app_data_hex: value.app_data_hex,
        }
    }
}

/// App-data validation result.
#[cfg(feature = "app-data")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResultDto {
    /// Whether validation succeeded.
    pub success: bool,
    /// Errors when validation failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub errors: Option<String>,
}

#[cfg(feature = "app-data")]
impl From<pure::dto::ValidationResultDto> for ValidationResultDto {
    fn from(value: pure::dto::ValidationResultDto) -> Self {
        Self {
            success: value.success,
            errors: value.errors,
        }
    }
}
