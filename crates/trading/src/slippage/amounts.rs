use num_bigint::BigInt;

use cow_sdk_core::{Amount, OrderKind, QuoteAmountsAndCosts};
use cow_sdk_orderbook::QuoteData;

use crate::TradingError;

use super::breakdown::{QuoteFeeBreakdown, get_protocol_fee_amount};
use super::{GAS_MARGIN_PERCENT, ONE_HUNDRED_BPS};

/// Derives the signed and intermediate quote amounts after protocol, network, partner, and
/// slippage adjustments.
///
/// This function keeps the upstream quote strings as integer math. Partner-fee and protocol-fee
/// adjustments use integer division, so fractional remainder is truncated toward zero. Slippage
/// amounts are derived in basis points and also truncate toward zero before the final typed
/// amounts are materialized.
///
/// # Errors
///
/// Returns an error when quote numeric fields are malformed, when the quoted sell amount is zero
/// or negative, when protocol-fee math overflows the supported typed amount surface, or when any
/// derived typed amount cannot be represented as a `cow_sdk_core::Amount`.
pub fn calculate_quote_amounts_and_costs(
    quote: &QuoteData,
    slippage_percent_bps: u32,
    partner_fee_bps: Option<u32>,
    protocol_fee_bps: Option<f64>,
) -> Result<QuoteAmountsAndCosts, TradingError> {
    let is_sell = quote.kind == OrderKind::Sell;
    let sell_amount = parse_integer("sellAmount", &quote.sell_amount.to_string())?;
    let buy_amount = parse_integer("buyAmount", &quote.buy_amount.to_string())?;
    let network_cost_amount = parse_integer("feeAmount", &quote.network_cost_amount().to_string())?;

    if sell_amount <= BigInt::from(0) {
        return Err(TradingError::InvalidInput {
            field: "sellAmount",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "sell amount must be greater than 0",
            },
        });
    }

    let network_cost_amount_in_buy_currency = (&buy_amount * &network_cost_amount) / &sell_amount;
    let protocol_fee_amount = get_protocol_fee_amount(quote, protocol_fee_bps.unwrap_or(0.0))?;
    let partner_fee_bps = partner_fee_bps.unwrap_or(0);
    let stage_inputs = QuoteStageInputs {
        is_sell,
        sell_amount: &sell_amount,
        buy_amount: &buy_amount,
        network_cost_amount: &network_cost_amount,
        network_cost_amount_in_buy_currency: &network_cost_amount_in_buy_currency,
        protocol_fee_amount: &protocol_fee_amount,
        partner_fee_bps,
        slippage_percent_bps,
    };
    let (stages, partner_fee_amount) = build_quote_amount_stages(&stage_inputs);

    stages.into_quote_amounts_and_costs(
        is_sell,
        QuoteFeeBreakdown {
            network_cost_amount,
            network_cost_amount_in_buy_currency,
            partner_fee_amount,
            partner_fee_bps,
            protocol_fee_amount,
            protocol_fee_bps: protocol_fee_bps.unwrap_or(0.0),
        },
    )
}

#[allow(
    clippy::redundant_pub_crate,
    reason = "crate-visible re-export preserves crate::slippage helper imports from sibling modules"
)]
pub(crate) fn gas_with_margin(gas: &Amount) -> Result<Amount, TradingError> {
    let gas = parse_integer("gas", &gas.to_string())?;
    let margin = (&gas * BigInt::from(GAS_MARGIN_PERCENT)) / BigInt::from(100);
    Amount::new((gas + margin).to_string()).map_err(Into::into)
}

#[allow(
    clippy::redundant_pub_crate,
    clippy::option_if_let_else,
    reason = "crate-visible re-export preserves crate::slippage helper imports from sibling modules, and the if let/else form keeps the two parse-radix paths visually parallel"
)]
pub(crate) fn parse_integer(field: &'static str, value: &str) -> Result<BigInt, TradingError> {
    if let Some(hex_value) = value.strip_prefix("0x") {
        BigInt::parse_bytes(hex_value.as_bytes(), 16).ok_or_else(|| TradingError::InvalidNumeric {
            field,
            value: value.to_owned().into(),
        })
    } else {
        BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| TradingError::InvalidNumeric {
            field,
            value: value.to_owned().into(),
        })
    }
}

#[derive(Clone)]
pub(super) struct AmountsBig {
    sell_amount: BigInt,
    buy_amount: BigInt,
}

pub(super) struct QuoteAmountStages {
    before_all_fees: AmountsBig,
    after_protocol_fees: AmountsBig,
    after_network_costs: AmountsBig,
    after_partner_fees: AmountsBig,
    after_slippage: AmountsBig,
    amounts_to_sign: AmountsBig,
}

impl QuoteAmountStages {
    fn into_quote_amounts_and_costs(
        self,
        is_sell: bool,
        fee_breakdown: QuoteFeeBreakdown,
    ) -> Result<QuoteAmountsAndCosts, TradingError> {
        Ok(QuoteAmountsAndCosts::new(
            is_sell,
            fee_breakdown.into_costs()?,
            self.before_all_fees.into_amounts()?,
            self.after_protocol_fees.clone().into_amounts()?,
            self.after_protocol_fees.into_amounts()?,
            self.after_network_costs.into_amounts()?,
            self.after_partner_fees.into_amounts()?,
            self.after_slippage.into_amounts()?,
            self.amounts_to_sign.into_amounts()?,
        ))
    }
}

struct QuoteStageInputs<'a> {
    is_sell: bool,
    sell_amount: &'a BigInt,
    buy_amount: &'a BigInt,
    network_cost_amount: &'a BigInt,
    network_cost_amount_in_buy_currency: &'a BigInt,
    protocol_fee_amount: &'a BigInt,
    partner_fee_bps: u32,
    slippage_percent_bps: u32,
}

fn build_quote_amount_stages(inputs: &QuoteStageInputs<'_>) -> (QuoteAmountStages, BigInt) {
    let before_all_fees = if inputs.is_sell {
        AmountsBig {
            sell_amount: inputs.sell_amount + inputs.network_cost_amount,
            buy_amount: inputs.buy_amount
                + inputs.network_cost_amount_in_buy_currency
                + inputs.protocol_fee_amount,
        }
    } else {
        AmountsBig {
            sell_amount: inputs.sell_amount - inputs.protocol_fee_amount,
            buy_amount: inputs.buy_amount.clone(),
        }
    };

    let after_protocol_fees = if inputs.is_sell {
        AmountsBig {
            sell_amount: before_all_fees.sell_amount.clone(),
            buy_amount: &before_all_fees.buy_amount - inputs.protocol_fee_amount,
        }
    } else {
        AmountsBig {
            sell_amount: inputs.sell_amount.clone(),
            buy_amount: before_all_fees.buy_amount.clone(),
        }
    };

    let after_network_costs = if inputs.is_sell {
        AmountsBig {
            sell_amount: inputs.sell_amount.clone(),
            buy_amount: inputs.buy_amount.clone(),
        }
    } else {
        AmountsBig {
            sell_amount: inputs.sell_amount + inputs.network_cost_amount,
            buy_amount: after_protocol_fees.buy_amount.clone(),
        }
    };

    let surplus_amount_for_partner_fee = if inputs.is_sell {
        before_all_fees.buy_amount.clone()
    } else {
        before_all_fees.sell_amount.clone()
    };
    let partner_fee_amount = if inputs.partner_fee_bps > 0 {
        (&surplus_amount_for_partner_fee * BigInt::from(inputs.partner_fee_bps))
            / BigInt::from(ONE_HUNDRED_BPS)
    } else {
        BigInt::from(0)
    };

    let slippage_amount = |amount: &BigInt| {
        (amount * BigInt::from(inputs.slippage_percent_bps)) / BigInt::from(ONE_HUNDRED_BPS)
    };

    let after_partner_fees = if inputs.is_sell {
        AmountsBig {
            sell_amount: after_network_costs.sell_amount.clone(),
            buy_amount: &after_network_costs.buy_amount - &partner_fee_amount,
        }
    } else {
        AmountsBig {
            sell_amount: &after_network_costs.sell_amount + &partner_fee_amount,
            buy_amount: after_network_costs.buy_amount.clone(),
        }
    };

    let after_slippage = if inputs.is_sell {
        AmountsBig {
            sell_amount: after_partner_fees.sell_amount.clone(),
            buy_amount: &after_partner_fees.buy_amount
                - slippage_amount(&after_partner_fees.buy_amount),
        }
    } else {
        AmountsBig {
            sell_amount: &after_partner_fees.sell_amount
                + slippage_amount(&after_partner_fees.sell_amount),
            buy_amount: after_partner_fees.buy_amount.clone(),
        }
    };

    let amounts_to_sign = if inputs.is_sell {
        AmountsBig {
            sell_amount: before_all_fees.sell_amount.clone(),
            buy_amount: after_slippage.buy_amount.clone(),
        }
    } else {
        AmountsBig {
            sell_amount: after_slippage.sell_amount.clone(),
            buy_amount: before_all_fees.buy_amount.clone(),
        }
    };

    (
        QuoteAmountStages {
            before_all_fees,
            after_protocol_fees,
            after_network_costs,
            after_partner_fees,
            after_slippage,
            amounts_to_sign,
        },
        partner_fee_amount,
    )
}

impl AmountsBig {
    fn into_amounts(self) -> Result<cow_sdk_core::Amounts<Amount>, TradingError> {
        Ok(cow_sdk_core::Amounts::new(
            Amount::new(self.sell_amount.to_string())?,
            Amount::new(self.buy_amount.to_string())?,
        ))
    }
}

pub(super) fn rounded_nonnegative_f64_to_u32(
    value: f64,
    field: &'static str,
) -> Result<u32, TradingError> {
    let rounded = value.round();
    if !rounded.is_finite() || rounded < 0.0 || rounded > f64::from(u32::MAX) {
        return Err(TradingError::NumericOverflow {
            field,
            value: rounded.to_string().into(),
        });
    }
    if rounded == 0.0 {
        return Ok(0);
    }

    format!("{rounded:.0}")
        .parse::<u32>()
        .map_err(|_| TradingError::NumericOverflow {
            field,
            value: rounded.to_string().into(),
        })
}
