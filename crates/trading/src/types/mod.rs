//! Shared trading DTOs, trait seams, and settings types.

pub use self::{params::*, result::*, seams::*};

/// Compatibility alias for the transaction type returned by trading helpers.
pub type TradingTransactionParams = cow_sdk_core::TransactionRequest;

pub(crate) use self::params::{
    QuoteRequestParameterTargets, apply_app_data_parameter_overrides,
    apply_quote_request_parameter_overrides,
};
pub(crate) use self::result::{
    validate_orderbook_chain_context, validate_orderbook_context, validate_orderbook_env_context,
    validate_quote_orderbook_binding,
};

mod params;
mod result;
mod seams;
