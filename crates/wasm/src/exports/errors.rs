#[cfg(feature = "app-data")]
use cow_sdk_app_data::AppDataError;
use cow_sdk_core::{
    Cancelled, REDACTED_PLACEHOLDER, Redacted, TransportError, redact_response_body,
};
#[cfg(feature = "orderbook")]
use cow_sdk_orderbook::OrderbookError;
use cow_sdk_pure_helpers::errors::PureError;
#[cfg(feature = "signing")]
use cow_sdk_signing::SigningError;
#[cfg(feature = "subgraph")]
use cow_sdk_subgraph::SubgraphError;
#[cfg(feature = "trading")]
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
        /// Human-readable recovery guidance.
        message: String,
        /// Field name.
        field: String,
        /// Rejected value.
        value: String,
    },
    /// Unsupported chain id.
    UnsupportedChain {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Human-readable recovery guidance.
        message: String,
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
    /// Wallet callback timeout.
    WalletTimeout {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Human-readable recovery guidance.
        message: String,
        /// Timeout in milliseconds.
        timeout_ms: u32,
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
        /// Human-readable recovery guidance.
        message: String,
        /// Rejected target address.
        target: String,
        /// Human-readable reason.
        reason: String,
    },
    /// Cooperative cancellation.
    Cancelled {
        /// Error schema version.
        schema_version: SchemaVersion,
        /// Human-readable recovery guidance.
        message: String,
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
        /// Human-readable recovery guidance.
        message: String,
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
        let field = field.into();
        let message = invalid_input_message(&field, message.into());
        Self::InvalidInput {
            schema_version: SchemaVersion::V1,
            field: Some(field),
            message,
        }
    }

    pub(crate) fn wallet(method: impl Into<String>, message: impl Into<String>) -> Self {
        let method = method.into();
        let message = wallet_request_message(&method, message.into());
        Self::WalletRequest {
            schema_version: SchemaVersion::V1,
            method,
            code: None,
            message,
            data: None,
        }
    }

    pub(crate) fn wallet_timeout(timeout_ms: u32) -> Self {
        Self::WalletTimeout {
            schema_version: SchemaVersion::V1,
            message: wallet_timeout_message(timeout_ms),
            timeout_ms,
        }
    }

    pub(crate) fn cancelled() -> Self {
        Self::Cancelled {
            schema_version: SchemaVersion::V1,
            message: cancelled_message(),
        }
    }

    pub(crate) fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            schema_version: SchemaVersion::V1,
            message: internal_message(message.into()),
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
                message: invalid_input_message(&field, message),
                field: Some(field),
            },
            PureError::UnknownEnumValue { field, value } => Self::UnknownEnumValue {
                schema_version: SchemaVersion::V1,
                message: unknown_enum_message(&field, &value),
                field,
                value,
            },
            PureError::UnsupportedChain { chain_id } => Self::UnsupportedChain {
                schema_version: SchemaVersion::V1,
                message: unsupported_chain_message(chain_id),
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
                message: transport_message(&class.to_string(), detail.to_string()),
                status: None,
                headers: None,
                body: None,
            },
            TransportError::Configuration { message } => Self::Transport {
                schema_version: SchemaVersion::V1,
                class: "builder".to_owned(),
                message: transport_message("builder", message.to_string()),
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
                message: http_status_message(status),
                status: Some(status),
                headers: Some(redact_header_pairs(headers)),
                body: Some(redact_response_body(body.as_inner())),
            },
            error => Self::Transport {
                schema_version: SchemaVersion::V1,
                class: "other".to_owned(),
                message: transport_message("other", error.to_string()),
                status: None,
                headers: None,
                body: None,
            },
        }
    }
}

#[cfg(feature = "app-data")]
impl From<AppDataError> for WasmError {
    fn from(value: AppDataError) -> Self {
        match value {
            AppDataError::Transport { class, detail } => Self::AppData {
                schema_version: SchemaVersion::V1,
                class: Some(class.to_string()),
                message: app_data_message(Some(&class.to_string()), detail.to_string()),
            },
            AppDataError::Cancelled => Self::Cancelled {
                schema_version: SchemaVersion::V1,
                message: cancelled_message(),
            },
            error => Self::AppData {
                schema_version: SchemaVersion::V1,
                class: None,
                message: app_data_message(None, error.to_string()),
            },
        }
    }
}

#[cfg(feature = "signing")]
impl From<SigningError> for WasmError {
    fn from(value: SigningError) -> Self {
        Self::Signing {
            schema_version: SchemaVersion::V1,
            message: signing_message(value.to_string()),
        }
    }
}

#[cfg(feature = "orderbook")]
impl From<OrderbookError> for WasmError {
    fn from(value: OrderbookError) -> Self {
        match value {
            OrderbookError::Transport { class, detail } => Self::Transport {
                schema_version: SchemaVersion::V1,
                class: class.to_string(),
                message: transport_message(&class.to_string(), detail.to_string()),
                status: None,
                headers: None,
                body: None,
            },
            OrderbookError::Cancelled => Self::Cancelled {
                schema_version: SchemaVersion::V1,
                message: cancelled_message(),
            },
            OrderbookError::Rejected {
                status, rejection, ..
            } => Self::Orderbook {
                schema_version: SchemaVersion::V1,
                code: Some(status.as_u16().to_string()),
                message: orderbook_rejection_message(status.as_u16(), rejection.to_string()),
            },
            error => Self::Orderbook {
                schema_version: SchemaVersion::V1,
                code: None,
                message: orderbook_message(error.to_string()),
            },
        }
    }
}

#[cfg(feature = "subgraph")]
impl From<SubgraphError> for WasmError {
    fn from(value: SubgraphError) -> Self {
        match value {
            SubgraphError::Cancelled => Self::Cancelled {
                schema_version: SchemaVersion::V1,
                message: cancelled_message(),
            },
            error => Self::Subgraph {
                schema_version: SchemaVersion::V1,
                message: subgraph_message(error.to_string()),
            },
        }
    }
}

#[cfg(feature = "trading")]
impl From<TradingError> for WasmError {
    fn from(value: TradingError) -> Self {
        match value {
            TradingError::Cancelled => Self::Cancelled {
                schema_version: SchemaVersion::V1,
                message: cancelled_message(),
            },
            TradingError::Orderbook(error) => Self::from(error),
            TradingError::AppData(error) => Self::from(error),
            TradingError::Signing(error) => Self::from(error),
            error => Self::Orderbook {
                schema_version: SchemaVersion::V1,
                code: None,
                message: orderbook_message(error.to_string()),
            },
        }
    }
}

impl From<Cancelled> for WasmError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled {
            schema_version: SchemaVersion::V1,
            message: cancelled_message(),
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

#[cfg(feature = "signing")]
impl From<cow_sdk_contracts::ContractsError> for WasmError {
    fn from(value: cow_sdk_contracts::ContractsError) -> Self {
        match value {
            cow_sdk_contracts::ContractsError::ForbiddenInteractionTarget { target } => {
                Self::ForbiddenInteraction {
                    schema_version: SchemaVersion::V1,
                    message: forbidden_interaction_message(&target.to_hex_string()),
                    target: target.to_hex_string(),
                    reason: "forbidden settlement interaction target".to_owned(),
                }
            }
            error => Self::Signing {
                schema_version: SchemaVersion::V1,
                message: signing_message(error.to_string()),
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

fn invalid_input_message(field: &str, detail: String) -> String {
    format!(
        "Invalid `{field}`: {detail}. Check the value supplied for `{field}` and retry with a valid SDK input."
    )
}

fn unknown_enum_message(field: &str, value: &str) -> String {
    format!(
        "Unsupported value `{value}` for `{field}`. Use one of the documented values for this field."
    )
}

fn unsupported_chain_message(chain_id: u32) -> String {
    format!(
        "Unsupported chain ID {chain_id}. Call supportedChainIds() before constructing requests and route unsupported networks to another integration."
    )
}

fn wallet_request_message(method: &str, detail: String) -> String {
    format!(
        "Wallet request `{method}` failed: {detail}. Verify the wallet is connected, on the requested chain, and allowed to sign this request."
    )
}

fn wallet_timeout_message(timeout_ms: u32) -> String {
    format!(
        "Wallet request timed out after {timeout_ms} ms. Increase walletConfig.timeoutMs or ask the user to approve the wallet request before the timeout."
    )
}

fn transport_message(class: &str, detail: String) -> String {
    format!(
        "HTTP transport `{class}` failed: {detail}. Check network reachability, request URL, timeout, and callback response shape."
    )
}

fn http_status_message(status: u16) -> String {
    format!(
        "Orderbook transport returned HTTP {status}. Check the request payload, chain/environment, API key, and redacted response body."
    )
}

#[cfg(feature = "orderbook")]
fn orderbook_rejection_message(status: u16, detail: String) -> String {
    format!(
        "Orderbook rejected the request with HTTP {status}: {detail}. Verify balances, allowances, quote validity, signature, and order parameters before retrying."
    )
}

#[cfg(feature = "orderbook")]
fn orderbook_message(detail: String) -> String {
    format!(
        "Orderbook operation failed: {detail}. Verify the request payload, chain/environment, transport configuration, and order state."
    )
}

#[cfg(feature = "subgraph")]
fn subgraph_message(detail: String) -> String {
    format!(
        "Subgraph request failed: {detail}. Check chain support, query shape, API key, and endpoint availability."
    )
}

#[cfg(feature = "signing")]
fn signing_message(detail: String) -> String {
    format!(
        "Signing operation failed: {detail}. Verify the order fields, chain ID, owner address, and signature callback output."
    )
}

#[cfg(feature = "app-data")]
fn app_data_message(class: Option<&str>, detail: String) -> String {
    match class {
        Some(class) => format!(
            "App-data `{class}` operation failed: {detail}. Verify the app-data document, CID/hash, transport callback, and IPFS endpoint."
        ),
        None => format!(
            "App-data operation failed: {detail}. Verify the app-data document, CID/hash, and schema version."
        ),
    }
}

fn forbidden_interaction_message(target: &str) -> String {
    format!(
        "Forbidden settlement interaction target `{target}`. Remove this target from settlement interactions before signing or submitting the order."
    )
}

fn cancelled_message() -> String {
    "Operation was cancelled. Create a fresh AbortController or retry without an already-aborted signal."
        .to_owned()
}

fn internal_message(detail: String) -> String {
    format!(
        "SDK internal error: {detail}. This indicates serialization or invariant failure; retry with the same inputs only after checking the reported input shape."
    )
}
