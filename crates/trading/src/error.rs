use cow_sdk_app_data::AppDataError;
use cow_sdk_contracts::{ContractsError, SigningScheme};
use cow_sdk_core::CoreError;
use cow_sdk_orderbook::OrderbookError;
use cow_sdk_signing::SigningError;
use thiserror::Error;

/// Errors returned by trading orchestration, quote construction, and submission helpers.
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
    /// Missing quote-only parameters after precedence resolution.
    #[error("Missing quoter parameters: {0}")]
    MissingQuoterParameters(String),
    /// Missing trading parameters after precedence resolution.
    #[error("Missing trader parameters: {0}")]
    MissingTraderParameters(String),
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
        requested: String,
        /// Value fixed by the injected orderbook client.
        configured: String,
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
        quoted: String,
        /// Value used by the submission flow.
        submitted: String,
    },
    /// Recoverable local signing requires the submission owner to match the signer address.
    #[error(
        "recoverable signing scheme `{scheme:?}` requires owner `{owner}` to match signer `{signer}`"
    )]
    RecoverableSignatureOwnerMismatch {
        /// Recoverable signing scheme selected for submission.
        scheme: cow_sdk_orderbook::SigningScheme,
        /// Explicit owner used in the order payload.
        owner: String,
        /// Address resolved from the signing backend.
        signer: String,
    },
    /// Signer operation failed.
    #[error("signer error during {operation}: {message}")]
    Signer {
        /// Failed signer operation.
        operation: &'static str,
        /// Signer error message.
        message: String,
    },
    /// Provider operation failed.
    #[error("provider error during {operation}: {message}")]
    Provider {
        /// Failed provider operation.
        operation: &'static str,
        /// Provider error message.
        message: String,
    },
    /// Numeric parsing failed for a public string field.
    #[error("invalid numeric value for {field}: {value}")]
    InvalidNumeric {
        /// Public field name that could not be parsed.
        field: &'static str,
        /// Original field value supplied to the helper.
        value: String,
    },
    /// Numeric conversion overflowed the supported public representation.
    #[error("numeric overflow for {field}: {value}")]
    NumericOverflow {
        /// Public field name that overflowed.
        field: &'static str,
        /// Original field value that exceeded the supported range.
        value: String,
    },
    /// Input violated a documented helper precondition.
    #[error("invalid input: {0}")]
    InvalidInput(String),
    /// Local signing produced a scheme that the public workflow does not accept.
    #[error("unsupported local signer-generated scheme `{scheme:?}`")]
    UnsupportedLocalSigningScheme {
        /// Local signing scheme returned by the signer integration.
        scheme: SigningScheme,
    },
}
