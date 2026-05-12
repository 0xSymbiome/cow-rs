#![allow(
    clippy::redundant_pub_crate,
    reason = "these override helpers intentionally stay pub(crate) and are re-exported through types::mod for unchanged crate-local call sites"
)]

use serde::{Deserialize, Serialize};
use serde_json::Value;

use cow_sdk_app_data::{AppDataParams, PartnerFee};
use cow_sdk_core::{Address, BuyTokenDestination, SellTokenSource};
use cow_sdk_orderbook::{PriceQuality, SigningScheme};

use crate::TradingError;

/// Optional overrides applied directly to the orderbook quote request.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteRequestOverride {
    /// Replacement sell-token address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token: Option<Address>,
    /// Replacement buy-token address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token: Option<Address>,
    /// Replacement receiver address.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Replacement relative validity duration in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    /// Replacement absolute UNIX expiry timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    /// Replacement quote owner.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<Address>,
    /// Replacement price-quality mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price_quality: Option<PriceQuality>,
    /// Replacement signing scheme.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signing_scheme: Option<SigningScheme>,
    /// Replacement on-chain order flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub onchain_order: Option<bool>,
    /// Replacement verification gas limit.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verification_gas_limit: Option<u64>,
    /// Replacement timeout in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
    /// Replacement partial-fill flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partially_fillable: Option<bool>,
    /// Replacement sell-token balance source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_token_balance: Option<SellTokenSource>,
    /// Replacement buy-token balance destination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<BuyTokenDestination>,
}

impl QuoteRequestOverride {
    /// Creates an empty quote-request override; populate fields through the `with_*` setters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy with an explicit sell-token replacement.
    #[must_use]
    pub fn with_sell_token(mut self, sell_token: Address) -> Self {
        self.sell_token = Some(sell_token);
        self
    }

    /// Returns a copy with an explicit buy-token replacement.
    #[must_use]
    pub fn with_buy_token(mut self, buy_token: Address) -> Self {
        self.buy_token = Some(buy_token);
        self
    }

    /// Returns a copy with an explicit receiver replacement.
    #[must_use]
    pub fn with_receiver(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// Returns a copy with an explicit quote owner.
    #[must_use]
    pub fn with_from(mut self, from: Address) -> Self {
        self.from = Some(from);
        self
    }

    /// Returns a copy with an explicit absolute expiry timestamp.
    #[must_use]
    pub const fn with_valid_to(mut self, valid_to: u32) -> Self {
        self.valid_to = Some(valid_to);
        self
    }

    /// Returns a copy with an explicit relative validity duration in seconds.
    #[must_use]
    pub const fn with_valid_for(mut self, valid_for: u32) -> Self {
        self.valid_for = Some(valid_for);
        self
    }

    /// Returns a copy with an explicit price-quality replacement.
    #[must_use]
    pub const fn with_price_quality(mut self, price_quality: PriceQuality) -> Self {
        self.price_quality = Some(price_quality);
        self
    }

    /// Returns a copy with an explicit signing-scheme replacement.
    #[must_use]
    pub const fn with_signing_scheme(mut self, scheme: SigningScheme) -> Self {
        self.signing_scheme = Some(scheme);
        self
    }

    /// Returns a copy with an explicit on-chain order flag.
    #[must_use]
    pub const fn with_onchain_order(mut self, onchain: bool) -> Self {
        self.onchain_order = Some(onchain);
        self
    }

    /// Returns a copy with an explicit verification gas limit.
    #[must_use]
    pub const fn with_verification_gas_limit(mut self, limit: u64) -> Self {
        self.verification_gas_limit = Some(limit);
        self
    }

    /// Returns a copy with an explicit partial-fill replacement.
    #[must_use]
    pub const fn with_partially_fillable(mut self, partially_fillable: bool) -> Self {
        self.partially_fillable = Some(partially_fillable);
        self
    }

    /// Returns a copy with an explicit sell-token balance replacement.
    #[must_use]
    pub const fn with_sell_token_balance(mut self, balance: SellTokenSource) -> Self {
        self.sell_token_balance = Some(balance);
        self
    }

    /// Returns a copy with an explicit buy-token balance replacement.
    #[must_use]
    pub const fn with_buy_token_balance(mut self, balance: BuyTokenDestination) -> Self {
        self.buy_token_balance = Some(balance);
        self
    }

    /// Returns a copy with an explicit timeout override.
    #[must_use]
    pub const fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

pub(crate) fn apply_app_data_parameter_overrides(
    slippage_bps: &mut Option<u32>,
    partner_fee: &mut Option<PartnerFee>,
    app_data_override: Option<&AppDataParams>,
) -> Result<(), TradingError> {
    let Some(app_data_override) = app_data_override else {
        return Ok(());
    };

    if let Some(slippage) = app_data_override
        .metadata
        .get("quote")
        .and_then(|quote| quote.get("slippageBips"))
        .and_then(Value::as_u64)
        .and_then(|value| u32::try_from(value).ok())
    {
        *slippage_bps = Some(slippage);
    }

    if let Some(partner_fee_override) = app_data_override.metadata.get("partnerFee") {
        *partner_fee = Some(
            PartnerFee::from_value(partner_fee_override.clone()).map_err(|_| {
                TradingError::InvalidInput {
                    field: "appData.metadata.partnerFee",
                    reason: cow_sdk_core::ValidationReason::BadShape {
                        details: "value must match the partner-fee schema",
                    },
                }
            })?,
        );
    }

    Ok(())
}

pub(crate) struct QuoteRequestParameterTargets<'a> {
    pub owner: &'a mut Option<Address>,
    pub sell_token: &'a mut Address,
    pub buy_token: &'a mut Address,
    pub receiver: &'a mut Option<Address>,
    pub valid_for: &'a mut Option<u32>,
    pub valid_to: &'a mut Option<u32>,
    pub partially_fillable: &'a mut bool,
    pub sell_token_balance: &'a mut SellTokenSource,
    pub buy_token_balance: &'a mut BuyTokenDestination,
}

pub(crate) fn apply_quote_request_parameter_overrides(
    targets: &mut QuoteRequestParameterTargets<'_>,
    request_override: Option<&QuoteRequestOverride>,
) {
    let Some(request_override) = request_override else {
        return;
    };

    if let Some(sell_token_override) = &request_override.sell_token {
        *targets.sell_token = sell_token_override.clone();
    }
    if let Some(buy_token_override) = &request_override.buy_token {
        *targets.buy_token = buy_token_override.clone();
    }
    if let Some(receiver_override) = &request_override.receiver {
        *targets.receiver = Some(receiver_override.clone());
    }
    if let Some(from_override) = &request_override.from {
        *targets.owner = Some(from_override.clone());
    }
    if let Some(valid_for_override) = request_override.valid_for {
        *targets.valid_for = Some(valid_for_override);
        *targets.valid_to = None;
    }
    if let Some(valid_to_override) = request_override.valid_to {
        *targets.valid_to = Some(valid_to_override);
        *targets.valid_for = None;
    }
    if let Some(partially_fillable_override) = request_override.partially_fillable {
        *targets.partially_fillable = partially_fillable_override;
    }
    if let Some(sell_token_balance_override) = request_override.sell_token_balance {
        *targets.sell_token_balance = sell_token_balance_override;
    }
    if let Some(buy_token_balance_override) = request_override.buy_token_balance {
        *targets.buy_token_balance = buy_token_balance_override;
    }
}
