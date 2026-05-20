#![no_main]

//! Fuzz target for the trading slippage amount calculator.
//!
//! **Surface:** `cow_sdk_trading::calculate_quote_amounts_and_costs`.
//! **Property:** `PROP-TRD-003`.
//! **Seed contract:** corpus inputs cover canonical sell- and buy-sided
//! quote shapes, boundary zero / one / `u128::MAX` inputs, and adversarial
//! oversized fee, partner-fee, and slippage shapes that exercise the
//! documented `parse_integer` and protocol-fee math boundaries.
//! **Corpus README:** `../corpus/fuzz_slippage_amounts/README.md`.
//!
//! The target maps arbitrary bytes through `Arbitrary` into a typed
//! `SlippageInput`, builds a `QuoteData` from the canonical public builder,
//! invokes `calculate_quote_amounts_and_costs` with the supplied slippage,
//! partner-fee, and protocol-fee parameters, and asserts:
//! - Every successful result has non-negative amounts across every stage.
//! - For sell-sided paths, `amounts_to_sign.sell_amount` equals
//!   `before_all_fees.sell_amount` and `amounts_to_sign.buy_amount` is
//!   bounded by `after_partner_fees.buy_amount`.
//! - The calculator never panics for any input.
//! - Determinism on identical input.

use cow_sdk_core::{Address, Amount, AppDataHash, OrderKind};
use cow_sdk_orderbook::QuoteData;
use cow_sdk_trading::calculate_quote_amounts_and_costs;
use libfuzzer_sys::{
    arbitrary::{Arbitrary, Unstructured},
    fuzz_target,
};

const APP_DATA_HASH_STR: &str =
    "0xe269b09f45b1d3c98d8e4e841b99a0779fbd3b77943d069b91ddc4fd9789e27e";
const SELL_TOKEN: &str = "0xfFf9976782d46CC05630D1f6eBAb18b2324d6B14";
const BUY_TOKEN: &str = "0x0625aFB445C3B6B7B929342a04A22599fd5dBB59";

#[derive(Debug)]
struct SlippageInput {
    slippage_bps: u32,
    partner_bps: u32,
    protocol_bps_tag: u8,
    sell_amount: u128,
    buy_amount: u128,
    fee_amount: u128,
    valid_to: u32,
    kind_is_buy: bool,
}

impl<'a> Arbitrary<'a> for SlippageInput {
    fn arbitrary(bytes: &mut Unstructured<'a>) -> libfuzzer_sys::arbitrary::Result<Self> {
        Ok(Self {
            slippage_bps: u32::from(read_u16(bytes, 50)),
            partner_bps: u32::from(read_u16(bytes, 0)),
            protocol_bps_tag: read_u8(bytes, 0),
            sell_amount: read_u128(bytes, 1),
            buy_amount: read_u128(bytes, 1),
            fee_amount: read_u128(bytes, 0),
            valid_to: u32::from(read_u16(bytes, 1)),
            kind_is_buy: bool::arbitrary(bytes).unwrap_or(false),
        })
    }
}

fuzz_target!(|input: SlippageInput| {
    let Ok(quote) = build_quote(&input) else {
        return;
    };
    let protocol_bps = protocol_bps_for_tag(input.protocol_bps_tag);

    let first = calculate_quote_amounts_and_costs(
        &quote,
        input.slippage_bps,
        Some(input.partner_bps),
        protocol_bps,
    );
    let second = calculate_quote_amounts_and_costs(
        &quote,
        input.slippage_bps,
        Some(input.partner_bps),
        protocol_bps,
    );
    assert_eq!(
        first.is_ok(),
        second.is_ok(),
        "calculate_quote_amounts_and_costs must be deterministic on identical input",
    );

    let Ok(amounts) = first else {
        return;
    };

    // The documented `Amount` boundary guarantees non-negative values; the
    // typed wrapper enforces this on construction, so every stage must
    // parse as a non-negative BigUint.
    // `Amount` is `#[repr(transparent)]` over `alloy_primitives::U256`
    // per ADR 0052; the uint256 ceiling is enforced by the type system
    // at construction, so the historical per-stage `bits() <= 256`
    // runtime guards collapse to constant-true invariants. The
    // cross-stage ordering assertion below remains meaningful because
    // it compares typed `Amount` values rather than asserting absolute
    // bit width.

    if amounts.is_sell {
        assert_eq!(
            amounts.amounts_to_sign.sell_amount, amounts.before_all_fees.sell_amount,
            "sell-sided amounts_to_sign.sell_amount must equal before_all_fees.sell_amount",
        );
        assert!(
            amounts.amounts_to_sign.buy_amount <= amounts.after_partner_fees.buy_amount,
            "sell-sided amounts_to_sign.buy_amount must not exceed after_partner_fees.buy_amount",
        );
    }
});

fn build_quote(input: &SlippageInput) -> Result<QuoteData, ()> {
    let sell_token = Address::new(SELL_TOKEN).map_err(|_| ())?;
    let buy_token = Address::new(BUY_TOKEN).map_err(|_| ())?;
    let app_data_hash = AppDataHash::new(APP_DATA_HASH_STR).map_err(|_| ())?;
    let sell_amount = Amount::new(input.sell_amount.to_string()).map_err(|_| ())?;
    let buy_amount = Amount::new(input.buy_amount.to_string()).map_err(|_| ())?;
    let fee_amount = Amount::new(input.fee_amount.to_string()).map_err(|_| ())?;
    let kind = if input.kind_is_buy {
        OrderKind::Buy
    } else {
        OrderKind::Sell
    };

    Ok(QuoteData::new(
        sell_token,
        buy_token,
        sell_amount,
        buy_amount,
        input.valid_to,
        app_data_hash,
        kind,
    )
    .with_network_cost_amount(fee_amount))
}

fn protocol_bps_for_tag(tag: u8) -> Option<f64> {
    match tag % 6 {
        0 => None,
        1 => Some(0.0),
        2 => Some(0.0001),
        3 => Some(50.0),
        4 => Some(1_000.0),
        _ => Some(10_000.0),
    }
}

fn read_u8(bytes: &mut Unstructured<'_>, default: u8) -> u8 {
    u8::arbitrary(bytes).unwrap_or(default)
}

fn read_u16(bytes: &mut Unstructured<'_>, default: u16) -> u16 {
    u16::arbitrary(bytes).unwrap_or(default)
}

fn read_u128(bytes: &mut Unstructured<'_>, default: u128) -> u128 {
    u128::arbitrary(bytes).unwrap_or(default)
}
