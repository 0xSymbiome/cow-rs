#[cfg(feature = "app-data")]
use cow_sdk_app_data::AppDataError;
#[cfg(any(feature = "orderbook", feature = "trading"))]
use cow_sdk_core::ErrorClass;
use cow_sdk_core::{
    Cancelled, REDACTED_PLACEHOLDER, Redacted, TransportError, redact_response_body,
};
#[cfg(feature = "orderbook")]
use cow_sdk_orderbook::{OrderbookError, OrderbookRejectionCategory};
use cow_sdk_pure_helpers::errors::PureError;
#[cfg(feature = "signing")]
use cow_sdk_signing::SigningError;
#[cfg(feature = "subgraph")]
use cow_sdk_subgraph::SubgraphError;
#[cfg(feature = "trading")]
use cow_sdk_trading::{AmountSide, ClientRejection, TradingError};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(any(feature = "orderbook", feature = "trading"))]
use std::time::Duration;
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
        /// Optional orderbook rejection code (the HTTP status when known).
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<String>,
        /// Coarse, switchable rejection category when the response carried a
        /// recognised rejection envelope.
        #[serde(skip_serializing_if = "Option::is_none")]
        category: Option<OrderBookRejectionCategoryDto>,
        /// Redacted message.
        message: String,
        /// Whether retrying the same request may succeed. The SDK retried
        /// internally and exhausted its budget, so `true` means the failure is
        /// transient (a rate limit or server-fault status) and a later retry
        /// under your own backoff may succeed; `false` means the request was
        /// rejected on its merits and resubmitting it unchanged will not.
        #[serde(default)]
        retryable: bool,
        /// Server-suggested wait before the next attempt, in milliseconds,
        /// parsed from the response `Retry-After` header when one was present.
        #[serde(skip_serializing_if = "Option::is_none")]
        retry_after_ms: Option<u32>,
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

/// Coarse, switchable classification of an orderbook rejection, mirrored for
/// the JS error surface.
///
/// A consumer can branch on the action a rejection calls for — fix the
/// request, fund the wallet, re-quote, wait, or escalate — without matching
/// every wire tag. The category carries no message or code, so it never
/// re-exposes redacted rejection text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum OrderBookRejectionCategoryDto {
    /// Refused on policy or permission grounds; not fixable by editing the order.
    Authorization,
    /// Sell-side balance or allowance is insufficient; fund or approve, then resubmit unchanged.
    InsufficientFunds,
    /// The request is malformed or violates a validation rule; fix the parameters and rebuild.
    InvalidOrder,
    /// The referenced quote or order does not exist.
    NotFound,
    /// The order's lifecycle state conflicts with the request and it cannot be retried as is.
    Conflict,
    /// No solver, route, liquidity, or fee economics can currently fill the trade as sized; the condition may clear later — re-quote, wait, or resize.
    Unfulfillable,
    /// An upstream server-side fault.
    Server,
    /// A wire tag the SDK does not yet model, preserved for forward compatibility.
    #[serde(rename = "__unknown")]
    Unknown,
}

#[cfg(feature = "orderbook")]
impl From<OrderbookRejectionCategory> for OrderBookRejectionCategoryDto {
    fn from(value: OrderbookRejectionCategory) -> Self {
        match value {
            OrderbookRejectionCategory::Authorization => Self::Authorization,
            OrderbookRejectionCategory::InsufficientFunds => Self::InsufficientFunds,
            OrderbookRejectionCategory::InvalidOrder => Self::InvalidOrder,
            OrderbookRejectionCategory::NotFound => Self::NotFound,
            OrderbookRejectionCategory::Conflict => Self::Conflict,
            OrderbookRejectionCategory::Unfulfillable => Self::Unfulfillable,
            OrderbookRejectionCategory::Server => Self::Server,
            // The native `Unknown` category and any future `#[non_exhaustive]`
            // category both surface as the forward-compatible unknown sentinel.
            _ => Self::Unknown,
        }
    }
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

    /// Builds a wallet error from an EIP-1193 / JSON-RPC provider error `code`.
    ///
    /// Per the redaction policy (ADR 0053), the provider-authored error
    /// `message` and `data` payload can echo caller secrets or RPC endpoint
    /// tokens, so neither crosses the boundary. The structured `code` is the
    /// safe, machine-actionable signal, and the human message is SDK-authored
    /// guidance keyed off the standard provider code (for example the `-32601`
    /// method-not-found hint).
    pub(crate) fn wallet_from_code(method: impl Into<String>, code: Option<i64>) -> Self {
        let method = method.into();
        let message = wallet_request_message(&method, wallet_code_hint(code).to_owned());
        Self::WalletRequest {
            schema_version: SchemaVersion::V1,
            method,
            code,
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
        // Resolve the retry verdict and backoff hint once, before the match
        // consumes the error, so both `Orderbook` construction sites carry them.
        let retryable = value.is_retryable();
        let retry_after_ms = backoff_hint_ms(value.backoff_hint());
        match value {
            OrderbookError::Transport { class, detail } => Self::Transport {
                schema_version: SchemaVersion::V1,
                class: class.to_string(),
                message: transport_message(&class.to_string(), detail.to_string()),
                status: None,
                headers: None,
                body: None,
            },
            OrderbookError::Cancelled => Self::cancelled(),
            OrderbookError::Rejected {
                status, rejection, ..
            } => Self::Orderbook {
                schema_version: SchemaVersion::V1,
                code: Some(status.as_u16().to_string()),
                category: Some(rejection.category().into()),
                message: orderbook_rejection_message(status.as_u16(), rejection.to_string()),
                retryable,
                retry_after_ms,
            },
            // A content-addressed hash mismatch is a bad-request / caller-input
            // fault (the orderbook service rejects it with HTTP 400), so it
            // surfaces as `invalidInput` even though the native `class()` is
            // the stricter `Internal`.
            error @ OrderbookError::AppDataHashMismatch { .. } => {
                Self::invalid("appData.appDataHash", error.to_string())
            }
            OrderbookError::InvalidTradesQuery { field, reason }
            | OrderbookError::InvalidQuoteRequest { field, reason } => {
                Self::invalid(field, reason.to_string())
            }
            // Base `kind` follows the shared `ErrorClass`; the structured
            // orderbook `code` carries the HTTP status (including a 429
            // throttle) when the variant knows it.
            error => match error.class() {
                ErrorClass::Validation => Self::invalid("input", error.to_string()),
                ErrorClass::Signing => Self::Signing {
                    schema_version: SchemaVersion::V1,
                    message: signing_message(error.to_string()),
                },
                ErrorClass::Transport => Self::Transport {
                    schema_version: SchemaVersion::V1,
                    class: "other".to_owned(),
                    message: transport_message("other", error.to_string()),
                    status: None,
                    headers: None,
                    body: None,
                },
                ErrorClass::Cancelled => Self::cancelled(),
                ErrorClass::Internal => Self::internal(error.to_string()),
                // Remote and rate-limited faults (a 429 throttle keeps its
                // status in `code` rather than gaining a distinct kind), plus
                // future additive classes, surface as the orderbook kind.
                _ => Self::Orderbook {
                    schema_version: SchemaVersion::V1,
                    code: orderbook_status_code(&error),
                    category: None,
                    message: orderbook_message(error.to_string()),
                    retryable,
                    retry_after_ms,
                },
            },
        }
    }
}

/// Converts a parsed `Retry-After` backoff to whole milliseconds for the JS
/// surface, dropping a value that exceeds the `u32` millisecond range.
#[cfg(any(feature = "orderbook", feature = "trading"))]
fn backoff_hint_ms(backoff: Option<Duration>) -> Option<u32> {
    backoff.and_then(|delay| u32::try_from(delay.as_millis()).ok())
}

/// Lifts the HTTP status from an orderbook API error into a structured `code`,
/// keeping the redacted body and headers off the JS surface.
#[cfg(feature = "orderbook")]
fn orderbook_status_code(error: &OrderbookError) -> Option<String> {
    match error {
        OrderbookError::Api(api) => Some(api.status.to_string()),
        _ => None,
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
            TradingError::Cancelled => Self::cancelled(),
            TradingError::Orderbook(error) => Self::from(error),
            TradingError::AppData(error) => Self::from(error),
            TradingError::Signing(error) => Self::from(error),
            // The client-side bounds validator (which every managed submission
            // path runs) rejects locally with a typed `ClientRejection`;
            // surface it as `invalidInput` with the offending field rather than
            // the orderbook catch-all.
            TradingError::ClientRejected(rejection) => Self::from(rejection),
            // Base `kind` follows the shared `ErrorClass`: caller-side
            // validation faults are `invalidInput`, signer/provider faults are
            // `signing`, invariant failures are `internal`, and remote faults
            // fold into the orderbook kind.
            error => match error.class() {
                ErrorClass::Validation => Self::invalid("input", error.to_string()),
                ErrorClass::Signing => Self::Signing {
                    schema_version: SchemaVersion::V1,
                    message: signing_message(error.to_string()),
                },
                ErrorClass::Transport => Self::Transport {
                    schema_version: SchemaVersion::V1,
                    class: "other".to_owned(),
                    message: transport_message("other", error.to_string()),
                    status: None,
                    headers: None,
                    body: None,
                },
                ErrorClass::Cancelled => Self::cancelled(),
                ErrorClass::Internal => Self::internal(error.to_string()),
                // Remote, RateLimited, and future additive classes → orderbook.
                _ => Self::Orderbook {
                    schema_version: SchemaVersion::V1,
                    code: None,
                    category: None,
                    message: orderbook_message(error.to_string()),
                    retryable: error.is_retryable(),
                    retry_after_ms: backoff_hint_ms(error.backoff_hint()),
                },
            },
        }
    }
}

/// Maps a typed client-side [`ClientRejection`] to a JS-visible `invalidInput`
/// error, tagging the offending field so consumers can guide the caller. The
/// native `class()` is already `Validation`; this adds the structured `field`.
#[cfg(feature = "trading")]
impl From<ClientRejection> for WasmError {
    fn from(value: ClientRejection) -> Self {
        Self::invalid(client_rejection_field(&value), value.to_string())
    }
}

/// Resolves the public field name a [`ClientRejection`] should point the caller
/// at. Returns the order-level sentinel for forward-compatible variants the
/// crate does not yet map.
#[cfg(feature = "trading")]
const fn client_rejection_field(rejection: &ClientRejection) -> &'static str {
    match rejection {
        ClientRejection::ValidToInPast { .. } => "validTo",
        ClientRejection::MissingFrom | ClientRejection::OwnerMismatch { .. } => "from",
        ClientRejection::AppdataFromMismatch { .. } => "appData.signer",
        ClientRejection::SameBuyAndSellToken { .. } => "buyToken",
        ClientRejection::InvalidNativeSellToken => "sellToken",
        ClientRejection::ZeroAmount { side } => match side {
            AmountSide::Sell => "sellAmount",
            AmountSide::Buy => "buyAmount",
            // `AmountSide` is `#[non_exhaustive]`.
            _ => "amount",
        },
        // The specific partner-fee sub-field is carried in the rendered
        // message; the structured tag stays at the coarse top-level field.
        ClientRejection::InvalidPartnerFee { .. } => "partnerFee",
        // `ClientRejection` is `#[non_exhaustive]`; a future client-side
        // rejection still surfaces as `invalidInput`, defaulting to the
        // order-level field until mapped to a specific one here.
        _ => "order",
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

/// SDK-authored guidance for a standard EIP-1193 / JSON-RPC provider error
/// code, used in place of the redacted provider message.
const fn wallet_code_hint(code: Option<i64>) -> &'static str {
    match code {
        // EIP-1193 user rejection (4001).
        Some(4001) => "the user rejected the request",
        // EIP-1193 unauthorized (4100).
        Some(4100) => "the requested account or method is not authorized by the wallet",
        // EIP-1193 unsupported method (4200) or JSON-RPC method-not-found (-32601).
        Some(4200 | -32601) => "the wallet does not support the requested method",
        // EIP-1193 disconnected (4900) or chain-disconnected (4901).
        Some(4900 | 4901) => "the wallet is disconnected from the requested chain",
        _ => "the wallet could not complete the request",
    }
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
