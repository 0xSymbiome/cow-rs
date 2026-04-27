use cow_sdk_core::{Cancelled, CoreError, HostPolicyError, TransportErrorClass, ValidationReason};
use http::StatusCode;
use thiserror::Error;

use crate::rejection::{OrderbookRejection, parse_rejection};
use crate::request::{OrderBookApiError, ResponseBody};
use crate::types::SigningScheme;

/// Errors returned by the typed orderbook client and transport helpers.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum OrderbookError {
    /// Error bubbled up from shared core configuration or type validation.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// Structured non-2xx response returned by the orderbook API whose body
    /// did not carry a recognisable rejection envelope.
    #[error(transparent)]
    Api(Box<OrderBookApiError>),
    /// Structured rejection classified from the non-2xx response body using
    /// the typed [`OrderbookRejection`] taxonomy.
    #[error("orderbook rejected the request ({status}): {rejection}")]
    Rejected {
        /// HTTP status code surfaced by the orderbook service.
        status: StatusCode,
        /// Typed rejection classification parsed from the response body.
        rejection: OrderbookRejection,
        /// Raw transport-level envelope preserved for diagnostics.
        #[source]
        source: Box<OrderBookApiError>,
    },
    /// Network or request-execution failure before a structured API response was decoded.
    #[error("transport error ({class}): {detail}")]
    Transport {
        /// Classification of the underlying REST-transport failure.
        class: TransportErrorClass,
        /// Redacted detail message sourced from the transport layer.
        detail: String,
    },
    /// Explicit service endpoint override failed host-policy validation.
    #[error(transparent)]
    HostPolicy(#[from] HostPolicyError),
    /// JSON or text decoding failure while parsing a successful or error response.
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    /// Invalid trades query assembled locally before any network request was sent.
    #[error("invalid trades query for field `{field}`: {reason}")]
    InvalidTradesQuery {
        /// Public field name that failed validation.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: ValidationReason,
    },
    /// Invalid quote request assembled locally before any network request was sent.
    #[error("invalid quote request for field `{field}`: {reason}")]
    InvalidQuoteRequest {
        /// Public field name that failed validation.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: ValidationReason,
    },
    /// Quote signing-scheme and on-chain-order flags are incompatible before transport.
    #[error(
        "incompatible signing scheme `{signing_scheme:?}` for on-chain order flag `{onchain_order}`"
    )]
    IncompatibleSigningScheme {
        /// Signing scheme supplied for the quote request.
        signing_scheme: SigningScheme,
        /// Whether the eventual order is expected to be on-chain.
        onchain_order: bool,
    },
    /// Invalid normalized orderbook data encountered after a successful HTTP response.
    #[error("invalid transform for field `{field}`: {reason}")]
    InvalidTransform {
        /// Public field name that failed validation.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: ValidationReason,
    },
    /// A long-running orderbook operation was cancelled through a cooperative cancellation token.
    #[error("orderbook operation was cancelled")]
    Cancelled,
}

impl From<OrderBookApiError> for OrderbookError {
    fn from(value: OrderBookApiError) -> Self {
        let status = StatusCode::from_u16(value.status).ok();
        let rejection = match (status, &value.body) {
            (Some(status_code), ResponseBody::Json(body)) => serde_json::to_vec(body)
                .ok()
                .and_then(|bytes| parse_rejection(status_code, &bytes)),
            _ => None,
        };

        match (status, rejection) {
            (Some(status), Some(rejection)) => Self::Rejected {
                status,
                rejection,
                source: Box::new(value),
            },
            _ => Self::Api(Box::new(value)),
        }
    }
}

impl From<Cancelled> for OrderbookError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}

impl From<reqwest::Error> for OrderbookError {
    fn from(error: reqwest::Error) -> Self {
        let (class, detail) = classify_reqwest_error(error);
        Self::Transport { class, detail }
    }
}

/// Classifies a `reqwest::Error`, strips any attached URL, and returns a typed
/// `(class, detail)` pair.
///
/// [`reqwest::Error::without_url`] is called before the
/// [`std::fmt::Display`] implementation runs so partner-route URLs and their
/// query parameters cannot leak through error text; the typed
/// [`TransportErrorClass`] captures the classification produced by the
/// documented `is_timeout`, `is_connect`, `is_redirect`, `is_decode`,
/// `is_body`, `is_builder`, `is_request`, and `is_status` partition.
#[must_use]
pub fn classify_reqwest_error(error: reqwest::Error) -> (TransportErrorClass, String) {
    let sanitized = error.without_url();
    let class = reqwest_error_class(&sanitized);
    (class, sanitized.to_string())
}

fn reqwest_error_class(error: &reqwest::Error) -> TransportErrorClass {
    if error.is_timeout() {
        return TransportErrorClass::Timeout;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if error.is_connect() {
            return TransportErrorClass::Connect;
        }
        if error.is_redirect() {
            return TransportErrorClass::Redirect;
        }
    }
    if error.is_decode() {
        TransportErrorClass::Decode
    } else if error.is_body() {
        TransportErrorClass::Body
    } else if error.is_builder() {
        TransportErrorClass::Builder
    } else if error.is_request() {
        TransportErrorClass::Request
    } else if error.is_status() {
        TransportErrorClass::Status
    } else {
        TransportErrorClass::Other
    }
}
