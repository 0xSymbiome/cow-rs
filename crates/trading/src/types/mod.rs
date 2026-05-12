//! Shared trading DTOs, trait seams, and settings types.

pub use self::overrides::QuoteRequestOverride;
pub use self::{
    advanced::*, allowance::*, app_code::*, eip1271::*, options::*, result::*, seams::*,
    slippage::*, trade::*, trader::*,
};

/// Compatibility alias for the transaction type returned by trading helpers.
pub type TradingTransactionParams = cow_sdk_core::TransactionRequest;

pub(crate) use self::context::*;
pub(crate) use self::overrides::{
    QuoteRequestParameterTargets, apply_app_data_parameter_overrides,
    apply_quote_request_parameter_overrides,
};

mod advanced;
mod allowance;
mod app_code;
mod context;
mod eip1271;
mod options;
mod overrides;
mod result;
mod seams;
mod slippage;
mod trade;
mod trader;
