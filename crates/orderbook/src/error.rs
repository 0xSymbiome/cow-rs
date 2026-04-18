use cow_sdk_core::{Cancelled, CoreError};
use thiserror::Error;

use crate::request::OrderBookApiError;

/// Errors returned by the typed orderbook client and transport helpers.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum OrderbookError {
    /// Error bubbled up from shared core configuration or type validation.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// Structured non-2xx response returned by the orderbook API.
    #[error(transparent)]
    Api(Box<OrderBookApiError>),
    /// Network or request-execution failure before a structured API response was decoded.
    #[error("transport error: {0}")]
    Transport(String),
    /// JSON or text decoding failure while parsing a successful or error response.
    #[error("serialization error: {0}")]
    Serialization(String),
    /// Invalid trades query assembled locally before any network request was sent.
    #[error("invalid trades query: {0}")]
    InvalidTradesQuery(String),
    /// Invalid quote request assembled locally before any network request was sent.
    #[error("invalid quote request: {0}")]
    InvalidQuoteRequest(String),
    /// Invalid normalized orderbook data encountered after a successful HTTP response.
    #[error("invalid transform: {0}")]
    InvalidTransform(String),
    /// A long-running orderbook operation was cancelled through a cooperative cancellation token.
    #[error("orderbook operation was cancelled")]
    Cancelled,
}

impl From<OrderBookApiError> for OrderbookError {
    fn from(value: OrderBookApiError) -> Self {
        Self::Api(Box::new(value))
    }
}

impl From<Cancelled> for OrderbookError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}

impl From<reqwest::Error> for OrderbookError {
    fn from(error: reqwest::Error) -> Self {
        let message = classify_reqwest_error(error);
        if message.starts_with("decode:") || message.starts_with("body:") {
            Self::Serialization(message)
        } else {
            Self::Transport(message)
        }
    }
}

/// Classifies a `reqwest::Error`, strips any attached URL, and returns a sanitized message.
///
/// The transport error is partitioned through `is_timeout`, `is_connect`,
/// `is_redirect`, `is_decode`, `is_body`, `is_builder`, `is_request`, and
/// `is_status`. [`reqwest::Error::without_url`] is called before the
/// [`std::fmt::Display`] implementation runs so partner-route URLs and their
/// query parameters cannot leak through error text.
#[must_use]
pub fn classify_reqwest_error(error: reqwest::Error) -> String {
    let sanitized = error.without_url();
    let class = reqwest_error_class(&sanitized);
    format!("{class}: {sanitized}")
}

fn reqwest_error_class(error: &reqwest::Error) -> &'static str {
    if error.is_timeout() {
        return "timeout";
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        if error.is_connect() {
            return "connect";
        }
        if error.is_redirect() {
            return "redirect";
        }
    }
    if error.is_decode() {
        "decode"
    } else if error.is_body() {
        "body"
    } else if error.is_builder() {
        "builder"
    } else if error.is_request() {
        "request"
    } else if error.is_status() {
        "status"
    } else {
        "other"
    }
}
