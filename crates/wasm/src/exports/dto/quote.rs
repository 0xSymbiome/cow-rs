//! Typed mirror of the native `cow_sdk_trading::QuoteResults` tree.
//!
//! `getQuote` serializes the native [`cow_sdk_trading::QuoteResults`] directly
//! and `postSwapOrderFromQuote` deserializes it directly, so these DTOs are
//! used only as tsify type annotations on the JavaScript boundary (the actual
//! ABI value stays a `JsValue` that round-trips through the native serde
//! representation). Each struct/enum therefore mirrors the native serde shape
//! exactly; a field-name or optionality drift would make the generated
//! `.d.ts` describe a shape the runtime does not produce.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::{
    OrderDataDto, OrderKindDto, OrderQuoteResponseDto, TokenBalanceDto, TypedDataEnvelopeDto,
};

/// Sell/buy amount pair at one quote stage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct AmountsDto {
    /// Sell-side amount.
    pub sell_amount: String,
    /// Buy-side amount.
    pub buy_amount: String,
}

/// Network-fee amounts expressed in both quote currencies.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFeeDto {
    /// Network fee expressed in sell-token units.
    pub amount_in_sell_currency: String,
    /// Network fee expressed in buy-token units.
    pub amount_in_buy_currency: String,
}

/// Fee component represented by an amount and a basis-point value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct FeeComponentDto {
    /// Fee amount.
    pub amount: String,
    /// Fee in basis points.
    pub bps: u32,
}

/// Full quote cost breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
#[allow(
    clippy::struct_field_names,
    reason = "the field names mirror the native Costs serde shape exactly"
)]
pub struct CostsDto {
    /// Network-fee component.
    pub network_fee: NetworkFeeDto,
    /// Partner-fee component.
    pub partner_fee: FeeComponentDto,
    /// Protocol-fee component.
    pub protocol_fee: FeeComponentDto,
}

/// Stepwise quote amounts and cost components across the quote lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct QuoteAmountsAndCostsDto {
    /// Whether the source quote was sell-sided.
    pub is_sell: bool,
    /// Cost breakdown for the quote.
    pub costs: CostsDto,
    /// Amounts before all fees.
    pub before_all_fees: AmountsDto,
    /// Amounts before network costs.
    pub before_network_costs: AmountsDto,
    /// Amounts after protocol fees.
    pub after_protocol_fees: AmountsDto,
    /// Amounts after network costs.
    pub after_network_costs: AmountsDto,
    /// Amounts after partner fees.
    pub after_partner_fees: AmountsDto,
    /// Amounts after slippage.
    pub after_slippage: AmountsDto,
    /// Amounts that should be signed.
    pub amounts_to_sign: AmountsDto,
}

/// `CoW` Protocol environment, mirroring `cow_sdk_core::CowEnv`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum CowEnvDto {
    /// Production endpoints and deployments.
    Prod,
    /// Staging endpoints and deployments.
    Staging,
}

/// One typed partner-fee policy object, mirroring
/// `cow_sdk_app_data::PartnerFeePolicy`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(untagged)]
#[non_exhaustive]
pub enum PartnerFeePolicyDto {
    /// Fee paid from traded volume.
    Volume {
        /// Fee paid in basis points of volume.
        #[serde(rename = "volumeBps")]
        volume_bps: u16,
        /// Recipient of the partner fee.
        recipient: String,
    },
    /// Fee paid from surplus, capped by volume.
    Surplus {
        /// Fee paid in basis points of surplus.
        #[serde(rename = "surplusBps")]
        surplus_bps: u16,
        /// Maximum fee paid in basis points of volume.
        #[serde(rename = "maxVolumeBps")]
        max_volume_bps: u16,
        /// Recipient of the partner fee.
        recipient: String,
    },
    /// Fee paid from price improvement, capped by volume.
    PriceImprovement {
        /// Fee paid in basis points of price improvement.
        #[serde(rename = "priceImprovementBps")]
        price_improvement_bps: u16,
        /// Maximum fee paid in basis points of volume.
        #[serde(rename = "maxVolumeBps")]
        max_volume_bps: u16,
        /// Recipient of the partner fee.
        recipient: String,
    },
}

/// Partner-fee metadata, mirroring `cow_sdk_app_data::PartnerFee`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(untagged)]
#[non_exhaustive]
pub enum PartnerFeeDto {
    /// Single fee policy object.
    Single(PartnerFeePolicyDto),
    /// Ordered fee policy list.
    Multiple(Vec<PartnerFeePolicyDto>),
}

/// Effective trade parameters after SDK defaults and advanced settings were
/// applied, mirroring `cow_sdk_trading::TradeParameters`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TradeParametersDto {
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
    pub env: Option<CowEnvDto>,
    /// Optional settlement-contract overrides keyed by chain id. Typed as
    /// `Record` rather than `Map` for runtime alignment (the wire form is a
    /// plain JSON object).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[tsify(optional, type = "Record<string, string>")]
    pub settlement_contract_override: Option<BTreeMap<u64, String>>,
    /// Optional `EthFlow`-contract overrides keyed by chain id.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[tsify(optional, type = "Record<string, string>")]
    pub eth_flow_contract_override: Option<BTreeMap<u64, String>>,
    /// Whether partial fills are allowed.
    pub partially_fillable: bool,
    /// Sell-token balance source.
    pub sell_token_balance: TokenBalanceDto,
    /// Buy-token balance destination.
    pub buy_token_balance: TokenBalanceDto,
    /// Optional explicit slippage tolerance in basis points.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    /// Optional receiver override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Optional relative validity duration in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    /// Optional absolute UNIX expiry timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    /// Optional partner-fee metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner_fee: Option<PartnerFeeDto>,
}

/// Originating orderbook runtime binding captured by the quote flow, mirroring
/// `cow_sdk_orderbook::OrderbookRuntimeBinding`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderBookRuntimeBindingDto {
    /// Chain id fixed by the orderbook client.
    pub chain_id: u64,
    /// Environment fixed by the orderbook client.
    pub env: CowEnvDto,
    /// Resolved base URL used by the orderbook client when available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolved_base_url: Option<String>,
}

/// App-data document, serialized payload, and digest used by the quote flow,
/// mirroring `cow_sdk_trading::TradingAppDataInfo`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct TradingAppDataInfoDto {
    /// Parsed app-data document (arbitrary JSON).
    pub doc: Value,
    /// Canonically serialized app-data payload.
    pub full_app_data: String,
    /// Keccak-256 digest used in protocol order payloads.
    pub app_data_keccak256: String,
}

/// Fully resolved quote result produced by trading quote helpers, mirroring
/// `cow_sdk_trading::QuoteResults`.
///
/// Returned by `getQuote` and accepted by `postSwapOrderFromQuote` unchanged.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResultsDto {
    /// Effective trade parameters after SDK defaults and advanced settings.
    pub trade_parameters: TradeParametersDto,
    /// Suggested slippage in basis points after resolution.
    pub suggested_slippage_bps: u32,
    /// Fee and amount breakdown derived from the orderbook quote.
    pub amounts_and_costs: QuoteAmountsAndCostsDto,
    /// Unsigned order payload produced for signing or on-chain submission.
    pub order_to_sign: OrderDataDto,
    /// Raw orderbook quote response.
    pub quote_response: OrderQuoteResponseDto,
    /// App-data document, serialized payload, and digest used by the quote flow.
    pub app_data_info: TradingAppDataInfoDto,
    /// Originating orderbook runtime binding captured by the quote flow.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub orderbook_binding: Option<OrderBookRuntimeBindingDto>,
    /// Typed order-facing EIP-712 envelope kept for consumers.
    pub order_typed_data: TypedDataEnvelopeDto,
}
