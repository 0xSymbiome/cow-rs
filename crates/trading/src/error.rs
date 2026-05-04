use cow_sdk_app_data::AppDataError;
use cow_sdk_contracts::{ContractsError, SigningScheme};
use std::fmt;

use cow_sdk_core::{Cancelled, ChainId, CoreError, CowEnv, Redacted, ValidationReason};
use cow_sdk_orderbook::OrderbookError;
use cow_sdk_signing::SigningError;
use thiserror::Error;

use crate::types::AppCodeError;
use crate::validation::ClientRejection;

/// Value captured in an orderbook runtime-context conflict.
///
/// Typed protocol values remain visible for diagnostics, while URL-bearing
/// values stay redacted on public renderings.
#[non_exhaustive]
#[derive(Debug)]
pub enum OrderbookContextValue {
    /// Numeric chain id.
    ChainId(ChainId),
    /// `CoW` Protocol environment.
    Env(CowEnv),
    /// Resolved orderbook base URL.
    BaseUrl(Redacted<String>),
}

impl fmt::Display for OrderbookContextValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ChainId(chain_id) => chain_id.fmt(f),
            Self::Env(env) => env.fmt(f),
            Self::BaseUrl(base_url) => base_url.fmt(f),
        }
    }
}

/// Errors returned by trading orchestration, quote construction, and submission helpers.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum TradingError {
    /// Shared core type or configuration error.
    #[error(transparent)]
    Core(#[from] CoreError),
    /// App-data generation or validation error.
    #[error(transparent)]
    AppData(#[from] AppDataError),
    /// Contract helper or ABI error.
    #[error(transparent)]
    Contracts(#[from] ContractsError),
    /// Orderbook transport or API error.
    #[error(transparent)]
    Orderbook(#[from] OrderbookError),
    /// Signing helper error.
    #[error(transparent)]
    Signing(#[from] SigningError),
    /// App-code validation error.
    #[error(transparent)]
    AppCode(#[from] AppCodeError),
    /// Missing quote-only parameters after precedence resolution.
    #[error("Missing quoter parameters: {0}")]
    MissingQuoterParameters(&'static str),
    /// Missing trading parameters after precedence resolution.
    #[error("Missing trader parameters: {0}")]
    MissingTraderParameters(&'static str),
    /// Both relative and absolute quote-validity values were provided simultaneously.
    #[error(
        "Cannot specify both validFor and validTo. Use validFor for relative time or validTo for absolute time."
    )]
    QuoteValidityConflict,
    /// The quote response omitted a quote id required by the named workflow.
    #[error("quote response is missing quote id required for {0}")]
    MissingQuoteId(&'static str),
    /// Quote-only flows require an explicit owner from settings or call-level input.
    #[error("owner address is required for quote-only flows")]
    MissingOwner,
    /// Order submission requires an explicit owner or a signer address that can supply one.
    #[error("owner address is required for order submission")]
    MissingSubmissionOwner,
    /// Injected orderbook context conflicts with requested chain or environment.
    #[error(
        "injected orderbook client fixes {field} to `{configured}`, but `{requested}` was requested"
    )]
    InjectedOrderbookContextConflict {
        /// Conflicting field name.
        field: &'static str,
        /// Requested value.
        requested: OrderbookContextValue,
        /// Value fixed by the injected orderbook client.
        configured: OrderbookContextValue,
    },
    /// Quote-derived posting requires the original orderbook runtime binding.
    #[error(
        "quote results are missing the originating orderbook runtime binding; requote before posting"
    )]
    MissingQuoteOrderbookBinding,
    /// Quote-derived posting changed runtime authority between quote and submission.
    #[error("quote results fix {field} to `{quoted}`, but submission uses `{submitted}`")]
    QuoteOrderbookBindingConflict {
        /// Conflicting field name.
        field: &'static str,
        /// Value captured by the quote flow.
        quoted: OrderbookContextValue,
        /// Value used by the submission flow.
        submitted: OrderbookContextValue,
    },
    /// Typed client-side rejection surfaced before any HTTP transport runs.
    ///
    /// Every variant of [`ClientRejection`] reflects a condition the
    /// reviewed services validator enforces so the client-side reject
    /// fires locally with a typed error rather than as an opaque 422
    /// response from the orderbook.
    #[error(transparent)]
    ClientRejected(#[from] ClientRejection),
    /// Signer operation failed.
    #[error("signer error during {operation}: {message}")]
    Signer {
        /// Failed signer operation.
        operation: &'static str,
        /// Signer error message.
        message: Redacted<String>,
    },
    /// Provider operation failed.
    #[error("provider error during {operation}: {message}")]
    Provider {
        /// Failed provider operation.
        operation: &'static str,
        /// Provider error message.
        message: Redacted<String>,
    },
    /// Numeric parsing failed for a public string field.
    #[error("invalid numeric value for {field}: {value}")]
    InvalidNumeric {
        /// Public field name that could not be parsed.
        field: &'static str,
        /// Original field value supplied to the helper.
        value: Redacted<String>,
    },
    /// Numeric conversion overflowed the supported public representation.
    #[error("numeric overflow for {field}: {value}")]
    NumericOverflow {
        /// Public field name that overflowed.
        field: &'static str,
        /// Original field value that exceeded the supported range.
        value: Redacted<String>,
    },
    /// Input violated a documented helper precondition.
    #[error("invalid input for field `{field}`: {reason}")]
    InvalidInput {
        /// Public field name that failed validation.
        field: &'static str,
        /// Canonical validation-failure mode.
        reason: ValidationReason,
    },
    /// Local signing produced a scheme that the public workflow does not accept.
    #[error("unsupported local signer-generated scheme `{scheme:?}`")]
    UnsupportedLocalSigningScheme {
        /// Local signing scheme returned by the signer integration.
        scheme: SigningScheme,
    },
    /// A long-running trading operation was cancelled through a cooperative cancellation token.
    #[error("trading operation was cancelled")]
    Cancelled,
    /// An injected orderbook client is required on `wasm32` targets because
    /// the browser runtime does not ship a default HTTP transport.
    #[error(
        "wasm32 targets require an injected orderbook client through TradingSdkOptions::with_orderbook_client"
    )]
    MissingInjectedOrderbookClient,
}

impl From<Cancelled> for TradingError {
    fn from(_: Cancelled) -> Self {
        Self::Cancelled
    }
}

impl From<std::convert::Infallible> for TradingError {
    fn from(error: std::convert::Infallible) -> Self {
        match error {}
    }
}
