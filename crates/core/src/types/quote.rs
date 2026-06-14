use serde::{Deserialize, Serialize};

use super::amount::Amount;

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
    #[expect(
        clippy::too_many_arguments,
        reason = "constructor mirrors the public field set so callers can migrate off struct-literal construction without losing explicit control over any wire field"
    )]
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
}
