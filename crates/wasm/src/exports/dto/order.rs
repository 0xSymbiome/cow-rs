//! Typed mirror of the native `cow_sdk_orderbook::Order` read response.
//!
//! `getOrder`/`getOrders` serialize the native `Order` directly, so these DTOs
//! are tsify type annotations only (the ABI value stays a `JsValue` that
//! round-trips through the native serde representation). Each struct/enum
//! mirrors the native serde shape exactly; the native `Order` already folds in
//! the normalized `total_fee`, so it is the enriched order shape.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tsify::Tsify;
use wasm_bindgen::prelude::*;

use super::{OrderKindDto, SigningSchemeDto, TokenBalanceDto};

/// Order class surfaced by the orderbook API, mirroring
/// `cow_sdk_orderbook::OrderClass`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "lowercase")]
#[non_exhaustive]
pub enum OrderClassDto {
    /// Market order.
    Market,
    /// Limit order.
    Limit,
    /// Liquidity order.
    Liquidity,
}

/// Order lifecycle status returned by the orderbook API, mirroring
/// `cow_sdk_orderbook::OrderStatus`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum OrderStatusDto {
    /// Waiting for a pre-signature to become valid.
    PresignaturePending,
    /// Open and fillable.
    Open,
    /// Fully or terminally fulfilled.
    Fulfilled,
    /// Cancelled by the owner or protocol.
    Cancelled,
    /// Expired because `validTo` has passed.
    Expired,
}

/// `EthFlow`-specific order metadata, mirroring
/// `cow_sdk_orderbook::EthflowData`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct EthflowDataDto {
    /// Transaction in which the order was refunded, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refund_tx_hash: Option<String>,
    /// User-facing validity timestamp for the `EthFlow` order.
    pub user_valid_to: u32,
}

/// On-chain placement metadata, mirroring
/// `cow_sdk_orderbook::OnchainOrderData`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OnchainOrderDataDto {
    /// Sender address associated with the on-chain placement.
    pub sender: String,
    /// Placement error emitted by services, when on-chain placement failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub placement_error: Option<String>,
}

/// A single pre/post interaction attached to an order, mirroring
/// `cow_sdk_orderbook::InteractionData`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct InteractionDataDto {
    /// Contract address targeted by the interaction.
    pub target: String,
    /// Native token value sent with the interaction.
    pub value: String,
    /// Hex-encoded calldata forwarded to `target`.
    pub call_data: String,
}

/// Pre/post interactions associated with an order, mirroring
/// `cow_sdk_orderbook::OrderInteractions`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderInteractionsDto {
    /// Interactions executed before the order's trade.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre: Option<Vec<InteractionDataDto>>,
    /// Interactions executed after the order's trade.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub post: Option<Vec<InteractionDataDto>>,
}

/// Stored quote metadata for quote-linked orders, mirroring
/// `cow_sdk_orderbook::StoredOrderQuote`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct StoredOrderQuoteDto {
    /// Estimated gas units required to execute the quoted trade.
    pub gas_amount: String,
    /// Estimated gas price at quote time, in wei per gas unit.
    pub gas_price: String,
    /// Sell-token price in native-token atoms per sell-token atom.
    pub sell_token_price: String,
    /// Quoted sell amount.
    pub sell_amount: String,
    /// Quoted buy amount.
    pub buy_amount: String,
    /// Estimated network fee in sell-token atoms.
    pub fee_amount: String,
    /// Solver address that provided the quote.
    pub solver: String,
    /// Whether the quote was verified through simulation.
    pub verified: bool,
    /// Additional services-provided quote metadata, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
}

/// Order returned by the orderbook order endpoints, mirroring
/// `cow_sdk_orderbook::Order` (the enriched order shape, with the normalized
/// `totalFee` folded in).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(rename_all = "camelCase")]
pub struct OrderDto {
    /// Sell-token address.
    pub sell_token: String,
    /// Buy-token address.
    pub buy_token: String,
    /// Optional receiver override.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub receiver: Option<String>,
    /// Sell amount in the upstream decimal-string wire shape.
    pub sell_amount: String,
    /// Buy amount in the upstream decimal-string wire shape.
    pub buy_amount: String,
    /// Absolute UNIX expiry timestamp.
    pub valid_to: u32,
    /// App-data hash attached to the order.
    pub app_data: String,
    /// Optional app-data hash echoed for debugging by the orderbook.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<String>,
    /// Order-level fee echoed on the orderbook response; always `"0"` in
    /// practice because services rejects non-zero order-level fees.
    pub fee_amount: String,
    /// Strict balance-check flag, present only when the order was created with
    /// it set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_balance_check: Option<bool>,
    /// Order kind.
    pub kind: OrderKindDto,
    /// Whether partial fills are allowed.
    pub partially_fillable: bool,
    /// Sell-token balance source.
    pub sell_token_balance: TokenBalanceDto,
    /// Buy-token balance destination.
    pub buy_token_balance: TokenBalanceDto,
    /// Signature scheme used for `signature`.
    pub signing_scheme: SigningSchemeDto,
    /// Raw signature string.
    pub signature: String,
    /// Effective owner field returned by the API, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Quote id used when the order originated from a quote.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
    /// Order class.
    pub class: OrderClassDto,
    /// Canonical owner surfaced by the orderbook response.
    pub owner: String,
    /// Order UID.
    pub uid: String,
    /// Creation timestamp string returned by the API.
    #[serde(default)]
    pub creation_date: String,
    /// Executed sell amount.
    #[serde(default)]
    pub executed_sell_amount: String,
    /// Executed sell amount before fees.
    #[serde(default)]
    pub executed_sell_amount_before_fees: String,
    /// Executed buy amount.
    #[serde(default)]
    pub executed_buy_amount: String,
    /// Executed fee component, when provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed_fee: Option<String>,
    /// Deprecated legacy executed-fee value, present on older order payloads.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub executed_fee_amount: String,
    /// Token in which the executed fee was captured, when returned.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub executed_fee_token: Option<String>,
    /// Whether the order was invalidated by the protocol.
    #[serde(default)]
    pub invalidated: bool,
    /// Order lifecycle status.
    pub status: OrderStatusDto,
    /// Whether services classified the order as a liquidity order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_liquidity_order: Option<bool>,
    /// On-chain user for `EthFlow`-style orders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub onchain_user: Option<String>,
    /// `EthFlow`-specific metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ethflow_data: Option<EthflowDataDto>,
    /// On-chain placement metadata, when services returns it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub onchain_order_data: Option<OnchainOrderDataDto>,
    /// Full app-data payload, when services returns it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub full_app_data: Option<String>,
    /// Settlement contract address against which the order was signed.
    pub settlement_contract: String,
    /// Stored quote metadata for quote-linked orders.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quote: Option<StoredOrderQuoteDto>,
    /// Optional pre and post interactions associated with the order.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interactions: Option<OrderInteractionsDto>,
    /// Total fee normalized by the SDK transform layer.
    #[serde(default)]
    pub total_fee: String,
}
