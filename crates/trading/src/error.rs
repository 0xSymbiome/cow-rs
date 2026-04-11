use cow_sdk_app_data::AppDataError;
use cow_sdk_contracts::{ContractsError, SigningScheme};
use cow_sdk_core::CoreError;
use cow_sdk_orderbook::OrderbookError;
use cow_sdk_signing::SigningError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TradingError {
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error(transparent)]
    AppData(#[from] AppDataError),
    #[error(transparent)]
    Contracts(#[from] ContractsError),
    #[error(transparent)]
    Orderbook(#[from] OrderbookError),
    #[error(transparent)]
    Signing(#[from] SigningError),
    #[error("Missing quoter parameters: {0}")]
    MissingQuoterParameters(String),
    #[error("Missing trader parameters: {0}")]
    MissingTraderParameters(String),
    #[error(
        "Cannot specify both validFor and validTo. Use validFor for relative time or validTo for absolute time."
    )]
    QuoteValidityConflict,
    #[error("quote response is missing quote id required for {0}")]
    MissingQuoteId(&'static str),
    #[error("owner address is required for quote-only flows")]
    MissingOwner,
    #[error(
        "injected orderbook client fixes {field} to `{configured}`, but `{requested}` was requested"
    )]
    InjectedOrderbookContextConflict {
        field: &'static str,
        requested: String,
        configured: String,
    },
    #[error("signer error during {operation}: {message}")]
    Signer {
        operation: &'static str,
        message: String,
    },
    #[error("provider error during {operation}: {message}")]
    Provider {
        operation: &'static str,
        message: String,
    },
    #[error("invalid numeric value for {field}: {value}")]
    InvalidNumeric { field: &'static str, value: String },
    #[error("numeric overflow for {field}: {value}")]
    NumericOverflow { field: &'static str, value: String },
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("unsupported local signer-generated scheme `{scheme:?}`")]
    UnsupportedLocalSigningScheme { scheme: SigningScheme },
}
