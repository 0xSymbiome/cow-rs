//! wasm-bindgen DTOs for the TypeScript-callable surface.

use serde::Serialize;
#[cfg(any(feature = "orderbook", feature = "trading"))]
use serde::de::DeserializeOwned;
#[cfg(any(feature = "orderbook", feature = "trading"))]
use serde_json::Value;
use wasm_bindgen::JsValue;

use crate::exports::errors::JsResultExt;
#[cfg(any(feature = "orderbook", feature = "trading"))]
use crate::exports::errors::WasmError;

mod app_data;
mod contracts;
mod core;
#[cfg(feature = "signing")]
mod events;
#[cfg(feature = "orderbook")]
mod order;
mod orderbook;
#[cfg(feature = "trading")]
mod quote;
mod signing;
#[cfg(feature = "subgraph")]
mod subgraph;
mod trading;
mod transport;

#[cfg(feature = "app-data")]
pub use self::app_data::{AppDataDocDto, AppDataDocInput, AppDataInfoDto, ValidationResultDto};
#[cfg(any(feature = "cancellation", feature = "trading"))]
pub use self::contracts::TransactionRequestDto;
#[cfg(feature = "trading")]
pub use self::contracts::{BuiltSellNativeCurrencyTxDto, ContractCallDto};
pub use self::contracts::{DeploymentAddressesDto, WrappedNativeTokenDto};
pub use self::core::{OrderInput, OrderKindDto, TokenBalanceDto};
#[cfg(feature = "signing")]
pub use self::events::{EthFlowEventDto, EventLogInput, SettlementEventDto};
#[cfg(feature = "orderbook")]
pub use self::order::OrderDto;
#[cfg(feature = "orderbook")]
pub use self::orderbook::{
    AppDataObjectDto, CompetitionAuctionDto, CompetitionOrderStatusDto,
    CompetitionOrderStatusKindDto, ExecutedAmountsDto, ExecutedProtocolFeeDto,
    NativePriceResponseDto, OrderCreationInput, OrderQuoteRequestInput, OrderQuoteResponseDto,
    SigningSchemeDto, SolverCompetitionOrderDto, SolverCompetitionResponseDto, SolverExecutionDto,
    SolverSettlementDto, TotalSurplusDto, TradeDto,
};
#[cfg(feature = "orderbook")]
pub use self::orderbook::{PaginationOptions, TradesQueryInput};
#[cfg(feature = "trading")]
pub use self::quote::QuoteResultsDto;
#[cfg(any(feature = "cancellation", feature = "orderbook"))]
pub use self::signing::SignedCancellationsInput;
pub use self::signing::{
    CowEip1271SignRequest, Eip1193Request, GeneratedOrderUidDto, SignedOrderDto,
    TypedDataDomainDto, TypedDataEnvelopeDto, TypedDataFieldDto,
};
#[cfg(feature = "subgraph")]
pub use self::subgraph::SubgraphQueryInput;
#[cfg(feature = "cancellation")]
pub use self::trading::OrderTraderParametersInput;
#[cfg(feature = "trading")]
pub use self::trading::{
    AllowanceParametersInput, ApprovalParametersInput, LimitTradeParametersInput, OrderDataDto,
    OrderPostingResultDto, PartnerFeeInput, PartnerFeePolicyInput, SwapParametersInput,
};
pub use self::transport::{CowFetchRequest, CowFetchResponse};
#[cfg(feature = "transport-policy")]
pub use self::transport::{
    JitterStrategyConfig, LimiterScopeConfig, RequestRateLimiterConfig, RetryPolicyConfig,
    TransportPolicyConfig,
};

#[cfg(any(
    feature = "signing",
    feature = "orderbook",
    feature = "trading",
    feature = "subgraph"
))]
pub(crate) use self::core::{parse_chain, parse_order, parse_owner};
#[cfg(feature = "orderbook")]
pub(crate) use self::orderbook::{ecdsa_signing_scheme, orderbook_signing_scheme};
#[cfg(feature = "signing")]
pub(crate) use self::signing::typed_data_json;
#[cfg(feature = "transport-policy")]
pub(crate) use self::transport::transport_policy_from_config;

pub(crate) fn to_js_value<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value.serialize(&serializer).map_js()
}

#[cfg(any(feature = "orderbook", feature = "trading"))]
pub(crate) fn from_json_value<T: DeserializeOwned>(
    field: &'static str,
    value: Value,
) -> Result<T, JsValue> {
    serde_json::from_value(value)
        .map_err(|error| WasmError::invalid(field, error.to_string()).into_js())
}
