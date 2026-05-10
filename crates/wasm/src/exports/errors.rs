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
use tsify::Tsify;
use wasm_bindgen::prelude::*;

/// JS-visible typed error envelope for every wasm export.
#[derive(Debug, Clone, Serialize, Deserialize, Tsify)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[non_exhaustive]
pub enum WasmError {
    /// Invalid user input.
    InvalidInput {
        /// Human-readable validation failure.
        message: String,
        /// Optional field name.
        #[serde(skip_serializing_if = "Option::is_none")]
        field: Option<String>,
    },
    /// Unknown string enum value.
    UnknownEnumValue {
        /// Field name.
        field: String,
        /// Rejected value.
        value: String,
    },
    /// Unsupported chain id.
    UnsupportedChain {
        /// Numeric chain id.
        chain_id: u32,
    },
    /// Wallet or signer callback failure.
    WalletRequest {
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
        /// Optional orderbook rejection code.
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
        /// Redacted message.
        message: String,
    },
    /// Subgraph failure.
    Subgraph {
        /// Redacted message.
        message: String,
    },
    /// Signing failure.
    Signing {
        /// Redacted message.
        message: String,
    },
    /// App-data failure.
    AppData {
        /// Optional class.
        #[serde(skip_serializing_if = "Option::is_none")]
        class: Option<String>,
        /// Redacted message.
        message: String,
    },
    /// Cooperative cancellation.
    Cancelled,
    /// Internal serialization or invariant failure.
    Internal {
        /// Human-readable message.
        message: String,
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
            field: Some(field.into()),
            message: message.into(),
        }
    }

    pub(crate) fn wallet(method: impl Into<String>, message: impl Into<String>) -> Self {
        Self::WalletRequest {
            method: method.into(),
            code: None,
            message: message.into(),
            data: None,
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
                field: Some(field),
                message,
            },
            PureError::UnknownEnumValue { field, value } => Self::UnknownEnumValue { field, value },
            PureError::UnsupportedChain { chain_id } => Self::UnsupportedChain { chain_id },
            error => Self::Internal {
                message: error.to_string(),
            },
        }
    }
}

impl From<TransportError> for WasmError {
    fn from(value: TransportError) -> Self {
        match value {
            TransportError::Transport { class, detail } => Self::Transport {
                class: class.to_string(),
                message: detail.to_string(),
                status: None,
                headers: None,
                body: None,
            },
            TransportError::Configuration { message } => Self::Transport {
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
                class: "status".to_owned(),
                message: format!("HTTP {status}"),
                status: Some(status),
                headers: Some(redact_header_pairs(headers)),
                body: Some(redact_response_body(body.as_inner())),
            },
            error => Self::Transport {
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
                class: Some(class.to_string()),
                message: detail.to_string(),
            },
            AppDataError::Cancelled => Self::Cancelled,
            error => Self::AppData {
                class: None,
                message: error.to_string(),
            },
        }
    }
}

impl From<SigningError> for WasmError {
    fn from(value: SigningError) -> Self {
        Self::Signing {
            message: value.to_string(),
        }
    }
}

impl From<OrderbookError> for WasmError {
    fn from(value: OrderbookError) -> Self {
        match value {
            OrderbookError::Transport { class, detail } => Self::Transport {
                class: class.to_string(),
                message: detail.to_string(),
                status: None,
                headers: None,
                body: None,
            },
            OrderbookError::Cancelled => Self::Cancelled,
            OrderbookError::Rejected {
                status, rejection, ..
            } => Self::Orderbook {
                code: Some(status.as_u16().to_string()),
                message: rejection.to_string(),
            },
            error => Self::Orderbook {
                code: None,
                message: error.to_string(),
            },
        }
    }
}

impl From<SubgraphError> for WasmError {
    fn from(value: SubgraphError) -> Self {
        match value {
            SubgraphError::Cancelled => Self::Cancelled,
            error => Self::Subgraph {
                message: error.to_string(),
            },
        }
    }
}

impl From<TradingError> for WasmError {
    fn from(value: TradingError) -> Self {
        match value {
            TradingError::Cancelled => Self::Cancelled,
            TradingError::Orderbook(error) => Self::from(error),
            TradingError::AppData(error) => Self::from(error),
            TradingError::Signing(error) => Self::from(error),
            error => Self::Orderbook {
                code: None,
                message: error.to_string(),
            },
        }
    }
}

impl From<Cancelled> for WasmError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}

impl From<serde_wasm_bindgen::Error> for WasmError {
    fn from(value: serde_wasm_bindgen::Error) -> Self {
        Self::Internal {
            message: value.to_string(),
        }
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
        Self::Signing {
            message: value.to_string(),
        }
    }
}

fn redact_header_pairs(headers: Vec<(String, Redacted<String>)>) -> Vec<[String; 2]> {
    headers
        .into_iter()
        .map(|(name, _)| [name, REDACTED_PLACEHOLDER.to_owned()])
        .collect()
}
