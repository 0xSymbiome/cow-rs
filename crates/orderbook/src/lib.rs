//! Typed `CoW` Protocol orderbook transport models, request policy, and response
//! transforms.
//!
//! # Parity-scope invariant: `fee_amount` is not a public builder setter
//!
//! The cow-protocol services backend rejects orders that carry a non-zero
//! order-level fee, so the submission path always wires `"feeAmount": "0"`
//! and no public builder on this crate exposes a `fee_amount(...)` setter.
//! The compile-fail witnesses below prove that attempting `.fee_amount(...)`
//! on any public builder fails to compile. If any of the snippets ever
//! compiles, the intentional parity-scope divergence has regressed.
//!
//! ```compile_fail
//! use cow_sdk_core::{Address, AppDataHash, OrderKind};
//! use cow_sdk_orderbook::QuoteData;
//!
//! let address = Address::new("0x0000000000000000000000000000000000000001").unwrap();
//! let app_data = AppDataHash::new(
//!     "0x0000000000000000000000000000000000000000000000000000000000000000",
//! )
//! .unwrap();
//! let _quote = QuoteData::new(
//!     address.clone(),
//!     address,
//!     "1",
//!     "1",
//!     1,
//!     app_data,
//!     OrderKind::Sell,
//! )
//! .fee_amount("1");
//! ```
//!
//! ```compile_fail
//! use cow_sdk_core::{Address, OrderKind};
//! use cow_sdk_orderbook::{OrderCreation, SigningScheme};
//!
//! let address = Address::new("0x0000000000000000000000000000000000000001").unwrap();
//! let _order = OrderCreation::new(
//!     address.clone(),
//!     address.clone(),
//!     "1",
//!     "1",
//!     1,
//!     OrderKind::Sell,
//!     SigningScheme::Eip712,
//!     "0x",
//!     address,
//! )
//! .fee_amount("1");
//! ```

#![warn(missing_docs)]

/// High-level orderbook client with chain/env-aware endpoint resolution.
pub mod api;
/// Typestate-checked construction surface for [`OrderBookApi`].
pub mod builder;
/// Typed orderbook client errors.
pub mod error;
/// Request execution policy, retry rules, and low-level transport helpers.
pub mod request;
/// Orderbook response normalization helpers.
pub mod transform;
/// Public wire DTOs and builder-style request models for the orderbook API.
pub mod types;

pub use api::OrderBookApi;
pub use builder::{
    ChainIdSet, ChainIdUnset, EnvSet, EnvUnset, OrderBookApiBuilder, TransportSet, TransportUnset,
};
pub use error::OrderbookError;
pub use request::{
    BAD_GATEWAY, DEFAULT_INTERVAL_LABEL, DEFAULT_MAX_ATTEMPTS, DEFAULT_ORDERBOOK_USER_AGENT,
    DEFAULT_TOKENS_PER_INTERVAL, GATEWAY_TIMEOUT, HttpMethod, INTERNAL_SERVER_ERROR,
    OrderBookApiError, OrderBookTransportPolicy, REQUEST_TIMEOUT, RETRYABLE_STATUS_CODES,
    RequestPolicy, ResponseBody, SERVICE_UNAVAILABLE, TOO_EARLY, TOO_MANY_REQUESTS,
};
pub use transform::{calculate_total_fee, transform_order, transform_orders};
pub use types::{
    Address, ApiBaseUrls, ApiContext, ApiContextOverride, AppDataHash, AppDataObject, Auction,
    CompetitionAuction, CompetitionOrderStatus, CompetitionOrderStatusKind, CowEnv, ENVS_LIST,
    EVM_NATIVE_CURRENCY_ADDRESS, EcdsaSigningScheme, EnvBaseUrlOverrides, EthflowData,
    GetOrdersRequest, GetTradesRequest, NativePriceResponse, Order, OrderBalance,
    OrderCancellations, OrderClass, OrderCreation, OrderKind, OrderQuoteRequest,
    OrderQuoteResponse, OrderStatus, OrderUid, PriceQuality, QuoteAmountsAndCosts, QuoteData,
    QuoteSide, SigningScheme, SolverCompetitionResponse, SolverExecution, SolverSettlement,
    SupportedChainId, TotalSurplus, Trade,
};

/// Backwards-compatible alias for the orderbook API client.
pub type OrderbookClient = OrderBookApi;
