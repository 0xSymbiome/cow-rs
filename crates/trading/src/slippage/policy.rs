use alloy_primitives::aliases::I512;

use cow_sdk_core::{Amount, SupportedChainId};
use cow_sdk_orderbook::{OrderQuoteResponse, PriceQuality};

use crate::{
    QuoterParameters, SlippageToleranceResponse, TradeAdvancedSettings, TradeParameters,
    TradingError,
};

use super::amounts::{calculate_quote_amounts_and_costs, parse_integer};
use super::breakdown::{
    apply_percentage, get_slippage_percent_scaled, parse_percent_scaled, partner_fee_bps,
    scaled_percent_to_bps,
};
use super::{MAX_SLIPPAGE_BPS, default_slippage_bps};

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
    advanced_settings: Option<&TradeAdvancedSettings>,
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
                is_ethflow,
                Some(f64::from(suggested) / 100.0),
            )?),
        }),
        Ok(_) | Err(_) => Ok(crate::SlippageToleranceResponse {
            slippage_bps: Some(default_suggestion),
        }),
    }
}
