//! Boundary DTO shapes owned by the wasm leaf.
//!
//! These are the chain / deployment constructs, the app-data input shape, and
//! the per-domain input and projection shapes that the TypeScript-callable
//! surface accepts and returns but that have no native crate counterpart. The
//! shapes are defined here, outside the FFI-bearing `exports` tree, so the
//! host-safe `helpers` build them on the native target while the wasm-bindgen
//! `exports` re-export and surface them on `wasm32-unknown-unknown`. Each type
//! carries its TypeScript declaration derive (`tsify::Tsify`) gated to the
//! wasm-bindgen target, so the derive is an inert no-op on a native or WASI
//! build and only the JS realm generates the `.d.ts`.

#[cfg(target_arch = "wasm32")]
use serde::Serialize;
#[cfg(all(
    target_arch = "wasm32",
    any(feature = "orderbook", feature = "trading")
))]
use serde::de::DeserializeOwned;
#[cfg(all(
    target_arch = "wasm32",
    any(feature = "orderbook", feature = "trading")
))]
use serde_json::Value;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

#[cfg(target_arch = "wasm32")]
use crate::exports::errors::JsResultExt;
#[cfg(all(
    target_arch = "wasm32",
    any(
        feature = "signing",
        feature = "orderbook",
        feature = "trading",
        feature = "subgraph"
    )
))]
use crate::exports::errors::WasmError;
#[cfg(all(
    target_arch = "wasm32",
    any(
        feature = "signing",
        feature = "orderbook",
        feature = "trading",
        feature = "subgraph"
    )
))]
use crate::helpers::{self as pure};

pub mod app_data;
pub mod chains;
mod contracts;
#[cfg(feature = "signing")]
mod events;
mod orderbook;
mod signing;
mod trading;
mod transport;

// The order enums and the native `OrderData` are re-exported directly from their
// source crate, where each carries its own boundary `tsify` derive (gated to the
// wasm-bindgen npm target), so every `.d.ts` declaration is generated from a
// single definition. The chain / deployment constructs and the app-data input
// shape are defined in this leaf's own host-safe submodules. The per-domain
// submodules above hold only the hand-written input and projection shapes that
// have no native counterpart.

// Boundary order shapes and default-flavour constructs, always surfaced.
pub use cow_sdk_core::{BuyTokenDestination, OrderKind, SellTokenSource};

#[cfg(feature = "app-data")]
pub use self::app_data::AppDataDocument;
pub use self::app_data::{AppDataParams, ValidationResult};
pub use self::chains::{DeploymentAddresses, GeneratedOrderUid, WrappedNativeToken};
#[cfg(feature = "app-data")]
pub use cow_sdk_app_data::AppDataInfo;

#[cfg(feature = "trading")]
pub use self::contracts::BuiltSellNativeCurrencyTx;
#[cfg(feature = "trading")]
pub use cow_sdk_core::ContractCall;
#[cfg(any(feature = "cancellation", feature = "trading"))]
pub use cow_sdk_core::TransactionRequest;

#[cfg(feature = "signing")]
pub use self::events::{EthFlowEvent, EventLog, SettlementEvent};

#[cfg(feature = "orderbook")]
pub use self::orderbook::{
    OrderCreation, OrderQuoteRequest, PaginationOptions, GetTradesRequest,
};
#[cfg(all(target_arch = "wasm32", feature = "orderbook"))]
pub use cow_sdk_orderbook::{
    AppDataObject, CompetitionAuction, CompetitionOrderStatus, CompetitionOrderStatusKind,
    EthflowData, ExecutedAmounts, ExecutedProtocolFee, InteractionData, NativePriceResponse,
    OnchainOrderData, Order, OrderClass, OrderInteractions, OrderQuoteResponse, OrderStatus,
    QuoteData, SigningScheme, SolverCompetitionOrder, SolverCompetitionResponse, SolverExecution,
    SolverSettlement, StoredOrderQuote, TotalSurplus, Trade,
};

#[cfg(any(feature = "cancellation", feature = "orderbook"))]
pub use self::signing::SignedCancellations;
pub use self::signing::{CowEip1271SignRequest, SignedOrder};
#[cfg(all(target_arch = "wasm32", feature = "signing"))]
pub(crate) use self::signing::{envelope_callback_value, payload_to_envelope};

#[cfg(feature = "cancellation")]
pub use self::trading::OrderTraderParams;
#[cfg(feature = "trading")]
pub use self::trading::{AllowanceParams, ApprovalParams};
#[cfg(feature = "trading")]
pub use cow_sdk_app_data::{PartnerFee, PartnerFeePolicy};
#[cfg(feature = "trading")]
pub use cow_sdk_core::{
    Amounts, Costs, CowEnv, FeeComponent, NetworkFee, OrderData, QuoteAmountsAndCosts,
    TypedDataDomain, TypedDataEnvelope, TypedDataField,
};
#[cfg(all(target_arch = "wasm32", feature = "trading"))]
pub use cow_sdk_orderbook::OrderbookBinding;
#[cfg(all(target_arch = "wasm32", feature = "trading"))]
pub use cow_sdk_trading::{
    LimitTradeParams, OrderPostingResult, QuoteResults, TradeParams, TradingAppDataInfo,
};

pub use self::transport::{CowFetchRequest, CowFetchResponse};
#[cfg(feature = "transport-policy")]
pub use self::transport::{
    JitterStrategyConfig, LimiterScopeConfig, RequestRateLimiterConfig, RetryPolicyConfig,
    TransportPolicyConfig,
};

#[cfg(all(target_arch = "wasm32", feature = "orderbook"))]
pub(crate) use self::orderbook::{ecdsa_signing_scheme, orderbook_signing_scheme};
#[cfg(all(target_arch = "wasm32", feature = "transport-policy"))]
pub(crate) use self::transport::transport_policy_from_config;

#[cfg(target_arch = "wasm32")]
pub(crate) fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value.serialize(&serializer).map_js()
}

#[cfg(all(
    target_arch = "wasm32",
    any(feature = "orderbook", feature = "trading")
))]
pub(crate) fn from_json_value<T: DeserializeOwned>(
    field: &'static str,
    value: Value,
) -> Result<T, JsValue> {
    serde_json::from_value(value)
        .map_err(|error| WasmError::invalid(field, error.to_string()).into_js())
}

/// Resolves a numeric chain id into the supported-chain enum.
#[cfg(all(
    target_arch = "wasm32",
    any(
        feature = "signing",
        feature = "orderbook",
        feature = "trading",
        feature = "subgraph"
    )
))]
pub(crate) fn parse_chain(chain_id: u32) -> Result<cow_sdk_core::SupportedChainId, WasmError> {
    pure::chains::supported_chain(chain_id).map_err(WasmError::from)
}

/// Parses an owner address from its hex string.
#[cfg(all(
    target_arch = "wasm32",
    any(
        feature = "signing",
        feature = "orderbook",
        feature = "trading",
        feature = "subgraph"
    )
))]
pub(crate) fn parse_owner(owner: &str) -> Result<cow_sdk_core::Address, WasmError> {
    pure::dto::parse_address("owner", owner).map_err(WasmError::from)
}
