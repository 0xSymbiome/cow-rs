use alloy_primitives::aliases::I512;

use cow_sdk_core::{Amount, OrderKind, QuoteAmountsAndCosts};
use cow_sdk_orderbook::QuoteData;

use crate::TradingError;

use super::breakdown::{QuoteFeeBreakdown, get_protocol_fee_amount};
use super::{GAS_MARGIN_PERCENT, ONE_HUNDRED_BPS};

/// Lifts an `i64` integer into the cow slippage-math signed 512-bit
/// primitive without any runtime fallibility.
///
/// The body widens `value` into the eight 64-bit limbs that back
/// [`alloy_primitives::aliases::I512`] via the canonical two's-complement
/// sign-extension: the lowest limb carries the unsigned bit pattern of
/// `value`, and the upper limbs replicate the sign bit (`0` for
/// non-negative inputs, `u64::MAX` for negative inputs). The
/// `Uint::from_limbs` and `Signed::from_raw` constructors are both
/// `const fn`, so the whole lifter is `const`-callable.
#[inline]
const fn i512(value: i64) -> I512 {
    let lower = value.cast_unsigned();
    let upper = if value.is_negative() { u64::MAX } else { 0 };
    I512::from_raw(alloy_primitives::Uint::from_limbs([
        lower, upper, upper, upper, upper, upper, upper, upper,
    ]))
}

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

    if sell_amount <= I512::ZERO {
        return Err(TradingError::InvalidInput {
            field: "sellAmount",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "sell amount must be greater than 0",
            },
        });
    }

    // `buy_amount` and `network_cost_amount` are each bounded by the cow
    // `Amount` newtype to the unsigned 256-bit range. Their product can
    // therefore reach `(2^256 - 1)^2`, which exceeds the I512 signed
    // ceiling of `2^511 - 1`. The cow slippage primitive uses
    // `alloy_primitives::aliases::I512` for headroom over the much smaller
    // products in the surrounding fee/slippage math (~2^283), but this
    // specific product is the workspace's widest intermediate. Guard the
    // multiplication explicitly via `checked_mul` so the boundary case
    // surfaces as a typed `TradingError::NumericOverflow` instead of the
    // debug-build `Signed::handle_overflow` panic or the release-build
    // two's-complement wrap.
    let buy_times_fee = buy_amount.checked_mul(network_cost_amount).ok_or_else(|| {
        TradingError::NumericOverflow {
            field: "buyAmount * networkCostAmount",
            value: format!("{buy_amount} * {network_cost_amount}").into(),
        }
    })?;
    let network_cost_amount_in_buy_currency = buy_times_fee / sell_amount;
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
    let margin = (gas * i512(i64::from(GAS_MARGIN_PERCENT))) / i512(100i64);
    Amount::new((gas + margin).to_string()).map_err(Into::into)
}

#[allow(
    clippy::redundant_pub_crate,
    clippy::option_if_let_else,
    reason = "crate-visible re-export preserves crate::slippage helper imports from sibling modules, and the if let/else form keeps the two parse-radix paths visually parallel"
)]
pub(crate) fn parse_integer(field: &'static str, value: &str) -> Result<I512, TradingError> {
    if value.starts_with("0x") {
        I512::from_hex_str(value).map_err(|_| TradingError::InvalidNumeric {
            field,
            value: value.to_owned().into(),
        })
    } else {
        I512::from_dec_str(value).map_err(|_| TradingError::InvalidNumeric {
            field,
            value: value.to_owned().into(),
        })
    }
}

#[derive(Clone, Copy)]
pub(super) struct AmountsBig {
    sell_amount: I512,
    buy_amount: I512,
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
            self.after_protocol_fees.into_amounts()?,
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
    sell_amount: &'a I512,
    buy_amount: &'a I512,
    network_cost_amount: &'a I512,
    network_cost_amount_in_buy_currency: &'a I512,
    protocol_fee_amount: &'a I512,
    partner_fee_bps: u32,
    slippage_percent_bps: u32,
}

fn build_quote_amount_stages(inputs: &QuoteStageInputs<'_>) -> (QuoteAmountStages, I512) {
    // The cow slippage primitive is `alloy_primitives::I512`, which is
    // `Copy`, so the per-stage borrow / clone discipline that the prior
    // `num_bigint::BigInt` body needed (BigInt is heap-backed and not
    // Copy) collapses into plain value moves. Every stage carries
    // borrowed inputs plus a small set of derived I512 values.
    let before_all_fees = if inputs.is_sell {
        AmountsBig {
            sell_amount: *inputs.sell_amount + *inputs.network_cost_amount,
            buy_amount: *inputs.buy_amount
                + *inputs.network_cost_amount_in_buy_currency
                + *inputs.protocol_fee_amount,
        }
    } else {
        AmountsBig {
            sell_amount: *inputs.sell_amount - *inputs.protocol_fee_amount,
            buy_amount: *inputs.buy_amount,
        }
    };

    let after_protocol_fees = if inputs.is_sell {
        AmountsBig {
            sell_amount: before_all_fees.sell_amount,
            buy_amount: before_all_fees.buy_amount - *inputs.protocol_fee_amount,
        }
    } else {
        AmountsBig {
            sell_amount: *inputs.sell_amount,
            buy_amount: before_all_fees.buy_amount,
        }
    };

    let after_network_costs = if inputs.is_sell {
        AmountsBig {
            sell_amount: *inputs.sell_amount,
            buy_amount: *inputs.buy_amount,
        }
    } else {
        AmountsBig {
            sell_amount: *inputs.sell_amount + *inputs.network_cost_amount,
            buy_amount: after_protocol_fees.buy_amount,
        }
    };

    let surplus_amount_for_partner_fee = if inputs.is_sell {
        before_all_fees.buy_amount
    } else {
        before_all_fees.sell_amount
    };
    let partner_fee_amount = if inputs.partner_fee_bps > 0 {
        (surplus_amount_for_partner_fee * i512(i64::from(inputs.partner_fee_bps)))
            / i512(ONE_HUNDRED_BPS)
    } else {
        I512::ZERO
    };

    let slippage_amount = |amount: I512| {
        (amount * i512(i64::from(inputs.slippage_percent_bps))) / i512(ONE_HUNDRED_BPS)
    };

    let after_partner_fees = if inputs.is_sell {
        AmountsBig {
            sell_amount: after_network_costs.sell_amount,
            buy_amount: after_network_costs.buy_amount - partner_fee_amount,
        }
    } else {
        AmountsBig {
            sell_amount: after_network_costs.sell_amount + partner_fee_amount,
            buy_amount: after_network_costs.buy_amount,
        }
    };

    let after_slippage = if inputs.is_sell {
        AmountsBig {
            sell_amount: after_partner_fees.sell_amount,
            buy_amount: after_partner_fees.buy_amount
                - slippage_amount(after_partner_fees.buy_amount),
        }
    } else {
        AmountsBig {
            sell_amount: after_partner_fees.sell_amount
                + slippage_amount(after_partner_fees.sell_amount),
            buy_amount: after_partner_fees.buy_amount,
        }
    };

    let amounts_to_sign = if inputs.is_sell {
        AmountsBig {
            sell_amount: before_all_fees.sell_amount,
            buy_amount: after_slippage.buy_amount,
        }
    } else {
        AmountsBig {
            sell_amount: after_slippage.sell_amount,
            buy_amount: before_all_fees.buy_amount,
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
