#![cfg_attr(doctest, doc = include_str!("../README.md"))]

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
//! use cow_sdk_core::{Address, Amount, AppDataHash, OrderKind};
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
//!     Amount::new("1").unwrap(),
//!     Amount::new("1").unwrap(),
//!     1,
//!     app_data,
//!     OrderKind::Sell,
//! )
//! .fee_amount("1");
//! ```
//!
//! ```compile_fail
//! use cow_sdk_core::{Address, Amount, OrderKind};
//! use cow_sdk_orderbook::{OrderCreation, SigningScheme};
//!
//! let address = Address::new("0x0000000000000000000000000000000000000001").unwrap();
//! let _order = OrderCreation::new(
//!     address.clone(),
//!     address.clone(),
//!     Amount::new("1").unwrap(),
//!     Amount::new("1").unwrap(),
//!     1,
//!     OrderKind::Sell,
//!     SigningScheme::Eip712,
//!     "0x",
//!     address,
//! )
//! .fee_amount("1");
//! ```

#![warn(missing_docs)]

use async_trait::async_trait;
use cow_sdk_core::{
    ApiContext as CoreApiContext, AppDataHash as CoreAppDataHash, CowEnv as CoreCowEnv,
    OrderUid as CoreOrderUid, SupportedChainId as CoreSupportedChainId,
};
use serde::{Deserialize, Serialize};

/// High-level orderbook client with chain/env-aware endpoint resolution.
pub mod api;
/// Typestate-checked construction surface for [`OrderBookApi`].
pub mod builder;
/// Typed orderbook client errors.
pub mod error;
/// Typed rejection taxonomy and wire-envelope parser for orderbook
/// non-2xx responses.
pub mod rejection;
/// Request execution policy, retry rules, and low-level transport helpers.
pub mod request;
/// Orderbook response normalization helpers.
pub mod transform;
/// Public wire DTOs and builder-style request models for the orderbook API.
pub mod types;

/// Runtime binding captured from an orderbook client for quote-derived workflows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OrderbookRuntimeBinding {
    /// Chain id fixed by the orderbook client.
    pub chain_id: CoreSupportedChainId,
    /// Environment fixed by the orderbook client.
    pub env: CoreCowEnv,
    /// Resolved base URL used by the orderbook client when it is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_base_url: Option<String>,
}

impl OrderbookRuntimeBinding {
    /// Creates a runtime binding with the required chain and environment identifiers.
    #[must_use]
    pub const fn new(chain_id: CoreSupportedChainId, env: CoreCowEnv) -> Self {
        Self {
            chain_id,
            env,
            resolved_base_url: None,
        }
    }

    /// Returns a copy of this binding with an explicit resolved base URL.
    #[must_use]
    pub fn with_resolved_base_url(mut self, url: impl Into<String>) -> Self {
        self.resolved_base_url = Some(url.into());
        self
    }
}

// The shared `OrderbookClient` trait is owned by `cow-sdk-orderbook` because
// it abstracts the orderbook concept itself; placing it on the trading crate
// would also make the orderbook/trading crate graph cyclic if orderbook tried
// to re-export it back. `cow-sdk-trading` re-exports the trait as an additive
// convenience so trading-crate consumers can compose against it without an
// explicit orderbook-crate import.
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// Minimal orderbook capability required by trading and composable consumers.
///
/// Consumers can import the trait directly from this crate without taking a
/// trading-crate dependency.
///
/// ```
/// use cow_sdk_orderbook::OrderbookClient;
/// fn assert_object_safe<T: OrderbookClient>(_: T) {}
/// fn assert_dyn(_: &dyn OrderbookClient) {}
/// ```
pub trait OrderbookClient: Send + Sync {
    /// Returns the effective orderbook API context.
    fn context(&self) -> &CoreApiContext;

    /// Returns the runtime binding used by this orderbook client.
    ///
    /// Implementations that apply additional endpoint overrides should override
    /// this method so quote-derived posting can validate the originating
    /// runtime authority precisely.
    fn runtime_binding(&self) -> OrderbookRuntimeBinding {
        OrderbookRuntimeBinding {
            chain_id: self.context().chain_id,
            env: self.context().env,
            resolved_base_url: self.context().resolved_base_url().ok(),
        }
    }

    /// Requests a quote from the orderbook.
    ///
    /// # Errors
    ///
    /// Returns the underlying orderbook error from the implementation.
    async fn get_quote(
        &self,
        request: &types::OrderQuoteRequest,
    ) -> Result<types::OrderQuoteResponse, error::OrderbookError>;

    /// Submits an order to the orderbook.
    ///
    /// # Errors
    ///
    /// Returns the underlying orderbook error from the implementation.
    async fn send_order(
        &self,
        request: &types::OrderCreation,
    ) -> Result<CoreOrderUid, error::OrderbookError>;

    /// Submits signed order cancellations to the orderbook.
    ///
    /// # Errors
    ///
    /// Returns the underlying orderbook error from the implementation.
    async fn send_signed_order_cancellations(
        &self,
        request: &types::OrderCancellations,
    ) -> Result<(), error::OrderbookError>;

    /// Fetches an order by UID.
    ///
    /// # Errors
    ///
    /// Returns the underlying orderbook error from the implementation.
    async fn get_order(
        &self,
        order_uid: &CoreOrderUid,
    ) -> Result<types::Order, error::OrderbookError>;

    /// Uploads full app-data for a specific app-data hash.
    ///
    /// # Errors
    ///
    /// Returns the underlying orderbook error from the implementation.
    async fn upload_app_data(
        &self,
        app_data_hash: &CoreAppDataHash,
        full_app_data: &str,
    ) -> Result<types::AppDataObject, error::OrderbookError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl OrderbookClient for api::OrderBookApi {
    fn context(&self) -> &CoreApiContext {
        self.context()
    }

    fn runtime_binding(&self) -> OrderbookRuntimeBinding {
        OrderbookRuntimeBinding {
            chain_id: self.context().chain_id,
            env: self.context().env,
            resolved_base_url: self.effective_base_url().ok(),
        }
    }

    async fn get_quote(
        &self,
        request: &types::OrderQuoteRequest,
    ) -> Result<types::OrderQuoteResponse, error::OrderbookError> {
        Self::get_quote(self, request).await
    }

    async fn send_order(
        &self,
        request: &types::OrderCreation,
    ) -> Result<CoreOrderUid, error::OrderbookError> {
        Self::send_order(self, request).await
    }

    async fn send_signed_order_cancellations(
        &self,
        request: &types::OrderCancellations,
    ) -> Result<(), error::OrderbookError> {
        Self::send_signed_order_cancellations(self, request).await
    }

    async fn get_order(
        &self,
        order_uid: &CoreOrderUid,
    ) -> Result<types::Order, error::OrderbookError> {
        Self::get_order(self, order_uid).await
    }

    async fn upload_app_data(
        &self,
        app_data_hash: &CoreAppDataHash,
        full_app_data: &str,
    ) -> Result<types::AppDataObject, error::OrderbookError> {
        Self::upload_app_data(self, app_data_hash, full_app_data).await
    }
}

pub use api::OrderBookApi;
pub use builder::{
    ChainIdSet, ChainIdUnset, EnvSet, EnvUnset, OrderBookApiBuilder, TransportSet, TransportUnset,
};
pub use error::OrderbookError;
pub use rejection::{OrderbookRejection, parse_rejection};
pub use request::{HttpMethod, OrderBookApiError, ResponseBody};
pub use transform::{calculate_total_fee, transform_order, transform_orders};
pub use types::{
    Address, Amount, ApiBaseUrls, ApiContext, ApiContextOverride, AppDataHash, AppDataObject,
    Auction, AuctionOrder, BuyTokenDestination, CompetitionAuction, CompetitionOrderStatus,
    CompetitionOrderStatusKind, CowEnv, ENVS_LIST, EVM_NATIVE_CURRENCY_ADDRESS, EcdsaSigningScheme,
    EnvBaseUrlOverrides, EthflowData, ExecutedAmounts, ExecutedProtocolFee, ExternalHostPolicy,
    FeePolicy, GetOrdersRequest, GetTradesRequest, HostPolicyError, InteractionData,
    NativePriceResponse, OnchainOrderData, Order, OrderCancellations, OrderClass, OrderCreation,
    OrderInteractions, OrderKind, OrderQuoteRequest, OrderQuoteResponse, OrderStatus, OrderUid,
    PriceQuality, Quote, QuoteAmountsAndCosts, QuoteData, QuoteSide, SellTokenSource,
    SigningScheme, SigningSchemeNotEcdsa, SolverCompetitionResponse, SolverExecution,
    SolverSettlement, StoredOrderQuote, SupportedChainId, TotalSurplus, Trade, TransactionHash,
};
