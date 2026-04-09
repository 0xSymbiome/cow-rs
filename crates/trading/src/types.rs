use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use cow_sdk_app_data::{AppDataDoc, AppDataParams};
use cow_sdk_core::{
    Address, AddressPerChain, ApiContext, AppDataHash, CowEnv, OrderBalance, OrderKind, OrderUid,
    QuoteAmountsAndCosts, SupportedChainId, TransactionRequest, UnsignedOrder,
};
use cow_sdk_orderbook::{
    AppDataObject, Order, OrderBookApi, OrderCancellations, OrderCreation, OrderQuoteRequest,
    OrderQuoteResponse, OrderbookError, PriceQuality, SigningScheme,
};
use cow_sdk_signing::OrderTypedData;

use crate::TradingError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TraderParameters {
    pub chain_id: SupportedChainId,
    pub app_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartialTraderParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoterParameters {
    pub chain_id: SupportedChainId,
    pub app_code: String,
    pub account: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradeParameters {
    pub kind: OrderKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    pub sell_token: Address,
    pub sell_token_decimals: u8,
    pub buy_token: Address,
    pub buy_token_decimals: u8,
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
    #[serde(default)]
    pub partially_fillable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_fee: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LimitTradeParameters {
    pub kind: OrderKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    pub sell_token: Address,
    pub sell_token_decimals: u8,
    pub buy_token: Address,
    pub buy_token_decimals: u8,
    pub sell_amount: String,
    pub buy_amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
    #[serde(default)]
    pub partially_fillable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_fee: Option<Value>,
}

pub type LimitTradeParametersFromQuote = LimitTradeParameters;
pub type TradingTransactionParams = TransactionRequest;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlippageToleranceRequest {
    pub chain_id: SupportedChainId,
    pub sell_token: Address,
    pub buy_token: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlippageToleranceResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResults {
    pub trade_parameters: TradeParameters,
    pub suggested_slippage_bps: u32,
    pub amounts_and_costs: QuoteAmountsAndCosts<String>,
    pub order_to_sign: UnsignedOrder,
    pub quote_response: OrderQuoteResponse,
    pub app_data_info: TradingAppDataInfo,
    pub order_typed_data: OrderTypedData,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderPostingResult {
    pub order_id: OrderUid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    pub signing_scheme: SigningScheme,
    pub signature: String,
    pub order_to_sign: UnsignedOrder,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingAppDataInfo {
    pub doc: AppDataDoc,
    pub full_app_data: String,
    pub app_data_keccak256: AppDataHash,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequestOverride {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_quality: Option<PriceQuality>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_scheme: Option<SigningScheme>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onchain_order: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_gas_limit: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partially_fillable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<OrderBalance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<OrderBalance>,
}

#[derive(Clone, Default)]
pub struct PostTradeAdditionalParams {
    pub check_eth_flow_order_exists: Option<Arc<dyn EthFlowOrderExistsChecker>>,
    pub network_costs_amount: Option<String>,
    pub signing_scheme: Option<SigningScheme>,
    pub custom_eip1271_signature: Option<Arc<dyn Eip1271SignatureProvider>>,
    pub apply_costs_slippage_and_fees: Option<bool>,
}

#[derive(Clone, Default)]
pub struct SwapAdvancedSettings {
    pub quote_request: Option<QuoteRequestOverride>,
    pub app_data: Option<AppDataParams>,
    pub additional_params: Option<PostTradeAdditionalParams>,
    pub slippage_suggester: Option<Arc<dyn SlippageSuggestionProvider>>,
}

#[derive(Clone, Default)]
pub struct LimitOrderAdvancedSettings {
    pub quote_request: Option<QuoteRequestOverride>,
    pub app_data: Option<AppDataParams>,
    pub additional_params: Option<PostTradeAdditionalParams>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderTraderParameters {
    pub order_uid: OrderUid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowanceParameters {
    pub token_address: Address,
    pub owner: Address,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_relayer_address: Option<Address>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalParameters {
    pub token_address: Address,
    pub amount: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_relayer_address: Option<Address>,
}

#[derive(Clone, Default)]
pub struct TradingSdkOptions {
    pub order_book_api: Option<Arc<dyn OrderbookClient>>,
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait OrderbookClient: Send + Sync {
    fn context(&self) -> &ApiContext;

    async fn get_quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError>;

    async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError>;

    async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), OrderbookError>;

    async fn get_order(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError>;

    async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<AppDataObject, OrderbookError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait SlippageSuggestionProvider: Send + Sync {
    async fn get_slippage_suggestion(
        &self,
        request: SlippageToleranceRequest,
    ) -> Result<SlippageToleranceResponse, TradingError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait EthFlowOrderExistsChecker: Send + Sync {
    async fn order_exists(
        &self,
        order_id: &OrderUid,
        order_digest: &str,
    ) -> Result<bool, TradingError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
pub trait Eip1271SignatureProvider: Send + Sync {
    async fn sign(&self, order_to_sign: &UnsignedOrder) -> Result<String, TradingError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl OrderbookClient for OrderBookApi {
    fn context(&self) -> &ApiContext {
        self.context()
    }

    async fn get_quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError> {
        OrderBookApi::get_quote(self, request).await
    }

    async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError> {
        OrderBookApi::send_order(self, request).await
    }

    async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), OrderbookError> {
        OrderBookApi::send_signed_order_cancellations(self, request).await
    }

    async fn get_order(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        OrderBookApi::get_order(self, order_uid).await
    }

    async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<AppDataObject, OrderbookError> {
        OrderBookApi::upload_app_data(self, app_data_hash, full_app_data).await
    }
}
