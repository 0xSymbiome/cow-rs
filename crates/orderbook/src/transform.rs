use cow_sdk_core::{Amount, NATIVE_CURRENCY_ADDRESS};

use crate::types::Order;

/// Normalizes an orderbook order response into the crate's stable DTO contract.
///
/// Sets `total_fee` to the canonical executed-fee component (ADR 0021) — the
/// typed [`Order::executed_fee`], never folding the legacy
/// [`Order::executed_fee_amount`] — and updates `EthFlow` orders so the
/// user-visible owner, validity, and native token address match the effective
/// order semantics exposed by the orderbook. Infallible: `executed_fee` is
/// already a validated [`Amount`], so there is no wire string left to reject.
#[must_use]
pub fn transform_order(mut order: Order) -> Order {
    order.total_fee = order.executed_fee.unwrap_or(Amount::ZERO);

    if let Some(ethflow_data) = &order.ethflow_data {
        order.valid_to = ethflow_data.user_valid_to;
        if let Some(onchain_user) = &order.onchain_user {
            order.owner = *onchain_user;
        }
        order.sell_token = NATIVE_CURRENCY_ADDRESS;
    }

    order
}

/// Applies [`transform_order`] to every order in the provided response list.
#[must_use]
pub fn transform_orders(orders: Vec<Order>) -> Vec<Order> {
    orders.into_iter().map(transform_order).collect()
}
