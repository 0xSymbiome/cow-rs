use cow_sdk_app_data::AppDataError;
use cow_sdk_core::{
    Cancelled, REDACTED_PLACEHOLDER, Redacted, TransportError, redact_response_body,
};
use cow_sdk_orderbook::OrderbookError;
use cow_sdk_pure_helpers::errors::PureError;
use cow_sdk_signing::SigningError;
use cow_sdk_subgraph::SubgraphError;
use cow_sdk_trading::TradingError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use crate::exports::envelope::SchemaVersion;

/// JS-visible typed error envelope for every wasm export.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[non_exhaustive]
pub enum WasmError {
    /// Invalid user input.
    InvalidInput {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Human-readable validation failure.
        message: String,
        /// Optional field name.
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<String>,
    },
    /// Unknown string enum value.
    UnknownEnumValue {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Field name.
        field: String,
        /// Rejected value.
        value: String,
    },
    /// Unsupported chain id.
    UnsupportedChain {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Numeric chain id.
        chain_id: u32,
    },
    /// Wallet or signer callback failure.
    WalletRequest {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Request method or callback name.
        method: String,
        /// Optional provider error code.
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<i64>,
        /// Redacted or callback-provided message.
        message: String,
        /// Optional provider data.
        #[serde(skip_serializing_if = "Option::is_none")]
        data: Option<serde_json::Value>,
    },
    /// HTTP transport failure.
    Transport {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Transport class.
        class: String,
        /// Redacted message.
        message: String,
        /// HTTP status when available.
        #[serde(skip_serializing_if = "Option::is_none")]
        status: Option<u16>,
        /// Response headers when available.
        #[serde(skip_serializing_if = "Option::is_none")]
        headers: Option<Vec<[String; 2]>>,
        /// Redacted response body when available.
        #[serde(skip_serializing_if = "Option::is_none")]
        body: Option<String>,
    },
    /// Orderbook failure.
    Orderbook {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Optional orderbook rejection code.
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
        /// Redacted message.
        message: String,
    },
    /// Subgraph failure.
    Subgraph {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Redacted message.
        message: String,
    },
    /// Signing failure.
    Signing {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Redacted message.
        message: String,
    },
    /// App-data failure.
    AppData {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Optional class.
        #[serde(skip_serializing_if = "Option::is_none")]
        class: Option<String>,
        /// Redacted message.
        message: String,
    },
    /// Forbidden contract interaction target.
    ForbiddenInteraction {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Rejected target address.
        target: String,
        /// Human-readable reason.
        reason: String,
    },
    /// Cooperative cancellation.
    Cancelled {
        /// Error schema version.
        schema_version: SchemaVersion,
    },
    /// Internal serialization or invariant failure.
    Internal {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Human-readable message.
        message: String,
    },
    /// Forward-compatible sentinel for errors unknown to this crate.
    #[serde(rename = "__unknown")]
    Unknown {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Raw unrecognized error value.
        raw: Value,
    },
}

impl WasmError {
    /// Converts this typed error into a `JsValue` without panicking.
    #[must_use]
    pub fn into_js(self) -> JsValue {
        let serializer = serde_wasm_bindgen::Serializer::json_compatible();
        self.serialize(&serializer)
            .unwrap_or_else(|_| JsValue::from_str("WasmError serialization failed"))
    }

    pub(crate) fn invalid(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidInput {
            schema_version: SchemaVersion::V1,
            field: Some(field.into()),
            message: message.into(),
        }
    }

    pub(crate) fn wallet(method: impl Into<String>, message: impl Into<String>) -> Self {
        Self::WalletRequest {
            schema_version: SchemaVersion::V1,
            method: method.into(),
            code: None,
            message: message.into(),
            data: None,
        }
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            schema_version: SchemaVersion::V1,
            message: message.into(),
        }
    }
}

impl From<WasmError> for JsValue {
    fn from(value: WasmError) -> Self {
        value.into_js()
    }
}

impl From<PureError> for WasmError {
    fn from(value: PureError) -> Self {
        match value {
            PureError::InvalidInput { field, message } => Self::InvalidInput {
                schema_version: SchemaVersion::V1,
                field: Some(field),
                message,
            },
            PureError::UnknownEnumValue { field, value } => Self::UnknownEnumValue {
                schema_version: SchemaVersion::V1,
                field,
                value,
            },
            PureError::UnsupportedChain { chain_id } => Self::UnsupportedChain {
                schema_version: SchemaVersion::V1,
                chain_id,
            },
            error => Self::internal(error.to_string()),
        }
    }
}

impl From<TransportError> for WasmError {
    fn from(value: TransportError) -> Self {
        match value {
            TransportError::Transport { class, detail } => Self::Transport {
                schema_version: SchemaVersion::V1,
                class: class.to_string(),
                message: detail.to_string(),
                status: None,
                headers: None,
                body: None,
            },
            TransportError::Configuration { message } => Self::Transport {
                schema_version: SchemaVersion::V1,
                class: "builder".to_owned(),
                message: message.to_string(),
                status: None,
                headers: None,
                body: None,
            },
            TransportError::HttpStatus {
                status,
                headers,
                body,
            } => Self::Transport {
                schema_version: SchemaVersion::V1,
                class: "status".to_owned(),
                message: format!("HTTP {status}"),
                status: Some(status),
                headers: Some(redact_header_pairs(headers)),
                body: Some(redact_response_body(body.as_inner())),
            },
            error => Self::Transport {
                schema_version: SchemaVersion::V1,
                class: "other".to_owned(),
                message: error.to_string(),
                status: None,
                headers: None,
                body: None,
            },
        }
    }
}

impl From<AppDataError> for WasmError {
    fn from(value: AppDataError) -> Self {
        match value {
            AppDataError::Transport { class, detail } => Self::AppData {
                schema_version: SchemaVersion::V1,
                class: Some(class.to_string()),
                message: detail.to_string(),
            },
            AppDataError::Cancelled => Self::Cancelled {
                schema_version: SchemaVersion::V1,
            },
            error => Self::AppData {
                schema_version: SchemaVersion::V1,
                class: None,
                message: error.to_string(),
            },
        }
    }
}

impl From<SigningError> for WasmError {
    fn from(value: SigningError) -> Self {
        Self::Signing {
            schema_version: SchemaVersion::V1,
            message: value.to_string(),
        }
    }
}

impl From<OrderbookError> for WasmError {
    fn from(value: OrderbookError) -> Self {
        match value {
            OrderbookError::Transport { class, detail } => Self::Transport {
                schema_version: SchemaVersion::V1,
                class: class.to_string(),
                message: detail.to_string(),
                status: None,
                headers: None,
                body: None,
            },
            OrderbookError::Cancelled => Self::Cancelled {
                schema_version: SchemaVersion::V1,
            },
            OrderbookError::Rejected {
                status, rejection, ..
            } => Self::Orderbook {
                schema_version: SchemaVersion::V1,
                code: Some(status.as_u16().to_string()),
                message: rejection.to_string(),
            },
            error => Self::Orderbook {
                schema_version: SchemaVersion::V1,
                code: None,
                message: error.to_string(),
            },
        }
    }
}

impl From<SubgraphError> for WasmError {
    fn from(value: SubgraphError) -> Self {
        match value {
            SubgraphError::Cancelled => Self::Cancelled {
                schema_version: SchemaVersion::V1,
            },
            error => Self::Subgraph {
                schema_version: SchemaVersion::V1,
                message: error.to_string(),
            },
        }
    }
}

impl From<TradingError> for WasmError {
    fn from(value: TradingError) -> Self {
        match value {
            TradingError::Cancelled => Self::Cancelled {
                schema_version: SchemaVersion::V1,
            },
            TradingError::Orderbook(error) => Self::from(error),
            TradingError::AppData(error) => Self::from(error),
            TradingError::Signing(error) => Self::from(error),
            error => Self::Orderbook {
                schema_version: SchemaVersion::V1,
                code: None,
                message: error.to_string(),
            },
        }
    }
}

impl From<Cancelled> for WasmError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled {
            schema_version: SchemaVersion::V1,
        }
    }
}

impl From<serde_wasm_bindgen::Error> for WasmError {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self::internal(value.to_string())
    }
}

impl From<serde_json::Error> for WasmError {
    fn from(value: serde_json::Error) -> Self {
        Self::invalid("json", value.to_string())
    }
}

impl From<cow_sdk_core::CoreError> for WasmError {
    fn from(value: cow_sdk_core::CoreError) -> Self {
        Self::invalid("input", value.to_string())
    }
}

impl From<cow_sdk_contracts::ContractsError> for WasmError {
    fn from(value: cow_sdk_contracts::ContractsError) -> Self {
        match value {
            cow_sdk_contracts::ContractsError::ForbiddenInteractionTarget { target } => {
                Self::ForbiddenInteraction {
                    schema_version: SchemaVersion::V1,
                    target: target.as_str().to_owned(),
                    reason: "forbidden settlement interaction target".to_owned(),
                }
            }
            error => Self::Signing {
                schema_version: SchemaVersion::V1,
                message: error.to_string(),
            },
        }
    }
}

fn redact_header_pairs(headers: Vec<(String, Redacted<String>)>) -> Vec<[String; 2]> {
    headers
        .into_iter()
        .map(|(name, _)| [name, REDACTED_PLACEHOLDER.to_owned()])
        .collect()
}
