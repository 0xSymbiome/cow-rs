//! Typed `CoW` Protocol orderbook transport models, request policy, and response
//! transforms.

/// High-level orderbook client with chain/env-aware endpoint resolution.
pub mod api;
/// Typed orderbook client errors.
pub mod error;
/// Request execution policy, retry rules, and low-level transport helpers.
pub mod request;
/// Orderbook response normalization helpers.
pub mod transform;
/// Public wire DTOs and builder-style request models for the orderbook API.
pub mod types;

pub use api::OrderBookApi;
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
