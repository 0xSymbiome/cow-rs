use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
#[cfg(feature = "trading")]
use serde_json::Value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[cfg(feature = "trading")]
use super::{OrderInput, OrderKindDto, TokenBalanceDto};
#[cfg(feature = "trading")]
use crate::exports::errors::WasmError;

/// Quote-response reference accepted by quote-derived posting helpers.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponseRefInput {
    /// Upstream quote id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<i64>,
}

/// Minimal quote-results payload accepted by `TradingClient.postSwapOrderFromQuote`.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResultsInput {
    /// Order returned by a previous quote response.
    pub order_to_sign: OrderInput,
    /// Upstream quote response reference.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_response: Option<QuoteResponseRefInput>,
    /// Direct quote id fallback.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
}

#[cfg(feature = "trading")]
impl QuoteResultsInput {
    /// Returns the quote id supplied by either supported input shape.
    #[must_use]
    pub fn quote_id(&self) -> Option<i64> {
        self.quote_response
            .as_ref()
            .and_then(|response| response.id)
            .or(self.quote_id)
    }
}

/// Partner-fee policy input for trading swap parameters.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct PartnerFeePolicyInput {
    /// Volume fee in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub volume_bps: Option<u16>,
    /// Surplus fee in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub surplus_bps: Option<u16>,
    /// Price-improvement fee in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_improvement_bps: Option<u16>,
    /// Maximum volume fee in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_volume_bps: Option<u16>,
    /// Fee recipient address.
    pub recipient: String,
}

/// Partner-fee input accepted by trading swap parameters.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(untagged)]
pub enum PartnerFeeInput {
    /// Single partner-fee policy.
    Single(PartnerFeePolicyInput),
    /// Ordered partner-fee policies.
    Multiple(Vec<PartnerFeePolicyInput>),
}

/// Trading swap-parameter input.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct SwapParametersInput {
    /// Order side.
    pub kind: OrderKindDto,
    /// Optional owner override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Sell-token address.
    pub sell_token: String,
    /// Sell-token decimals.
    pub sell_token_decimals: u8,
    /// Buy-token address.
    pub buy_token: String,
    /// Buy-token decimals.
    pub buy_token_decimals: u8,
    /// Amount interpreted according to `kind`.
    pub amount: String,
    /// Optional environment override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<TokenBalanceDto>,
    /// Buy-token balance destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<TokenBalanceDto>,
    /// Optional slippage tolerance in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    /// Optional receiver override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Optional relative validity duration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    /// Optional absolute UNIX expiry timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    /// Optional partner-fee metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_fee: Option<PartnerFeeInput>,
}

#[cfg(feature = "trading")]
impl SwapParametersInput {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}

/// Limit-order parameters accepted by trading posting helpers.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct LimitTradeParametersInput {
    /// Order side.
    pub kind: OrderKindDto,
    /// Optional owner override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    /// Sell-token address.
    pub sell_token: String,
    /// Sell-token decimals.
    pub sell_token_decimals: u8,
    /// Buy-token address.
    pub buy_token: String,
    /// Buy-token decimals.
    pub buy_token_decimals: u8,
    /// Sell amount before transformations.
    pub sell_amount: String,
    /// Buy amount before transformations.
    pub buy_amount: String,
    /// Optional quote id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
    /// Optional environment override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    /// Optional settlement-contract overrides keyed by chain id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<BTreeMap<u64, String>>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<BTreeMap<u64, String>>,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<TokenBalanceDto>,
    /// Buy-token balance destination.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<TokenBalanceDto>,
    /// Optional slippage tolerance in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    /// Optional receiver override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Optional relative validity duration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    /// Optional absolute UNIX expiry timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    /// Optional partner-fee metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_fee: Option<PartnerFeeInput>,
}

#[cfg(feature = "trading")]
impl LimitTradeParametersInput {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}

/// Order transaction helper parameters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderTraderParametersInput {
    /// Target order UID.
    #[serde(rename = "orderUid")]
    pub order_uid: String,
    /// Optional chain-id override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u32>,
    /// Optional environment override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    /// Optional settlement-contract overrides keyed by chain id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<BTreeMap<u64, String>>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<BTreeMap<u64, String>>,
}

/// Allowance helper parameters.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AllowanceParametersInput {
    /// ERC-20 token address.
    pub token_address: String,
    /// Owner whose allowance should be inspected.
    pub owner: String,
    /// Optional chain-id override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<u32>,
    /// Optional environment override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub env: Option<String>,
    /// Optional vault-relayer deployment override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vault_relayer_override: Option<String>,
}

#[cfg(feature = "trading")]
impl AllowanceParametersInput {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}
