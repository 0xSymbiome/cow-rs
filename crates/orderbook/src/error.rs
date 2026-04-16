use cow_sdk_core::CoreError;
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
}

impl From<OrderBookApiError> for OrderbookError {
    fn from(value: OrderBookApiError) -> Self {
        Self::Api(Box::new(value))
    }
}
