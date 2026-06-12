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
use cow_sdk_trading::{calculate_quote_amounts_and_costs, sanitize_protocol_fee_bps};
use serde::Deserialize;

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

const COMPOSITION_FIXTURE: &str =
    include_str!("../../../parity/fixtures/trading/protocol_fee_partner_fee_composition.json");

#[derive(Deserialize)]
struct CompositionFixture {
    cases: Vec<CompositionCase>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CompositionCase {
    name: String,
    kind: String,
    sell_amount: String,
    buy_amount: String,
    network_cost_amount: String,
    partner_fee_volume_bps: u32,
    slippage_bps: u32,
    protocol_fee_bps: Option<String>,
    expected_signed_sell_amount: String,
    expected_signed_buy_amount: String,
}

#[test]
fn protocol_fee_partner_fee_composition_matches_upstream_goldens() {
    // Pins the signed amounts the quote engine derives when a protocol fee and a
    // partner fee compose, against the goldens transcribed from the upstream
    // posting test. The protocol fee enlarges the partner-fee base, so the
    // signed buy amount is strictly lower than the no-protocol-fee row.
    let fixture: CompositionFixture =
        serde_json::from_str(COMPOSITION_FIXTURE).expect("composition fixture must parse");
    assert!(
        !fixture.cases.is_empty(),
        "composition fixture must carry at least one case"
    );

    for case in &fixture.cases {
        let kind = match case.kind.as_str() {
            "sell" => OrderKind::Sell,
            "buy" => OrderKind::Buy,
            other => panic!("case {}: unsupported order kind `{other}`", case.name),
        };
        let quote = QuoteData::new(
            address(WETH),
            address(COW),
            amount(&case.sell_amount),
            amount(&case.buy_amount),
            1_700_000_000,
            app_data_hash(),
            kind,
        )
        .with_network_cost_amount(amount(&case.network_cost_amount));

        let result = calculate_quote_amounts_and_costs(
            &quote,
            case.slippage_bps,
            Some(case.partner_fee_volume_bps),
            sanitize_protocol_fee_bps(case.protocol_fee_bps.as_deref()),
        )
        .unwrap_or_else(|err| panic!("case {}: composition must succeed: {err}", case.name));

        assert_eq!(
            result.amounts_to_sign.sell_amount.to_string(),
            case.expected_signed_sell_amount,
            "case {}: signed sell amount",
            case.name
        );
        assert_eq!(
            result.amounts_to_sign.buy_amount.to_string(),
            case.expected_signed_buy_amount,
            "case {}: signed buy amount",
            case.name
        );
    }
}
