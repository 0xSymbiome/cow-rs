use num_bigint::BigInt;

use cow_sdk_app_data::PartnerFee;
use cow_sdk_core::{Amount, OrderKind};
use cow_sdk_orderbook::QuoteData;

use crate::TradingError;

use super::amounts::{parse_integer, rounded_nonnegative_f64_to_u32};
use super::{MAX_SLIPPAGE_BPS, ONE_HUNDRED_BPS};

const PROTOCOL_FEE_BPS_SCALE: i64 = 100_000;
const PERCENT_SCALE: i64 = 1_000_000;

/// Extracts the first supported volume-based partner-fee basis-point value from the typed
/// partner-fee payload.
#[must_use]
pub fn partner_fee_bps(partner_fee: Option<&PartnerFee>) -> Option<u32> {
    partner_fee.and_then(PartnerFee::volume_bps).map(u32::from)
}

pub(super) fn parse_percent_scaled(
    percent: f64,
    field: &'static str,
) -> Result<BigInt, TradingError> {
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
            value: rendered.clone().into(),
        })?;
    let fractional = BigInt::parse_bytes(fractional.as_bytes(), 10).ok_or_else(|| {
        TradingError::InvalidNumeric {
            field,
            value: rendered.clone().into(),
        }
    })?;

    Ok(whole * BigInt::from(PERCENT_SCALE) + fractional)
}

pub(super) fn apply_percentage(amount: &BigInt, scaled_percent: BigInt) -> BigInt {
    let denominator = BigInt::from(100 * PERCENT_SCALE);
    let numerator = amount * scaled_percent;
    (numerator + (&denominator / 2)) / denominator
}

pub(super) fn get_protocol_fee_amount(
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

    let sell_amount = parse_integer("sellAmount", &quote.sell_amount.to_string())?;
    let buy_amount = parse_integer("buyAmount", &quote.buy_amount.to_string())?;
    let fee_amount = parse_integer("feeAmount", &quote.network_cost_amount().to_string())?;
    let denominator_base = BigInt::from(ONE_HUNDRED_BPS * PROTOCOL_FEE_BPS_SCALE);

    if quote.kind == OrderKind::Sell {
        let denominator = &denominator_base - &protocol_fee_bps_big;
        Ok((buy_amount * protocol_fee_bps_big) / denominator)
    } else {
        let denominator = &denominator_base + &protocol_fee_bps_big;
        Ok(((sell_amount + fee_amount) * protocol_fee_bps_big) / denominator)
    }
}

pub(super) fn get_slippage_percent_scaled(
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

pub(super) fn scaled_percent_to_bps(percent_scaled: &BigInt) -> Result<u32, TradingError> {
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
            value: value.into(),
        })
}

pub(super) struct QuoteFeeBreakdown {
    pub(super) network_cost_amount: BigInt,
    pub(super) network_cost_amount_in_buy_currency: BigInt,
    pub(super) partner_fee_amount: BigInt,
    pub(super) partner_fee_bps: u32,
    pub(super) protocol_fee_amount: BigInt,
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
