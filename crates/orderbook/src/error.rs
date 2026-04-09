use cow_sdk_core::CoreError;
use thiserror::Error;

use crate::request::OrderBookApiError;

#[derive(Debug, Error)]
pub enum OrderbookError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error(transparent)]
    Api(Box<OrderBookApiError>),
    #[error("transport error: {0}")]
    Transport(String),
    #[error("serialization error: {0}")]
    Serialization(String),
    #[error("invalid trades query: {0}")]
    InvalidTradesQuery(String),
    #[error("invalid quote request: {0}")]
    InvalidQuoteRequest(String),
    #[error("invalid transform: {0}")]
    InvalidTransform(String),
}

impl From<OrderBookApiError> for OrderbookError {
    fn from(value: OrderBookApiError) -> Self {
        Self::Api(Box::new(value))
    }
}
