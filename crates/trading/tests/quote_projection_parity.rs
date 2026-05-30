//! Locks the projection that derives the signable order amounts from a
//! `/quote` response.
//!
//! The orderbook returns a sell order's `sellAmount` net of the network fee
//! and a buy order's `sellAmount` excluding it. The signed sell amount
//! therefore restores the network fee on a sell order (the settlement
//! contract deducts it on-chain) and carries it on top on a buy order. These
//! vectors use no protocol fee, no partner fee, and no slippage so the
//! network-cost handling is verified in isolation.

mod common;

use cow_sdk_core::{Amount, OrderKind};
use cow_sdk_orderbook::QuoteData;
use cow_sdk_trading::calculate_quote_amounts_and_costs;

use crate::common::{COW, WETH, address, app_data_hash};

fn amount(value: &str) -> Amount {
    Amount::new(value).expect("test amount literal must be valid")
}

fn quote(kind: OrderKind, sell: &str, buy: &str, network_cost: &str) -> QuoteData {
    QuoteData::new(
        address(WETH),
        address(COW),
        amount(sell),
        amount(buy),
        1_700_000_000,
        app_data_hash(),
        kind,
    )
    .with_network_cost_amount(amount(network_cost))
}

#[test]
fn sell_signable_amounts_fold_network_cost_into_sell() {
    // SELL: response sell/buy are after network cost. With no other fees and no
    // slippage, the signed sell amount adds the network cost back (the
    // settlement contract deducts it on-chain) and the signed buy amount is the
    // quoted buy amount.
    let result = calculate_quote_amounts_and_costs(
        &quote(OrderKind::Sell, "1000", "2000", "50"),
        0,
        None,
        None,
    )
    .expect("sell projection must succeed");

    assert_eq!(result.amounts_to_sign.sell_amount, amount("1050"));
    assert_eq!(result.amounts_to_sign.buy_amount, amount("2000"));
    // Network fee expressed in both currencies: 50 sell-side, 2000*50/1000 buy-side.
    assert_eq!(
        result.costs.network_fee.amount_in_sell_currency,
        amount("50")
    );
    assert_eq!(
        result.costs.network_fee.amount_in_buy_currency,
        amount("100")
    );
}

#[test]
fn buy_signable_amounts_inflate_sell_by_network_cost() {
    // BUY: the buy amount is exact; the signed sell amount carries the network
    // cost on top, with no other fees and no slippage.
    let result = calculate_quote_amounts_and_costs(
        &quote(OrderKind::Buy, "1000", "2000", "50"),
        0,
        None,
        None,
    )
    .expect("buy projection must succeed");

    assert_eq!(result.amounts_to_sign.sell_amount, amount("1050"));
    assert_eq!(result.amounts_to_sign.buy_amount, amount("2000"));
}
