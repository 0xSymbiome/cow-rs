//! Typed classification of CoW Protocol orderbook rejection responses.
//!
//! Every non-2xx response returned by the orderbook service carries a
//! universal JSON envelope of shape
//! `{ "errorType": "<tag>", "description": "<message>", "data": <value>? }`.
//! [`OrderbookRejection`] maps every services-authoritative `errorType`
//! tag to a typed variant so SDK consumers can pattern-match structured
//! rejection outcomes instead of comparing free-form strings. The enum
//! is `#[non_exhaustive]`, and the final [`OrderbookRejection::Unknown`]
//! variant guarantees forward compatibility when services introduces a
//! new tag: deserialisation never silently coerces an unknown tag into
//! a default placeholder, but the SDK can still surface a sanitized
//! wire-code diagnostic while keeping the free-form description behind
//! explicit redacted access.
//!
//! The wire shape is authoritative: services encodes order-creation,
//! quote, cancellation, lookup, app-data registration, and
//! solver-competition failures through the same envelope. The
//! per-variant list below is the union of every `errorType` tag
//! emitted across `POST /orders`, `POST /quote`, `PUT /app_data`,
//! `GET /orders/{uid}`, `GET /solver_competition/{...}`,
//! `DELETE /orders`, and `DELETE /orders/{uid}`, grouped by their
//! upstream handler families for reviewability.

use std::fmt;

use cow_sdk_core::{Amount, REDACTED_PLACEHOLDER, Redacted};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Sanitized services rejection tag.
///
/// The orderbook wire contract uses short identifier tags. Inputs outside that
/// reviewed shape are collapsed to the shared redaction marker so arbitrary
/// server or caller text cannot leak through the forward-compatible unknown-code
/// fallback.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize)]
#[serde(transparent)]
pub struct OrderbookRejectionCode(String);

impl OrderbookRejectionCode {
    /// Creates a sanitized rejection-code value.
    #[must_use]
    pub fn new(code: impl Into<String>) -> Self {
        let code = code.into();
        if is_safe_rejection_code(&code) {
            Self(code)
        } else {
            Self(REDACTED_PLACEHOLDER.to_owned())
        }
    }

    /// Returns the sanitized code string.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OrderbookRejectionCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl From<String> for OrderbookRejectionCode {
    fn from(code: String) -> Self {
        Self::new(code)
    }
}

impl From<&str> for OrderbookRejectionCode {
    fn from(code: &str) -> Self {
        Self::new(code)
    }
}

/// Structured rejection code returned by the `CoW` Protocol orderbook.
///
/// Use [`parse_rejection`] to classify a transport-level response body
/// into this type. Variants preserve the wire spelling documented by
/// the services handler files; the permanent
/// [`OrderbookRejection::Unknown`] fallback keeps the SDK forward
/// compatible with new services codes without silently discarding
/// them.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize)]
pub enum OrderbookRejection {
    // --- Order-creation: structural / duplicate ---
    /// Order with the same UID already exists on this deployment.
    #[error("duplicated order")]
    DuplicatedOrder,
    /// Replacement flow rejected because an older order is still being
    /// actively bid on.
    #[error("old order is actively being bid on")]
    OldOrderActivelyBidOn,

    // --- Order-creation: quote linkage ---
    /// No stored quote matches the submitted `quoteId`.
    #[error("quote not found")]
    QuoteNotFound,
    /// The bound quote could not be verified against the current chain
    /// state.
    #[error("quote not verified")]
    QuoteNotVerified,

    // --- Signing and signer identity ---
    /// Order requires a `from` address for the declared signing scheme
    /// but none was supplied.
    #[error("missing from address")]
    MissingFrom,
    /// Recovered signer does not match the declared `from` address.
    #[error("wrong owner")]
    WrongOwner,
    /// EIP-1271 `isValidSignature` call rejected the signature on
    /// chain.
    #[error("invalid EIP-1271 signature")]
    InvalidEip1271Signature,
    /// Signature bytes fail ECDSA or EIP-712 validation.
    #[error("invalid signature")]
    InvalidSignature,
    /// Signing scheme is not compatible with the declared submission
    /// mode.
    #[error("incompatible signing scheme")]
    IncompatibleSigningScheme,

    // --- Balance / allowance ---
    /// Sell-side balance is below the required sell amount.
    #[error("insufficient balance")]
    InsufficientBalance,
    /// Sell-side allowance to the vault relayer is below the required
    /// sell amount.
    #[error("insufficient allowance")]
    InsufficientAllowance,

    // --- Amounts / gas / limits ---
    /// Declared sell or buy amount is zero.
    #[error("zero amount")]
    ZeroAmount,
    /// Declared fee amount on the signed envelope is non-zero.
    #[error("non-zero fee amount on signed envelope")]
    NonZeroFee,
    /// Sell amount plus fee would overflow `uint256`.
    #[error("sell amount overflow")]
    SellAmountOverflow,
    /// Settlement would require more gas than the account limit
    /// allows.
    #[error("too much gas")]
    TooMuchGas,
    /// Account already has the maximum number of open limit orders.
    #[error("too many limit orders")]
    TooManyLimitOrders,

    // --- Simulation ---
    /// Sell-token transfer simulation reverted.
    #[error("transfer simulation failed")]
    TransferSimulationFailed,

    // --- Validity window ---
    /// `validTo` is too close to the current block.
    #[error("insufficient validTo")]
    InsufficientValidTo,
    /// `validTo` extends beyond the allowed window.
    #[error("excessive validTo")]
    ExcessiveValidTo,

    // --- Token identity / destination / source ---
    /// The native asset cannot be declared as the sell token on a
    /// non-EthFlow path.
    #[error("invalid native sell token")]
    InvalidNativeSellToken,
    /// Buy token and sell token are the same address.
    #[error("same buy and sell token")]
    SameBuyAndSellToken,
    /// Declared sell or buy token is not supported on this deployment.
    #[error("unsupported token")]
    UnsupportedToken,
    /// Buy-side destination variant is not supported on this route.
    #[error("unsupported buy token destination")]
    UnsupportedBuyTokenDestination,
    /// Sell-side source variant is not supported on this route.
    #[error("unsupported sell token source")]
    UnsupportedSellTokenSource,
    /// Declared order type is not supported on this route.
    #[error("unsupported order type")]
    UnsupportedOrderType,

    // --- AppData ---
    /// App-data registration rejected the supplied document and
    /// preserves the validator message in the redacted wire
    /// `description` field.
    #[error("AppDataInvalid: {message}")]
    AppDataInvalid {
        /// Services-authored `description` string carried on the
        /// rejection envelope and redacted on public rendering.
        message: Redacted<String>,
    },
    /// App-data document failed validation.
    #[error("invalid app data")]
    InvalidAppData,
    /// Declared app-data hash does not match the supplied document.
    #[error("app-data hash mismatch")]
    AppDataHashMismatch,
    /// A previously-registered full app-data document for the same
    /// hash differs from the supplied bytes.
    #[error("AppDataMismatch: {message}")]
    AppDataMismatch {
        /// Services-authored `description` string carried on the
        /// rejection envelope and redacted on public rendering.
        message: Redacted<String>,
    },
    /// App-data `metadata.signer` must match the declared `from`
    /// address.
    #[error("app-data from mismatch")]
    AppdataFromMismatch,
    /// Services failed to serialize canonical metadata derived from
    /// the submitted app data.
    #[error("metadata serialization failed")]
    MetadataSerializationFailed,

    // --- Liquidity / route / solver ---
    /// No route or liquidity available for the requested trade.
    #[error("no liquidity")]
    NoLiquidity,
    /// Trade was requested outside the route's allowed operating
    /// window.
    #[error("trading outside allowed window")]
    TradingOutsideAllowedWindow,
    /// Token is temporarily suspended from trading.
    #[error("token temporarily suspended")]
    TokenTemporarilySuspended,
    /// Route has insufficient liquidity for the requested trade size.
    #[error("insufficient liquidity")]
    InsufficientLiquidity,
    /// Custom error surfaced by an upstream solver.
    #[error("custom solver error")]
    CustomSolverError,

    // --- GET-side filters / pagination ---
    /// Trade lookup rejected because the query did not specify exactly
    /// one of `owner` or `orderUid`.
    #[error("invalid trade filter")]
    InvalidTradeFilter,
    /// Paginated trade lookup rejected because the requested limit was
    /// outside the services-supported range.
    #[error("invalid limit")]
    InvalidLimit,
    /// User-order lookup rejected because the requested pagination
    /// limit was outside the services-supported range.
    ///
    /// Services intentionally emits the `LIMIT_OUT_OF_BOUNDS` wire tag
    /// in `SCREAMING_SNAKE_CASE`; do not casing-fix this tag to `PascalCase`.
    #[serde(rename = "LIMIT_OUT_OF_BOUNDS")]
    #[error("limit out of bounds")]
    LimitOutOfBounds,

    // --- Quote-only ---
    /// Quote rejected because the supplied sell amount does not cover
    /// the required fee. The `data` payload carries the required
    /// minimum fee amount.
    #[error("sell amount does not cover fee")]
    SellAmountDoesNotCoverFee {
        /// Required minimum fee amount as reported by services on the
        /// `data.fee_amount` field of the wire envelope.
        fee_amount: Amount,
    },

    // --- Cancellation-only ---
    /// Cancellation rejected because the order is already cancelled.
    #[error("already cancelled")]
    AlreadyCancelled,
    /// Cancellation rejected because the order has already been fully
    /// executed.
    #[error("order fully executed")]
    OrderFullyExecuted,
    /// Cancellation rejected because the order has already expired.
    #[error("order expired")]
    OrderExpired,
    /// Order UID is not known to this deployment.
    #[error("order not found")]
    OrderNotFound,
    /// Lookup-path 404 emitted by `GET /orders/{uid}` and
    /// `GET /solver_competition/{...}`.
    ///
    /// Distinct from [`OrderbookRejection::OrderNotFound`], which is
    /// the cancel-path 404 emitted by `DELETE /orders/{uid}`.
    #[error("NotFound: {message}")]
    NotFound {
        /// Services-authored `description` string carried on the
        /// rejection envelope and redacted on public rendering.
        message: Redacted<String>,
    },
    /// On-chain orders must be cancelled through the on-chain flow.
    #[error("on-chain order")]
    OnChainOrder,

    // --- Authorization / deny-list ---
    /// Deny-listed or otherwise forbidden request.
    #[error("forbidden")]
    Forbidden,

    // --- Last-resort server failure ---
    /// Upstream services handler surfaced an unclassified
    /// internal-server error.
    #[error("internal server error")]
    InternalServerError,

    // --- Fallback for unknown / newly-added codes ---
    /// Wire-envelope classification that the SDK does not yet model.
    ///
    /// The SDK keeps this variant as a permanent fallback so a new
    /// services-side tag never silently becomes a default placeholder
    /// or falls through the untyped [`crate::OrderbookError::Api`]
    /// envelope path: callers still receive the typed wire code and
    /// description through deliberate `Redacted<T>` accessors while
    /// public renderings preserve sanitized code tags and redact
    /// free-form descriptions. The `#[non_exhaustive]` attribute
    /// guarantees that promoting an unknown tag to a dedicated variant
    /// remains a non-breaking change.
    #[error("unknown rejection code `{code}`: {message}")]
    Unknown {
        /// Sanitized `errorType` tag as supplied by services.
        code: OrderbookRejectionCode,
        /// Raw `description` string as supplied by services, redacted on
        /// public rendering.
        message: Redacted<String>,
    },
}

/// Coarse, action-oriented partition over [`OrderbookRejection`].
///
/// [`OrderbookRejection::category`] returns this for callers that only need
/// to decide *what to do* about a rejection — fix the request, fund the
/// wallet, re-quote, stop, wait and retry, or escalate — without matching
/// every typed wire tag. It is an additive accessor: the full
/// [`OrderbookRejection`] taxonomy is unchanged, and this partition is
/// `#[non_exhaustive]` so a new category can be introduced without a breaking
/// change. The category carries no message or code, so it never re-exposes a
/// redacted rejection payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum OrderbookRejectionCategory {
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
    /// No solver, route, or liquidity can currently fill the trade; the condition may clear later.
    Unfulfillable,
    /// An upstream server-side fault.
    Server,
    /// A wire tag the SDK does not yet model, preserved for forward compatibility.
    Unknown,
}

impl OrderbookRejection {
    /// Returns the coarse [`OrderbookRejectionCategory`] for this rejection.
    ///
    /// The category names the consumer action a rejection calls for, so a
    /// caller that only needs coarse handling avoids matching every typed
    /// variant. The mapping is exhaustive over the typed tags with no
    /// wildcard arm, so a future wire variant must be assigned a category at
    /// the source and cannot be silently misclassified.
    #[must_use]
    pub const fn category(&self) -> OrderbookRejectionCategory {
        match self {
            Self::Forbidden => OrderbookRejectionCategory::Authorization,
            Self::InsufficientBalance | Self::InsufficientAllowance => {
                OrderbookRejectionCategory::InsufficientFunds
            }
            Self::QuoteNotFound | Self::OrderNotFound | Self::NotFound { .. } => {
                OrderbookRejectionCategory::NotFound
            }
            Self::DuplicatedOrder
            | Self::OldOrderActivelyBidOn
            | Self::AlreadyCancelled
            | Self::OrderFullyExecuted
            | Self::OrderExpired
            | Self::OnChainOrder => OrderbookRejectionCategory::Conflict,
            Self::NoLiquidity
            | Self::InsufficientLiquidity
            | Self::TradingOutsideAllowedWindow
            | Self::TokenTemporarilySuspended
            | Self::CustomSolverError => OrderbookRejectionCategory::Unfulfillable,
            Self::InternalServerError | Self::MetadataSerializationFailed => {
                OrderbookRejectionCategory::Server
            }
            Self::Unknown { .. } => OrderbookRejectionCategory::Unknown,
            Self::QuoteNotVerified
            | Self::MissingFrom
            | Self::WrongOwner
            | Self::InvalidEip1271Signature
            | Self::InvalidSignature
            | Self::IncompatibleSigningScheme
            | Self::ZeroAmount
            | Self::NonZeroFee
            | Self::SellAmountOverflow
            | Self::TooMuchGas
            | Self::TooManyLimitOrders
            | Self::TransferSimulationFailed
            | Self::InsufficientValidTo
            | Self::ExcessiveValidTo
            | Self::InvalidNativeSellToken
            | Self::SameBuyAndSellToken
            | Self::UnsupportedToken
            | Self::UnsupportedBuyTokenDestination
            | Self::UnsupportedSellTokenSource
            | Self::UnsupportedOrderType
            | Self::AppDataInvalid { .. }
            | Self::InvalidAppData
            | Self::AppDataHashMismatch
            | Self::AppDataMismatch { .. }
            | Self::AppdataFromMismatch
            | Self::InvalidTradeFilter
            | Self::InvalidLimit
            | Self::LimitOutOfBounds
            | Self::SellAmountDoesNotCoverFee { .. } => OrderbookRejectionCategory::InvalidOrder,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RejectionEnvelope {
    error_type: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

/// Classifies an orderbook response body into a typed
/// [`OrderbookRejection`].
///
/// The parser deserialises the universal rejection envelope (keyed on
/// the `errorType` string) and maps every services-authoritative tag
/// to the matching variant. Variants whose wire payload includes
/// machine-readable data (the quote-only
/// [`OrderbookRejection::SellAmountDoesNotCoverFee`]) or a
/// services-authored message that callers may need to preserve
/// ([`OrderbookRejection::AppDataInvalid`],
/// [`OrderbookRejection::AppDataMismatch`], and
/// [`OrderbookRejection::NotFound`]) consume the structured envelope
/// fields; the remaining variants stay unit-like because the matching
/// `description` string is only diagnostic.
///
/// The parser returns `None` when the supplied `body` is not a JSON
/// object with a string `errorType` field. Callers surface that case
/// through [`crate::OrderbookError::Api`] (wrapping the structured
/// [`crate::OrderbookApiError`] envelope, which preserves the decoded
/// [`crate::ResponseBody`] — including the `Text` variant for
/// plain-text bodies — and the derived public message) rather than
/// through [`crate::OrderbookError::Rejected`]. Unknown tags are
/// preserved as [`OrderbookRejection::Unknown`] so forward
/// compatibility with new services codes never degrades to a silent
/// placeholder.
///
/// The `status` argument is accepted so future dispatchers can key on
/// the `(status, errorType)` pair for tags that services emits at
/// different statuses depending on the call site. The current mapping
/// is keyed on `errorType` alone.
#[must_use]
pub fn parse_rejection(status: StatusCode, body: &[u8]) -> Option<OrderbookRejection> {
    let _ = status;
    let envelope: RejectionEnvelope = serde_json::from_slice(body).ok()?;
    Some(classify(envelope))
}

fn classify(envelope: RejectionEnvelope) -> OrderbookRejection {
    match envelope.error_type.as_str() {
        "DuplicatedOrder" => OrderbookRejection::DuplicatedOrder,
        "OldOrderActivelyBidOn" => OrderbookRejection::OldOrderActivelyBidOn,
        "QuoteNotFound" => OrderbookRejection::QuoteNotFound,
        "QuoteNotVerified" => OrderbookRejection::QuoteNotVerified,
        "MissingFrom" => OrderbookRejection::MissingFrom,
        "WrongOwner" => OrderbookRejection::WrongOwner,
        "InvalidEip1271Signature" => OrderbookRejection::InvalidEip1271Signature,
        "InvalidSignature" => OrderbookRejection::InvalidSignature,
        "IncompatibleSigningScheme" => OrderbookRejection::IncompatibleSigningScheme,
        "InsufficientBalance" => OrderbookRejection::InsufficientBalance,
        "InsufficientAllowance" => OrderbookRejection::InsufficientAllowance,
        "ZeroAmount" => OrderbookRejection::ZeroAmount,
        "NonZeroFee" => OrderbookRejection::NonZeroFee,
        "SellAmountOverflow" => OrderbookRejection::SellAmountOverflow,
        "TooMuchGas" => OrderbookRejection::TooMuchGas,
        "TooManyLimitOrders" => OrderbookRejection::TooManyLimitOrders,
        "TransferSimulationFailed" => OrderbookRejection::TransferSimulationFailed,
        "InsufficientValidTo" => OrderbookRejection::InsufficientValidTo,
        "ExcessiveValidTo" => OrderbookRejection::ExcessiveValidTo,
        "InvalidNativeSellToken" => OrderbookRejection::InvalidNativeSellToken,
        "SameBuyAndSellToken" => OrderbookRejection::SameBuyAndSellToken,
        "UnsupportedToken" => OrderbookRejection::UnsupportedToken,
        "UnsupportedBuyTokenDestination" => OrderbookRejection::UnsupportedBuyTokenDestination,
        "UnsupportedSellTokenSource" => OrderbookRejection::UnsupportedSellTokenSource,
        "UnsupportedOrderType" => OrderbookRejection::UnsupportedOrderType,
        "AppDataInvalid" => message_variant(envelope, |message| {
            OrderbookRejection::AppDataInvalid { message }
        }),
        "InvalidAppData" => OrderbookRejection::InvalidAppData,
        "AppDataHashMismatch" => OrderbookRejection::AppDataHashMismatch,
        "AppDataMismatch" => message_variant(envelope, |message| {
            OrderbookRejection::AppDataMismatch { message }
        }),
        "AppdataFromMismatch" => OrderbookRejection::AppdataFromMismatch,
        "MetadataSerializationFailed" => OrderbookRejection::MetadataSerializationFailed,
        "NoLiquidity" => OrderbookRejection::NoLiquidity,
        "TradingOutsideAllowedWindow" => OrderbookRejection::TradingOutsideAllowedWindow,
        "TokenTemporarilySuspended" => OrderbookRejection::TokenTemporarilySuspended,
        "InsufficientLiquidity" => OrderbookRejection::InsufficientLiquidity,
        "CustomSolverError" => OrderbookRejection::CustomSolverError,
        "InvalidTradeFilter" => OrderbookRejection::InvalidTradeFilter,
        "InvalidLimit" => OrderbookRejection::InvalidLimit,
        "LIMIT_OUT_OF_BOUNDS" => OrderbookRejection::LimitOutOfBounds,
        "SellAmountDoesNotCoverFee" => {
            parse_sell_amount_does_not_cover_fee(&envelope).unwrap_or_else(|| unknown(envelope))
        }
        "AlreadyCancelled" => OrderbookRejection::AlreadyCancelled,
        "OrderFullyExecuted" => OrderbookRejection::OrderFullyExecuted,
        "OrderExpired" => OrderbookRejection::OrderExpired,
        "OrderNotFound" => OrderbookRejection::OrderNotFound,
        "NotFound" => message_variant(envelope, |message| OrderbookRejection::NotFound { message }),
        "OnChainOrder" => OrderbookRejection::OnChainOrder,
        "Forbidden" => OrderbookRejection::Forbidden,
        "InternalServerError" => OrderbookRejection::InternalServerError,
        _ => unknown(envelope),
    }
}

fn message_variant(
    envelope: RejectionEnvelope,
    constructor: impl FnOnce(Redacted<String>) -> OrderbookRejection,
) -> OrderbookRejection {
    constructor(envelope.description.into())
}

fn parse_sell_amount_does_not_cover_fee(
    envelope: &RejectionEnvelope,
) -> Option<OrderbookRejection> {
    let fee_amount = envelope
        .data
        .as_ref()?
        .get("fee_amount")
        .or_else(|| envelope.data.as_ref()?.get("feeAmount"))
        .and_then(serde_json::Value::as_str)
        .and_then(|value| Amount::new(value).ok())?;
    Some(OrderbookRejection::SellAmountDoesNotCoverFee { fee_amount })
}

fn unknown(envelope: RejectionEnvelope) -> OrderbookRejection {
    OrderbookRejection::Unknown {
        code: envelope.error_type.into(),
        message: envelope.description.into(),
    }
}

fn is_safe_rejection_code(code: &str) -> bool {
    !code.is_empty()
        && code.len() <= 48
        && code.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        && code
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
