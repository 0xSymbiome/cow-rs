use alloy_primitives::aliases::I512;

use cow_sdk_app_data::PartnerFee;
use cow_sdk_core::{Amount, OrderKind};
use cow_sdk_orderbook::QuoteData;

use crate::TradingError;

use super::amounts::{parse_integer, rounded_nonnegative_f64_to_u32};
use super::{MAX_SLIPPAGE_BPS, ONE_HUNDRED_BPS};

const PROTOCOL_FEE_BPS_SCALE: i64 = 100_000;
const PERCENT_SCALE: i64 = 1_000_000;

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

/// Extracts the first supported volume-based partner-fee basis-point value from the typed
/// partner-fee payload.
#[must_use]
pub fn partner_fee_bps(partner_fee: Option<&PartnerFee>) -> Option<u32> {
    partner_fee.and_then(PartnerFee::volume_bps).map(u32::from)
}

/// Lifts `percent` (a non-negative finite f64) into the
/// `PERCENT_SCALE`-scaled signed-512-bit integer the cow slippage math
/// expects, applying the upstream TypeScript SDK's
/// `Math.floor(p * 1e6)` truncation semantics byte-identically.
///
/// The prior `format!("{p:.6}")`-based path applied round-half-to-even
/// at the 6th decimal place, which diverged from `Math.floor` for
/// high-precision floats. Cow protocol-fee strings on the wire are
/// always clean decimals (`"0.5"`, `"1.5"`) that survive either
/// rounding mode, but the cascade aligns the cow surface with the TS
/// upstream for parity-by-construction.
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
/// settlement math expects, applying the same round-half-away-from-zero
/// step the upstream TypeScript SDK uses (`Math.round(p * 1e5)`). The
/// previous cow path went through `parse_percent_scaled(p) / 10`, which
/// composed round-half-to-even at the 6th decimal place with truncation
/// by 10; that diverged from the TS rounding mode for sub-permille
/// protocol fees with non-zero precision beyond the 5th decimal. Wire
/// `protocol_fee_bps` strings from the cow orderbook API are always
/// clean decimals that survive either rounding mode, but the cascade
/// aligns the cow Rust surface with the TS upstream for
/// parity-by-construction.
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
