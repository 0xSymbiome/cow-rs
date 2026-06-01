use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
#[cfg(feature = "trading")]
use serde_json::Value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

#[cfg(feature = "trading")]
use super::{OrderKindDto, SigningSchemeDto, TokenBalanceDto};
#[cfg(feature = "trading")]
use crate::exports::errors::WasmError;

/// Unsigned order payload (`cow_sdk_core::OrderData`) returned by managed
/// trading flows.
///
/// Mirrors the signed-order field set. Unlike [`OrderInput`], the `receiver`
/// is always a concrete address because managed flows resolve it before
/// signing, so every field is present on the wire.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderDataDto {
    /// Sell-token address.
    pub sell_token: String,
    /// Buy-token address.
    pub buy_token: String,
    /// Receiver of the bought tokens.
    pub receiver: String,
    /// Sell amount.
    pub sell_amount: String,
    /// Buy amount.
    pub buy_amount: String,
    /// Valid-to timestamp.
    pub valid_to: u32,
    /// App-data hash.
    pub app_data: String,
    /// Fee amount.
    pub fee_amount: String,
    /// Order side.
    pub kind: OrderKindDto,
    /// Partial-fill flag.
    pub partially_fillable: bool,
    /// Sell balance source.
    pub sell_token_balance: TokenBalanceDto,
    /// Buy balance destination.
    pub buy_token_balance: TokenBalanceDto,
}

/// Result returned by a managed trading submission
/// (`cow_sdk_trading::OrderPostingResult`).
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderPostingResultDto {
    /// Final order UID.
    pub order_id: String,
    /// Transaction hash when the flow submitted an on-chain transaction directly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<String>,
    /// Signature scheme used for the posted order.
    pub signing_scheme: SigningSchemeDto,
    /// Signature payload sent to the orderbook, or an empty string for
    /// transaction-only flows.
    pub signature: String,
    /// Unsigned order payload used for signing or transaction generation.
    pub order_to_sign: OrderDataDto,
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
    /// Buy-token address.
    pub buy_token: String,
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
    /// Buy-token address.
    pub buy_token: String,
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
    ///
    /// Typed as `Record` rather than `Map` because the runtime
    /// serializer emits a plain JavaScript object for `BTreeMap`
    /// fields; the override aligns the declaration with the runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[tsify(optional, type = "Record<string, string>")]
    pub settlement_contract_override: Option<BTreeMap<u64, String>>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    ///
    /// Typed as `Record` rather than `Map` for the same runtime
    /// alignment reason as `settlement_contract_override`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[tsify(optional, type = "Record<string, string>")]
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
    ///
    /// Typed as `Record` rather than `Map` because the runtime
    /// serializer emits a plain JavaScript object for `BTreeMap`
    /// fields; the override aligns the declaration with the runtime.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[tsify(optional, type = "Record<string, string>")]
    pub settlement_contract_override: Option<BTreeMap<u64, String>>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    ///
    /// Typed as `Record` rather than `Map` for the same runtime
    /// alignment reason as `settlement_contract_override`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[tsify(optional, type = "Record<string, string>")]
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
