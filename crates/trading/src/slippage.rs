use num_bigint::BigInt;

use cow_sdk_app_data::PartnerFee;
use cow_sdk_core::{Amount, OrderKind, QuoteAmountsAndCosts, SupportedChainId};
use cow_sdk_orderbook::{OrderQuoteResponse, PriceQuality, QuoteData};

use crate::{
    QuoterParameters, SlippageToleranceResponse, SwapAdvancedSettings, TradeParameters,
    TradingError,
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
pub const GAS_LIMIT_DEFAULT: u32 = 150_000;

const PROTOCOL_FEE_BPS_SCALE: i64 = 100_000;
const ONE_HUNDRED_BPS: i64 = 10_000;
const PERCENT_SCALE: i64 = 1_000_000;
const SLIPPAGE_FEE_MULTIPLIER_PERCENT: f64 = 50.0;
const SLIPPAGE_VOLUME_MULTIPLIER_PERCENT: f64 = 0.5;
const PROTOCOL_FEE_BPS_MIN: f64 = 0.0001;

/// Returns the default slippage floor for the given chain and trade style.
#[must_use]
pub const fn default_slippage_bps(_chain_id: SupportedChainId, _is_ethflow: bool) -> u32 {
    DEFAULT_SLIPPAGE_BPS
}

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

    if fee_amount < BigInt::from(0) {
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

    if sell_amount <= BigInt::from(0) {
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
    let sell_amount = parse_integer("sellAmount", &quote.sell_amount)?;
    let buy_amount = parse_integer("buyAmount", &quote.buy_amount)?;
    let network_cost_amount = parse_integer("feeAmount", quote.network_cost_amount())?;

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
    trade_parameters: &TradeParameters,
    trader: &QuoterParameters,
    is_ethflow: bool,
    volume_multiplier_percent: Option<f64>,
) -> Result<u32, TradingError> {
    let amounts = calculate_quote_amounts_and_costs(
        &quote.quote,
        0,
        partner_fee_bps(trade_parameters.partner_fee.as_ref()),
        sanitize_protocol_fee_bps(quote.protocol_fee_bps.as_deref()),
    )?;
    let fee_amount = suggest_slippage_from_fee(
        quote.quote.network_cost_amount(),
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
    let lower_cap = if is_ethflow {
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
    trade_parameters: &TradeParameters,
    trader: &QuoterParameters,
    quote: &OrderQuoteResponse,
    is_ethflow: bool,
    advanced_settings: Option<&SwapAdvancedSettings>,
) -> Result<SlippageToleranceResponse, TradingError> {
    let default_suggestion =
        suggest_slippage_bps(quote, trade_parameters, trader, is_ethflow, None)?;
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
        sell_token: trade_parameters.sell_token.clone(),
        buy_token: trade_parameters.buy_token.clone(),
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

    match provider.get_slippage_suggestion(request).await {
        Ok(crate::SlippageToleranceResponse {
            slippage_bps: Some(suggested),
        }) => Ok(crate::SlippageToleranceResponse {
            slippage_bps: Some(suggest_slippage_bps(
                quote,
                trade_parameters,
                trader,
                is_ethflow,
                Some(f64::from(suggested) / 100.0),
            )?),
        }),
        Ok(_) | Err(_) => Ok(crate::SlippageToleranceResponse {
            slippage_bps: Some(default_suggestion),
        }),
    }
}

/// Extracts the first supported volume-based partner-fee basis-point value from the typed
/// partner-fee payload.
#[must_use]
pub fn partner_fee_bps(partner_fee: Option<&PartnerFee>) -> Option<u32> {
    partner_fee.and_then(PartnerFee::volume_bps).map(u32::from)
}

pub(crate) fn gas_with_margin(gas: &Amount) -> Result<Amount, TradingError> {
    let gas = parse_integer("gas", &gas.to_string())?;
    let margin = (&gas * BigInt::from(GAS_MARGIN_PERCENT)) / BigInt::from(100);
    Amount::new((gas + margin).to_string()).map_err(Into::into)
}

#[allow(
    clippy::option_if_let_else,
    reason = "both branches carry the same multi-field InvalidNumeric error literal; the if let/else form keeps the two parse-radix paths visually parallel instead of nesting duplicated error construction inside two map_or_else closures"
)]
pub(crate) fn parse_integer(field: &'static str, value: &str) -> Result<BigInt, TradingError> {
    if let Some(hex_value) = value.strip_prefix("0x") {
        BigInt::parse_bytes(hex_value.as_bytes(), 16).ok_or_else(|| TradingError::InvalidNumeric {
            field,
            value: value.to_owned(),
        })
    } else {
        BigInt::parse_bytes(value.as_bytes(), 10).ok_or_else(|| TradingError::InvalidNumeric {
            field,
            value: value.to_owned(),
        })
    }
}

fn parse_percent_scaled(percent: f64, field: &'static str) -> Result<BigInt, TradingError> {
    if !percent.is_finite() || percent < 0.0 {
        return Err(TradingError::InvalidInput {
            field,
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "percent must be finite and non-negative",
            },
        });
    }

    let rendered = format!("{percent:.6}");
    let mut parts = rendered.split('.');
    let whole = parts.next().unwrap_or("0");
    let fractional = parts.next().unwrap_or("0");
    let whole =
        BigInt::parse_bytes(whole.as_bytes(), 10).ok_or_else(|| TradingError::InvalidNumeric {
            field,
            value: rendered.clone(),
        })?;
    let fractional = BigInt::parse_bytes(fractional.as_bytes(), 10).ok_or_else(|| {
        TradingError::InvalidNumeric {
            field,
            value: rendered.clone(),
        }
    })?;

    Ok(whole * BigInt::from(PERCENT_SCALE) + fractional)
}

fn apply_percentage(amount: &BigInt, scaled_percent: BigInt) -> BigInt {
    let denominator = BigInt::from(100 * PERCENT_SCALE);
    let numerator = amount * scaled_percent;
    (numerator + (&denominator / 2)) / denominator
}

fn get_protocol_fee_amount(
    quote: &QuoteData,
    protocol_fee_bps: f64,
) -> Result<BigInt, TradingError> {
    if protocol_fee_bps <= 0.0 {
        return Ok(BigInt::from(0));
    }

    let protocol_fee_bps_big =
        parse_percent_scaled(protocol_fee_bps, "protocolFeeBps")? / BigInt::from(10);

    if protocol_fee_bps_big <= BigInt::from(0) {
        return Ok(BigInt::from(0));
    }

    let sell_amount = parse_integer("sellAmount", &quote.sell_amount)?;
    let buy_amount = parse_integer("buyAmount", &quote.buy_amount)?;
    let fee_amount = parse_integer("feeAmount", quote.network_cost_amount())?;
    let denominator_base = BigInt::from(ONE_HUNDRED_BPS * PROTOCOL_FEE_BPS_SCALE);

    if quote.kind == OrderKind::Sell {
        let denominator = &denominator_base - &protocol_fee_bps_big;
        Ok((buy_amount * protocol_fee_bps_big) / denominator)
    } else {
        let denominator = &denominator_base + &protocol_fee_bps_big;
        Ok(((sell_amount + fee_amount) * protocol_fee_bps_big) / denominator)
    }
}

fn get_slippage_percent_scaled(
    is_sell: bool,
    sell_amount_before_network_costs: &Amount,
    sell_amount_after_network_costs: &Amount,
    slippage: &str,
) -> Result<BigInt, TradingError> {
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

    if sell_amount <= BigInt::from(0) {
        return Err(TradingError::InvalidInput {
            field: "sellAmount",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "sell amount must be greater than 0",
            },
        });
    }
    if slippage < BigInt::from(0) {
        return Err(TradingError::InvalidInput {
            field: "slippage",
            reason: cow_sdk_core::ValidationReason::OutOfRange {
                details: "slippage must be non-negative",
            },
        });
    }

    let scale = BigInt::from(PERCENT_SCALE);
    if is_sell {
        Ok(&scale - ((&scale * (&sell_amount - slippage)) / sell_amount))
    } else {
        Ok(((&scale * (&sell_amount + slippage)) / sell_amount) - &scale)
    }
}

fn scaled_percent_to_bps(percent_scaled: &BigInt) -> Result<u32, TradingError> {
    let denominator = BigInt::from(100);
    let rounded: BigInt = (percent_scaled + (&denominator / 2)) / denominator;
    if rounded >= BigInt::from(MAX_SLIPPAGE_BPS) {
        return Ok(MAX_SLIPPAGE_BPS);
    }
    let value = rounded.to_string();
    value
        .parse::<u32>()
        .map_err(|_| TradingError::NumericOverflow {
            field: "slippageBps",
            value,
        })
}

#[derive(Clone)]
struct AmountsBig {
    sell_amount: BigInt,
    buy_amount: BigInt,
}

struct QuoteFeeBreakdown {
    network_cost_amount: BigInt,
    network_cost_amount_in_buy_currency: BigInt,
    partner_fee_amount: BigInt,
    partner_fee_bps: u32,
    protocol_fee_amount: BigInt,
    protocol_fee_bps: f64,
}

struct QuoteAmountStages {
    before_all_fees: AmountsBig,
    after_protocol_fees: AmountsBig,
    after_network_costs: AmountsBig,
    after_partner_fees: AmountsBig,
    after_slippage: AmountsBig,
    amounts_to_sign: AmountsBig,
}

impl QuoteFeeBreakdown {
    fn into_costs(self) -> Result<cow_sdk_core::Costs<Amount>, TradingError> {
        Ok(cow_sdk_core::Costs {
            network_fee: cow_sdk_core::NetworkFee {
                amount_in_sell_currency: Amount::new(self.network_cost_amount.to_string())?,
                amount_in_buy_currency: Amount::new(
                    self.network_cost_amount_in_buy_currency.to_string(),
                )?,
            },
            partner_fee: cow_sdk_core::FeeComponent {
                amount: Amount::new(self.partner_fee_amount.to_string())?,
                bps: self.partner_fee_bps,
            },
            protocol_fee: cow_sdk_core::FeeComponent {
                amount: Amount::new(self.protocol_fee_amount.to_string())?,
                bps: rounded_nonnegative_f64_to_u32(self.protocol_fee_bps, "protocolFeeBps")?,
            },
        })
    }
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
        Ok(cow_sdk_core::Amounts {
            sell_amount: Amount::new(self.sell_amount.to_string())?,
            buy_amount: Amount::new(self.buy_amount.to_string())?,
        })
    }
}

fn rounded_nonnegative_f64_to_u32(value: f64, field: &'static str) -> Result<u32, TradingError> {
    let rounded = value.round();
    if !rounded.is_finite() || rounded < 0.0 || rounded > f64::from(u32::MAX) {
        return Err(TradingError::NumericOverflow {
            field,
            value: rounded.to_string(),
        });
    }
    if rounded == 0.0 {
        return Ok(0);
    }

    format!("{rounded:.0}")
        .parse::<u32>()
        .map_err(|_| TradingError::NumericOverflow {
            field,
            value: rounded.to_string(),
        })
}
