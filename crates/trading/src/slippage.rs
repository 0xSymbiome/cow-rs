use num_bigint::BigInt;
use serde_json::Value;

use cow_sdk_core::{OrderKind, QuoteAmountsAndCosts, SupportedChainId};
use cow_sdk_orderbook::{OrderQuoteResponse, PriceQuality, QuoteData};

use crate::{
    QuoterParameters, SlippageToleranceResponse, SwapAdvancedSettings, TradeParameters,
    TradingError,
};

pub const DEFAULT_QUOTE_VALIDITY: u32 = 60 * 30;
pub const DEFAULT_SLIPPAGE_BPS: u32 = 50;
pub const MAX_SLIPPAGE_BPS: u32 = 10_000;
pub const GAS_MARGIN_PERCENT: u32 = 20;
pub const GAS_LIMIT_DEFAULT: u32 = 150_000;

const PROTOCOL_FEE_BPS_SCALE: i64 = 100_000;
const ONE_HUNDRED_BPS: i64 = 10_000;
const PERCENT_SCALE: i64 = 1_000_000;
const SLIPPAGE_FEE_MULTIPLIER_PERCENT: f64 = 50.0;
const SLIPPAGE_VOLUME_MULTIPLIER_PERCENT: f64 = 0.5;
const PROTOCOL_FEE_BPS_MIN: f64 = 0.0001;

pub fn default_slippage_bps(_chain_id: SupportedChainId, _is_ethflow: bool) -> u32 {
    DEFAULT_SLIPPAGE_BPS
}

pub fn sanitize_protocol_fee_bps(protocol_fee_bps: Option<&str>) -> Option<f64> {
    let parsed = protocol_fee_bps.and_then(|value| value.parse::<f64>().ok())?;

    if !parsed.is_finite() || parsed < PROTOCOL_FEE_BPS_MIN {
        return None;
    }

    Some(parsed)
}

pub fn suggest_slippage_from_fee(
    fee_amount: &str,
    multiplying_factor_percent: f64,
) -> Result<String, TradingError> {
    let fee_amount = parse_integer("feeAmount", fee_amount)?;

    if fee_amount < BigInt::from(0) {
        return Err(TradingError::InvalidInput(format!(
            "Fee amount must be non-negative: {fee_amount}"
        )));
    }

    let percent = parse_percent_scaled(multiplying_factor_percent, "multiplyingFactorPercent")?;
    Ok(apply_percentage(&fee_amount, percent).to_string())
}

pub fn suggest_slippage_from_volume(
    is_sell: bool,
    sell_amount_before_network_costs: &str,
    sell_amount_after_network_costs: &str,
    slippage_percent: f64,
) -> Result<String, TradingError> {
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
        return Err(TradingError::InvalidInput(format!(
            "sellAmount must be greater than 0: {sell_amount}"
        )));
    }

    let percent = parse_percent_scaled(slippage_percent, "slippagePercent")?;
    Ok(apply_percentage(&sell_amount, percent).to_string())
}

pub fn calculate_quote_amounts_and_costs(
    quote: &QuoteData,
    slippage_percent_bps: u32,
    partner_fee_bps: Option<u32>,
    protocol_fee_bps: Option<f64>,
) -> Result<QuoteAmountsAndCosts<String>, TradingError> {
    let is_sell = quote.kind == OrderKind::Sell;
    let sell_amount = parse_integer("sellAmount", &quote.sell_amount)?;
    let buy_amount = parse_integer("buyAmount", &quote.buy_amount)?;
    let network_cost_amount = parse_integer("feeAmount", &quote.fee_amount)?;

    if sell_amount <= BigInt::from(0) {
        return Err(TradingError::InvalidInput(format!(
            "sellAmount must be greater than 0: {sell_amount}"
        )));
    }

    let network_cost_amount_in_buy_currency = (&buy_amount * &network_cost_amount) / &sell_amount;
    let protocol_fee_amount = get_protocol_fee_amount(quote, protocol_fee_bps.unwrap_or(0.0))?;

    let before_all_fees = if is_sell {
        AmountsBig {
            sell_amount: &sell_amount + &network_cost_amount,
            buy_amount: &buy_amount + &network_cost_amount_in_buy_currency + &protocol_fee_amount,
        }
    } else {
        AmountsBig {
            sell_amount: &sell_amount - &protocol_fee_amount,
            buy_amount: buy_amount.clone(),
        }
    };

    let after_protocol_fees = if is_sell {
        AmountsBig {
            sell_amount: before_all_fees.sell_amount.clone(),
            buy_amount: &before_all_fees.buy_amount - &protocol_fee_amount,
        }
    } else {
        AmountsBig {
            sell_amount: sell_amount.clone(),
            buy_amount: before_all_fees.buy_amount.clone(),
        }
    };

    let after_network_costs = if is_sell {
        AmountsBig {
            sell_amount: sell_amount.clone(),
            buy_amount: buy_amount.clone(),
        }
    } else {
        AmountsBig {
            sell_amount: &sell_amount + &network_cost_amount,
            buy_amount: after_protocol_fees.buy_amount.clone(),
        }
    };

    let partner_fee_bps = partner_fee_bps.unwrap_or(0);
    let surplus_amount_for_partner_fee = if is_sell {
        before_all_fees.buy_amount.clone()
    } else {
        before_all_fees.sell_amount.clone()
    };
    let partner_fee_amount = if partner_fee_bps > 0 {
        (&surplus_amount_for_partner_fee * BigInt::from(partner_fee_bps))
            / BigInt::from(ONE_HUNDRED_BPS)
    } else {
        BigInt::from(0)
    };

    let after_partner_fees = if is_sell {
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

    let slippage_amount = |amount: &BigInt| {
        (amount * BigInt::from(slippage_percent_bps)) / BigInt::from(ONE_HUNDRED_BPS)
    };

    let after_slippage = if is_sell {
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

    let amounts_to_sign = if is_sell {
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

    Ok(QuoteAmountsAndCosts {
        is_sell,
        costs: cow_sdk_core::Costs {
            network_fee: cow_sdk_core::NetworkFee {
                amount_in_sell_currency: network_cost_amount.to_string(),
                amount_in_buy_currency: network_cost_amount_in_buy_currency.to_string(),
            },
            partner_fee: cow_sdk_core::FeeComponent {
                amount: partner_fee_amount.to_string(),
                bps: partner_fee_bps,
            },
            protocol_fee: cow_sdk_core::FeeComponent {
                amount: protocol_fee_amount.to_string(),
                bps: protocol_fee_bps.unwrap_or(0.0).round() as u32,
            },
        },
        before_all_fees: before_all_fees.into_strings(),
        before_network_costs: after_protocol_fees.clone().into_strings(),
        after_protocol_fees: after_protocol_fees.into_strings(),
        after_network_costs: after_network_costs.into_strings(),
        after_partner_fees: after_partner_fees.into_strings(),
        after_slippage: after_slippage.into_strings(),
        amounts_to_sign: amounts_to_sign.into_strings(),
    })
}

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
    let fee_amount =
        suggest_slippage_from_fee(&quote.quote.fee_amount, SLIPPAGE_FEE_MULTIPLIER_PERCENT)?;
    let volume_amount = suggest_slippage_from_volume(
        amounts.is_sell,
        &amounts.before_network_costs.sell_amount,
        &amounts.after_network_costs.sell_amount,
        volume_multiplier_percent.unwrap_or(SLIPPAGE_VOLUME_MULTIPLIER_PERCENT),
    )?;

    let total_slippage = parse_integer("totalSlippage", &fee_amount)?
        + parse_integer("totalSlippage", &volume_amount)?;
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

pub fn partner_fee_bps(partner_fee: Option<&Value>) -> Option<u32> {
    match partner_fee {
        Some(Value::Object(map)) => map
            .get("volumeBps")
            .and_then(Value::as_u64)
            .and_then(|value| u32::try_from(value).ok()),
        Some(Value::Array(items)) => items.iter().find_map(|item| partner_fee_bps(Some(item))),
        _ => None,
    }
}

pub(crate) fn gas_with_margin(gas: &str) -> Result<String, TradingError> {
    let gas = parse_integer("gas", gas)?;
    let margin = (&gas * BigInt::from(GAS_MARGIN_PERCENT)) / BigInt::from(100);
    Ok((gas + margin).to_string())
}

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
        return Err(TradingError::InvalidInput(format!(
            "{field} must be non-negative: {percent}"
        )));
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
    let fee_amount = parse_integer("feeAmount", &quote.fee_amount)?;
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
    sell_amount_before_network_costs: &str,
    sell_amount_after_network_costs: &str,
    slippage: &str,
) -> Result<BigInt, TradingError> {
    let sell_before = parse_integer(
        "sellAmountBeforeNetworkCosts",
        sell_amount_before_network_costs,
    )?;
    let sell_after = parse_integer(
        "sellAmountAfterNetworkCosts",
        sell_amount_after_network_costs,
    )?;
    let slippage = parse_integer("slippage", slippage)?;
    let sell_amount = if is_sell { sell_after } else { sell_before };

    if sell_amount <= BigInt::from(0) {
        return Err(TradingError::InvalidInput(format!(
            "sellAmount must be greater than 0: {sell_amount}"
        )));
    }
    if slippage < BigInt::from(0) {
        return Err(TradingError::InvalidInput(format!(
            "slippage must be non-negative: {slippage}"
        )));
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

impl AmountsBig {
    fn into_strings(self) -> cow_sdk_core::Amounts<String> {
        cow_sdk_core::Amounts {
            sell_amount: self.sell_amount.to_string(),
            buy_amount: self.buy_amount.to_string(),
        }
    }
}
