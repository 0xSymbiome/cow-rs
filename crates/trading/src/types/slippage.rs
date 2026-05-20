use serde::{Deserialize, Serialize};

use cow_sdk_core::{Address, Amount, SupportedChainId};

/// Slippage-suggestion request sent to a custom suggestion provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SlippageToleranceRequest {
    /// Active chain id for the quote.
    pub chain_id: SupportedChainId,
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Effective sell amount after precedence resolution, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sell_amount: Option<Amount>,
    /// Effective buy amount after precedence resolution, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_amount: Option<Amount>,
}

impl SlippageToleranceRequest {
    /// Creates a slippage-tolerance request with the required trade-pair fields.
    #[must_use]
    pub const fn new(chain_id: SupportedChainId, sell_token: Address, buy_token: Address) -> Self {
        Self {
            chain_id,
            sell_token,
            buy_token,
            sell_amount: None,
            buy_amount: None,
        }
    }

    /// Returns a copy of this request with an explicit sell amount.
    #[must_use]
    pub const fn with_sell_amount(mut self, sell_amount: Amount) -> Self {
        self.sell_amount = Some(sell_amount);
        self
    }

    /// Returns a copy of this request with an explicit buy amount.
    #[must_use]
    pub const fn with_buy_amount(mut self, buy_amount: Amount) -> Self {
        self.buy_amount = Some(buy_amount);
        self
    }
}

/// Slippage-suggestion response returned by a custom suggestion provider.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct SlippageToleranceResponse {
    /// Suggested slippage tolerance in basis points.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
}

impl SlippageToleranceResponse {
    /// Creates an empty slippage-tolerance response.
    #[must_use]
    pub const fn new() -> Self {
        Self { slippage_bps: None }
    }

    /// Returns a copy of this response with an explicit suggested slippage value.
    #[must_use]
    pub const fn with_slippage_bps(mut self, slippage_bps: u32) -> Self {
        self.slippage_bps = Some(slippage_bps);
        self
    }
}
