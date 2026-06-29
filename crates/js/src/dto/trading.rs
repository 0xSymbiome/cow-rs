#[cfg(feature = "cancellation")]
use std::collections::BTreeMap;

#[cfg(any(feature = "trading", feature = "cancellation"))]
use serde::{Deserialize, Serialize};
#[cfg(all(target_arch = "wasm32", feature = "trading"))]
use serde_json::Value;

#[cfg(all(target_arch = "wasm32", feature = "trading"))]
use crate::exports::errors::WasmError;

// The swap and limit trade requests are not hand-written boundary shapes: the
// wasm-bindgen and WebAssembly Component lanes pass the native
// `cow_sdk_trading::TradeParams` / `LimitTradeParams` directly. Those types carry
// their own `tsify` boundary derive and deserialize the camelCase wire shape
// (token addresses and amounts as strings, the partner fee as the native untagged
// `PartnerFee`), so this module holds only the trading helper inputs below: the
// cancellation, allowance, and approval parameter shapes, which have no native
// counterpart.
//
// The managed-submission result tree is likewise native: `OrderPostingResult`
// carries its own `tsify` boundary derive (gated to the wasm-bindgen npm target
// behind `ts-bindings`), so the `.d.ts` is generated from it directly and
// `orderToSign` resolves to the single native `OrderData` definition that
// `QuoteResults` already references.

/// Order transaction helper parameters.
#[cfg(feature = "cancellation")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(
        target_arch = "wasm32",
        target_os = "unknown",
        feature = "cancellation"
    ),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(
        target_arch = "wasm32",
        target_os = "unknown",
        feature = "cancellation"
    ),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct OrderTraderParams {
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
    #[cfg_attr(
        all(
            target_arch = "wasm32",
            target_os = "unknown",
            feature = "cancellation"
        ),
        tsify(optional, type = "Record<string, string>")
    )]
    pub settlement_contract_override: Option<BTreeMap<u64, String>>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    ///
    /// Typed as `Record` rather than `Map` for the same runtime
    /// alignment reason as `settlement_contract_override`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        all(
            target_arch = "wasm32",
            target_os = "unknown",
            feature = "cancellation"
        ),
        tsify(optional, type = "Record<string, string>")
    )]
    pub eth_flow_contract_override: Option<BTreeMap<u64, String>>,
}

/// Allowance helper parameters.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "trading"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "trading"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct AllowanceParams {
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

#[cfg(all(target_arch = "wasm32", feature = "trading"))]
impl AllowanceParams {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}

/// Approval-transaction helper parameters.
///
/// The chain and environment are taken from the `TradingClient`, matching the
/// other transaction builders; only the token, amount, and an optional
/// vault-relayer deployment override are supplied per call.
#[cfg(feature = "trading")]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "trading"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "trading"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalParams {
    /// ERC-20 token address to approve.
    pub token_address: String,
    /// Approval amount as a base-unit decimal string.
    pub amount: String,
    /// Optional vault-relayer deployment override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vault_relayer_override: Option<String>,
}

#[cfg(all(target_arch = "wasm32", feature = "trading"))]
impl ApprovalParams {
    pub(crate) fn into_value(self) -> Result<Value, WasmError> {
        serde_json::to_value(self).map_err(WasmError::from)
    }
}
