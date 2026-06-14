//! Slippage and fee calculation helpers.

use alloy_primitives::aliases::I512;

use cow_sdk_app_data::PartnerFee;
use cow_sdk_core::{Amount, OrderKind, QuoteAmountsAndCosts, SupportedChainId};
use cow_sdk_orderbook::{OrderQuoteResponse, PriceQuality, QuoteData};

use crate::{
    QuoterParams, SlippageToleranceResponse, TradeAdvancedSettings, TradeParams, TradingError,
};

/// Default quote validity, in seconds, when no explicit validity window is supplied.
pub const DEFAULT_QUOTE_VALIDITY: u32 = 60 * 30;
/// Default slippage suggestion, in basis points, for flows that do not require a higher floor.
pub const DEFAULT_SLIPPAGE_BPS: u32 = 50;
/// Maximum supported slippage, in basis points.
pub const MAX_SLIPPAGE_BPS: u32 = 10_000;
/// Extra gas margin, in percent, added to derived on-chain transaction estimates.
pub const GAS_MARGIN_PERCENT: u32 = 20;
/// Fallback gas limit used when no explicit verification gas limit is available.
pub const DEFAULT_GAS_LIMIT: u32 = 150_000;

pub(super) const ONE_HUNDRED_BPS: i64 = 10_000;

/// Returns the default slippage floor for the given chain and trade style.
#[must_use]
pub const fn default_slippage_bps(_chain_id: SupportedChainId, _is_ethflow: bool) -> u32 {
    DEFAULT_SLIPPAGE_BPS
}

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

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the guard above bounds `rounded` to a finite, non-negative value no greater than `u32::MAX`, and `round()` makes it integer-valued, so the narrowing cast is exact"
    )]
    Ok(rounded as u32)
}

const PROTOCOL_FEE_BPS_SCALE: i64 = 100_000;
const PERCENT_SCALE: i64 = 1_000_000;

/// Extracts the first supported volume-based partner-fee basis-point value from the typed
/// partner-fee payload.
#[must_use]
pub fn partner_fee_bps(partner_fee: Option<&PartnerFee>) -> Option<u32> {
    partner_fee.and_then(PartnerFee::volume_bps).map(u32::from)
}

/// Lifts `percent` (a non-negative finite f64) into the
/// `PERCENT_SCALE`-scaled signed-512-bit integer the cow slippage math
/// expects, applying the `Math.floor(p * 1e6)` fixed-point truncation the
/// upstream `@cowprotocol/cow-sdk` uses (ADR 0066), consistent with the
/// services fee accounting in `crates/orderbook/src/quoter.rs`.
///
/// The prior `format!("{p:.6}")`-based path applied round-half-to-even
/// at the 6th decimal place, which diverged from `floor` for
/// high-precision floats. Cow protocol-fee strings on the wire are
/// always clean decimals (`"0.5"`, `"1.5"`) that survive either
/// rounding mode; the explicit floor keeps the cow surface deterministic
/// across float precisions.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "`PERCENT_SCALE` (1_000_000) is exactly representable in f64; the explicit `!is_finite | < 0 | > i64::MAX` guard below bounds the floor result before the narrowing cast to `i64`"
)]
pub(super) fn parse_percent_scaled(
    percent: f64,
    field: &'static str,
) -> Result<I512, TradingError> {
    if !percent.is_finite() || percent < 0.0 {
        return Err(TradingError::InvalidInput {
            field,
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "percent must be finite and non-negative",
            },
        });
    }

    let scaled = (percent * PERCENT_SCALE as f64).floor();
    if !scaled.is_finite() || scaled < 0.0 || scaled > i64::MAX as f64 {
        return Err(TradingError::NumericOverflow {
            field,
            value: scaled.to_string().into(),
        });
    }
    Ok(i512(scaled as i64))
}

pub(super) fn apply_percentage(amount: &I512, scaled_percent: I512) -> I512 {
    let denominator = i512(100 * PERCENT_SCALE);
    let numerator = *amount * scaled_percent;
    (numerator + (denominator / i512(2))) / denominator
}

/// Lifts `protocol_fee_bps` (e.g., `1.5` for 1.5 bps) into the
/// `PROTOCOL_FEE_BPS_SCALE`-scaled signed-512-bit integer the cow
/// settlement math expects, applying the `Math.round(p * 1e5)`
/// round-half-away-from-zero step the upstream `@cowprotocol/cow-sdk`
/// uses (ADR 0066), consistent with the services protocol-fee accounting
/// in `crates/orderbook/src/quoter.rs`. The previous cow path went through
/// `parse_percent_scaled(p) / 10`, which composed round-half-to-even at
/// the 6th decimal place with truncation by 10; that diverged for
/// sub-permille protocol fees with non-zero precision beyond the 5th
/// decimal. Wire `protocol_fee_bps` strings from the cow orderbook API
/// are always clean decimals that survive either rounding mode; the
/// explicit round keeps the cow surface deterministic across float
/// precisions.
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "`PROTOCOL_FEE_BPS_SCALE` (100_000) is exactly representable in f64; the explicit `!is_finite | < 0 | > i64::MAX` guard below bounds the rounded result before the narrowing cast to `i64`"
)]
fn protocol_fee_bps_scaled(protocol_fee_bps: f64) -> Result<I512, TradingError> {
    if !protocol_fee_bps.is_finite() || protocol_fee_bps < 0.0 {
        return Err(TradingError::InvalidInput {
            field: "protocolFeeBps",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "protocol fee bps must be finite and non-negative",
            },
        });
    }
    let scaled = (protocol_fee_bps * PROTOCOL_FEE_BPS_SCALE as f64).round();
    if !scaled.is_finite() || scaled < 0.0 || scaled > i64::MAX as f64 {
        return Err(TradingError::NumericOverflow {
            field: "protocolFeeBps",
            value: scaled.to_string().into(),
        });
    }
    Ok(i512(scaled as i64))
}

pub(super) fn get_protocol_fee_amount(
    quote: &QuoteData,
    protocol_fee_bps: f64,
) -> Result<I512, TradingError> {
    if protocol_fee_bps <= 0.0 {
        return Ok(I512::ZERO);
    }

    let protocol_fee_bps_big = protocol_fee_bps_scaled(protocol_fee_bps)?;

    if protocol_fee_bps_big <= I512::ZERO {
        return Ok(I512::ZERO);
    }

    let sell_amount = parse_integer("sellAmount", &quote.sell_amount.to_string())?;
    let buy_amount = parse_integer("buyAmount", &quote.buy_amount.to_string())?;
    let fee_amount = parse_integer("feeAmount", &quote.network_cost_amount().to_string())?;
    let denominator_base = i512(ONE_HUNDRED_BPS) * i512(PROTOCOL_FEE_BPS_SCALE);

    // Reject protocol-fee values at or above 100%: on the sell path that
    // would zero the `denominator_base - protocol_fee_bps_big` divisor and
    // panic the typed math; on every path it represents a fee that consumes
    // the entire order, which the public surface does not support.
    if protocol_fee_bps_big >= denominator_base {
        return Err(TradingError::InvalidInput {
            field: "protocolFeeBps",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "protocol fee must be strictly less than 100%",
            },
        });
    }

    if quote.kind == OrderKind::Sell {
        let denominator = denominator_base - protocol_fee_bps_big;
        Ok((buy_amount * protocol_fee_bps_big) / denominator)
    } else {
        let denominator = denominator_base + protocol_fee_bps_big;
        Ok(((sell_amount + fee_amount) * protocol_fee_bps_big) / denominator)
    }
}

pub(super) fn get_slippage_percent_scaled(
    is_sell: bool,
    sell_amount_before_network_costs: &Amount,
    sell_amount_after_network_costs: &Amount,
    slippage: &str,
) -> Result<I512, TradingError> {
    let sell_before = parse_integer(
        "sellAmountBeforeNetworkCosts",
        &sell_amount_before_network_costs.to_string(),
    )?;
    let sell_after = parse_integer(
        "sellAmountAfterNetworkCosts",
        &sell_amount_after_network_costs.to_string(),
    )?;
    let slippage = parse_integer("slippage", slippage)?;
    let sell_amount = if is_sell { sell_after } else { sell_before };

    if sell_amount <= I512::ZERO {
        return Err(TradingError::InvalidInput {
            field: "sellAmount",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "sell amount must be greater than 0",
            },
        });
    }
    if slippage < I512::ZERO {
        return Err(TradingError::InvalidInput {
            field: "slippage",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "slippage must be non-negative",
            },
        });
    }

    let scale = i512(PERCENT_SCALE);
    if is_sell {
        Ok(scale - ((scale * (sell_amount - slippage)) / sell_amount))
    } else {
        Ok(((scale * (sell_amount + slippage)) / sell_amount) - scale)
    }
}

pub(super) fn scaled_percent_to_bps(percent_scaled: &I512) -> Result<u32, TradingError> {
    let denominator = i512(100);
    let rounded: I512 = (*percent_scaled + (denominator / i512(2))) / denominator;
    if rounded >= i512(i64::from(MAX_SLIPPAGE_BPS)) {
        return Ok(MAX_SLIPPAGE_BPS);
    }
    let value = rounded.to_string();
    value
        .parse::<u32>()
        .map_err(|_| TradingError::NumericOverflow {
            field: "slippageBps",
            value: value.into(),
        })
}

pub(super) struct QuoteFeeBreakdown {
    pub(super) network_cost_amount: I512,
    pub(super) network_cost_amount_in_buy_currency: I512,
    pub(super) partner_fee_amount: I512,
    pub(super) partner_fee_bps: u32,
    pub(super) protocol_fee_amount: I512,
    pub(super) protocol_fee_bps: f64,
}

impl QuoteFeeBreakdown {
    pub(super) fn into_costs(self) -> Result<cow_sdk_core::Costs<Amount>, TradingError> {
        Ok(cow_sdk_core::Costs::new(
            cow_sdk_core::NetworkFee::new(
                Amount::new(self.network_cost_amount.to_string())?,
                Amount::new(self.network_cost_amount_in_buy_currency.to_string())?,
            ),
            cow_sdk_core::FeeComponent::new(
                Amount::new(self.partner_fee_amount.to_string())?,
                self.partner_fee_bps,
            ),
            cow_sdk_core::FeeComponent::new(
                Amount::new(self.protocol_fee_amount.to_string())?,
                rounded_nonnegative_f64_to_u32(self.protocol_fee_bps, "protocolFeeBps")?,
            ),
        ))
    }
}

const SLIPPAGE_FEE_MULTIPLIER_PERCENT: f64 = 50.0;
const SLIPPAGE_VOLUME_MULTIPLIER_PERCENT: f64 = 0.5;
const PROTOCOL_FEE_BPS_MIN: f64 = 0.0001;

/// Parses protocol-fee basis points into a finite floating-point value.
///
/// Values that are malformed, non-finite, or smaller than the minimum supported precision are
/// ignored and return `None`.
#[must_use]
pub fn sanitize_protocol_fee_bps(protocol_fee_bps: Option<&str>) -> Option<f64> {
    let parsed = protocol_fee_bps.and_then(|value| value.parse::<f64>().ok())?;

    if !parsed.is_finite() || parsed < PROTOCOL_FEE_BPS_MIN {
        return None;
    }

    Some(parsed)
}

/// Suggests a slippage amount from a quote fee amount and multiplier percentage.
///
/// Percentage inputs are rounded to six decimal places before they are converted into integer
/// math, and the resulting amount is rounded to the nearest integer unit.
///
/// # Errors
///
/// Returns an error when the fee amount is malformed, negative, or the multiplier is negative or
/// non-finite.
pub fn suggest_slippage_from_fee(
    fee_amount: &str,
    multiplying_factor_percent: f64,
) -> Result<Amount, TradingError> {
    let fee_amount = parse_integer("feeAmount", fee_amount)?;

    if fee_amount < I512::ZERO {
        return Err(TradingError::InvalidInput {
            field: "feeAmount",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "fee amount must be non-negative",
            },
        });
    }

    let percent = parse_percent_scaled(multiplying_factor_percent, "multiplyingFactorPercent")?;
    Amount::new(apply_percentage(&fee_amount, percent).to_string()).map_err(Into::into)
}

/// Suggests a slippage amount from the quoted sell volume after network-cost adjustment.
///
/// Sell orders use the post-network-cost sell amount, while buy orders use the pre-network-cost
/// sell amount. Percentage inputs are rounded to six decimal places before integer math is
/// applied, and the resulting amount is rounded to the nearest integer unit.
///
/// # Errors
///
/// Returns an error when the referenced amounts are malformed, when the selected sell amount is
/// zero or negative, or when the percentage is negative or non-finite.
pub fn suggest_slippage_from_volume(
    is_sell: bool,
    sell_amount_before_network_costs: &str,
    sell_amount_after_network_costs: &str,
    slippage_percent: f64,
) -> Result<Amount, TradingError> {
    let sell_before = parse_integer(
        "sellAmountBeforeNetworkCosts",
        sell_amount_before_network_costs,
    )?;
    let sell_after = parse_integer(
        "sellAmountAfterNetworkCosts",
        sell_amount_after_network_costs,
    )?;
    let sell_amount = if is_sell { sell_after } else { sell_before };

    if sell_amount <= I512::ZERO {
        return Err(TradingError::InvalidInput {
            field: "sellAmount",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "sell amount must be greater than 0",
            },
        });
    }

    let percent = parse_percent_scaled(slippage_percent, "slippagePercent")?;
    Amount::new(apply_percentage(&sell_amount, percent).to_string()).map_err(Into::into)
}

/// Suggests a slippage tolerance in basis points for a quote response.
///
/// The result combines fee-based and volume-based suggestions, rounds the derived scaled percent
/// to the nearest basis point, and clamps the final value into the supported range. `EthFlow` flows
/// also apply the default slippage as a lower bound.
///
/// # Errors
///
/// Returns an error when quote amounts, fee inputs, or the derived slippage values are malformed
/// or overflow the supported typed amount surface.
pub fn suggest_slippage_bps(
    quote: &OrderQuoteResponse,
    trade_parameters: &TradeParams,
    trader: &QuoterParams,
    is_eth_flow: bool,
    volume_multiplier_percent: Option<f64>,
) -> Result<u32, TradingError> {
    let amounts = calculate_quote_amounts_and_costs(
        &quote.quote,
        0,
        partner_fee_bps(trade_parameters.partner_fee.as_ref()),
        sanitize_protocol_fee_bps(quote.protocol_fee_bps.as_deref()),
    )?;
    let fee_amount = suggest_slippage_from_fee(
        &quote.quote.network_cost_amount().to_string(),
        SLIPPAGE_FEE_MULTIPLIER_PERCENT,
    )?;
    let volume_amount = suggest_slippage_from_volume(
        amounts.is_sell,
        &amounts.before_network_costs.sell_amount.to_string(),
        &amounts.after_network_costs.sell_amount.to_string(),
        volume_multiplier_percent.unwrap_or(SLIPPAGE_VOLUME_MULTIPLIER_PERCENT),
    )?;

    let total_slippage = parse_integer("totalSlippage", &fee_amount.to_string())?
        + parse_integer("totalSlippage", &volume_amount.to_string())?;
    let slippage_percent_scaled = get_slippage_percent_scaled(
        amounts.is_sell,
        &amounts.before_network_costs.sell_amount,
        &amounts.after_network_costs.sell_amount,
        &total_slippage.to_string(),
    )?;
    let slippage_bps = scaled_percent_to_bps(&slippage_percent_scaled)?;
    let lower_cap = if is_eth_flow {
        default_slippage_bps(trader.chain_id, true)
    } else {
        0
    };

    Ok(slippage_bps.clamp(lower_cap, MAX_SLIPPAGE_BPS))
}

/// Resolves the effective slippage suggestion for a quote flow.
///
/// When no custom slippage suggester is configured, or when quote pricing uses
/// [`PriceQuality::Fast`], the built-in suggestion is returned directly. Custom suggesters may
/// influence the volume multiplier, but failures fall back to the built-in suggestion instead of
/// changing the quoting outcome.
///
/// # Errors
///
/// Returns an error when the built-in slippage calculation cannot be completed because quote or
/// fee inputs are malformed.
pub async fn resolve_slippage_suggestion(
    chain_id: SupportedChainId,
    trade_parameters: &TradeParams,
    trader: &QuoterParams,
    quote: &OrderQuoteResponse,
    is_eth_flow: bool,
    advanced_settings: Option<&TradeAdvancedSettings>,
) -> Result<SlippageToleranceResponse, TradingError> {
    let default_suggestion =
        suggest_slippage_bps(quote, trade_parameters, trader, is_eth_flow, None)?;
    let Some(provider) =
        advanced_settings.and_then(|settings| settings.slippage_suggester.as_ref())
    else {
        return Ok(SlippageToleranceResponse {
            slippage_bps: Some(default_suggestion),
        });
    };

    let price_quality = advanced_settings
        .and_then(|settings| settings.quote_request.as_ref())
        .and_then(|request| request.price_quality)
        .unwrap_or(PriceQuality::Optimal);

    if price_quality == PriceQuality::Fast {
        return Ok(SlippageToleranceResponse {
            slippage_bps: Some(default_suggestion),
        });
    }

    let amounts = calculate_quote_amounts_and_costs(
        &quote.quote,
        0,
        partner_fee_bps(trade_parameters.partner_fee.as_ref()),
        sanitize_protocol_fee_bps(quote.protocol_fee_bps.as_deref()),
    )?;

    let request = crate::SlippageToleranceRequest {
        chain_id,
        sell_token: trade_parameters.sell_token,
        buy_token: trade_parameters.buy_token,
        sell_amount: Some(if amounts.is_sell {
            amounts.before_all_fees.sell_amount
        } else {
            amounts.after_slippage.sell_amount
        }),
        buy_amount: Some(if amounts.is_sell {
            amounts.after_slippage.buy_amount
        } else {
            amounts.before_all_fees.buy_amount
        }),
    };

    match provider.slippage_suggestion(request).await {
        Ok(crate::SlippageToleranceResponse {
            slippage_bps: Some(suggested),
        }) => Ok(crate::SlippageToleranceResponse {
            slippage_bps: Some(suggest_slippage_bps(
                quote,
                trade_parameters,
                trader,
                is_eth_flow,
                Some(f64::from(suggested) / 100.0),
            )?),
        }),
        Ok(_) | Err(_) => Ok(crate::SlippageToleranceResponse {
            slippage_bps: Some(default_suggestion),
        }),
    }
}
