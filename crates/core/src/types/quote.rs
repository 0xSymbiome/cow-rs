use serde::{Deserialize, Serialize};

use super::{
    amount::Amount,
    identity::{Address, AppDataHash, OrderUid},
    order::OrderKind,
};
/// Canonical quote amount stage names used by [`QuoteAmountsAndCosts`].
pub const QUOTE_AMOUNT_STAGE_NAMES: [&str; 7] = [
    "beforeAllFees",
    "beforeNetworkCosts",
    "afterProtocolFees",
    "afterNetworkCosts",
    "afterPartnerFees",
    "afterSlippage",
    "amountsToSign",
];

/// User-domain quote request shape with validated quantities.
///
/// This is not the orderbook HTTP wire DTO. The orderbook crate keeps the upstream
/// string-based transport contract explicit.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteRequest {
    /// Quote side.
    pub kind: OrderKind,
    /// Optional sell token address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token: Option<Address>,
    /// Optional buy token address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token: Option<Address>,
    /// Optional receiver address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Optional order owner address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Optional sell amount input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount: Option<Amount>,
    /// Optional buy amount input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount: Option<Amount>,
    /// Optional explicit fee amount.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fee_amount: Option<Amount>,
    /// Optional app-data hash reference.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data_hash: Option<AppDataHash>,
    /// Optional raw app-data document payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_data: Option<String>,
    /// Optional order expiration timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
}

impl QuoteRequest {
    /// Creates a user-domain quote request from its optional input fields.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        kind: OrderKind,
        sell_token: Option<Address>,
        buy_token: Option<Address>,
        receiver: Option<Address>,
        from: Option<Address>,
        sell_amount: Option<Amount>,
        buy_amount: Option<Amount>,
        fee_amount: Option<Amount>,
        app_data_hash: Option<AppDataHash>,
        app_data: Option<String>,
        valid_to: Option<u32>,
    ) -> Self {
        Self {
            kind,
            sell_token,
            buy_token,
            receiver,
            from,
            sell_amount,
            buy_amount,
            fee_amount,
            app_data_hash,
            app_data,
            valid_to,
        }
    }
}

/// User-domain quote response with validated quantities.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QuoteResponse {
    /// Quote side.
    pub kind: OrderKind,
    /// Sell amount returned by the quote.
    pub sell_amount: Amount,
    /// Buy amount returned by the quote.
    pub buy_amount: Amount,
    /// Fee amount returned by the quote.
    pub fee_amount: Amount,
    /// Optional order UID when the quote is tied to a persisted order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order_uid: Option<OrderUid>,
    /// Optional price string from the upstream API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    /// Optional quote identifier from the upstream API.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<String>,
    /// Optional stepwise amounts-and-costs breakdown.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub amounts_and_costs: Option<QuoteAmountsAndCosts>,
}

impl QuoteResponse {
    /// Creates a user-domain quote response from the canonical amount fields.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        kind: OrderKind,
        sell_amount: Amount,
        buy_amount: Amount,
        fee_amount: Amount,
        order_uid: Option<OrderUid>,
        price: Option<String>,
        quote_id: Option<String>,
        amounts_and_costs: Option<QuoteAmountsAndCosts>,
    ) -> Self {
        Self {
            kind,
            sell_amount,
            buy_amount,
            fee_amount,
            order_uid,
            price,
            quote_id,
            amounts_and_costs,
        }
    }
}

/// Generic sell/buy amount pair.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Amounts<T> {
    /// Sell-side amount.
    pub sell_amount: T,
    /// Buy-side amount.
    pub buy_amount: T,
}

impl<T> Amounts<T> {
    /// Creates a sell/buy amount pair.
    #[inline]
    #[must_use]
    pub const fn new(sell_amount: T, buy_amount: T) -> Self {
        Self {
            sell_amount,
            buy_amount,
        }
    }
}

/// Network-fee amounts expressed in both quote currencies.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NetworkFee<T> {
    /// Network fee expressed in sell-token units.
    pub amount_in_sell_currency: T,
    /// Network fee expressed in buy-token units.
    pub amount_in_buy_currency: T,
}

impl<T> NetworkFee<T> {
    /// Creates network-fee amounts in both quote currencies.
    #[inline]
    #[must_use]
    pub const fn new(amount_in_sell_currency: T, amount_in_buy_currency: T) -> Self {
        Self {
            amount_in_sell_currency,
            amount_in_buy_currency,
        }
    }
}

/// Generic fee component represented by amount and basis points.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeeComponent<T> {
    /// Fee amount.
    pub amount: T,
    /// Fee in basis points.
    pub bps: u32,
}

impl<T> FeeComponent<T> {
    /// Creates a fee component from an amount and basis-point value.
    #[inline]
    #[must_use]
    pub const fn new(amount: T, bps: u32) -> Self {
        Self { amount, bps }
    }
}

/// Full quote cost breakdown.
#[non_exhaustive]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Costs<T> {
    /// Network fee component.
    pub network_fee: NetworkFee<T>,
    /// Partner fee component.
    pub partner_fee: FeeComponent<T>,
    /// Protocol fee component.
    pub protocol_fee: FeeComponent<T>,
}

impl<T> Costs<T> {
    /// Creates a full quote cost breakdown.
    #[inline]
    #[must_use]
    pub const fn new(
        network_fee: NetworkFee<T>,
        partner_fee: FeeComponent<T>,
        protocol_fee: FeeComponent<T>,
    ) -> Self {
        Self {
            network_fee,
            partner_fee,
            protocol_fee,
        }
    }
}

/// Stepwise quote amounts and cost components across the quote lifecycle.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteAmountsAndCosts<T = Amount> {
    /// Whether the source quote was sell-sided.
    pub is_sell: bool,
    /// Cost breakdown for the quote.
    pub costs: Costs<T>,
    /// Amounts before all fees.
    pub before_all_fees: Amounts<T>,
    /// Amounts before network costs.
    pub before_network_costs: Amounts<T>,
    /// Amounts after protocol fees.
    pub after_protocol_fees: Amounts<T>,
    /// Amounts after network costs.
    pub after_network_costs: Amounts<T>,
    /// Amounts after partner fees.
    pub after_partner_fees: Amounts<T>,
    /// Amounts after slippage.
    pub after_slippage: Amounts<T>,
    /// Amounts that should be signed.
    pub amounts_to_sign: Amounts<T>,
}

impl<T> QuoteAmountsAndCosts<T> {
    /// Creates a quote-stage breakdown from its individual stage amounts.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        is_sell: bool,
        costs: Costs<T>,
        before_all_fees: Amounts<T>,
        before_network_costs: Amounts<T>,
        after_protocol_fees: Amounts<T>,
        after_network_costs: Amounts<T>,
        after_partner_fees: Amounts<T>,
        after_slippage: Amounts<T>,
        amounts_to_sign: Amounts<T>,
    ) -> Self {
        Self {
            is_sell,
            costs,
            before_all_fees,
            before_network_costs,
            after_protocol_fees,
            after_network_costs,
            after_partner_fees,
            after_slippage,
            amounts_to_sign,
        }
    }

    /// Returns the canonical stage ordering for quote amount breakdowns.
    #[must_use]
    pub const fn stage_names() -> &'static [&'static str; QUOTE_AMOUNT_STAGE_NAMES.len()] {
        &QUOTE_AMOUNT_STAGE_NAMES
    }
}
