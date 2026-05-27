use serde::{Deserialize, Serialize};

use cow_sdk_app_data::PartnerFee;
use cow_sdk_core::{
    Address, AddressPerChain, Amount, BuyTokenDestination, CowEnv, OrderKind, SellTokenSource,
};

use crate::TradingError;

const fn default_sell_token_source() -> SellTokenSource {
    SellTokenSource::Erc20
}

const fn default_buy_token_destination() -> BuyTokenDestination {
    BuyTokenDestination::Erc20
}

/// Internal definition shared by the public trade-parameter structs so
/// the `with_*` setters whose bodies match across both types live in
/// one place. Each invocation emits inherent methods on the target
/// struct that are indistinguishable from hand-written setters; the
/// public API shape is preserved exactly.
macro_rules! impl_common_trade_setters {
    ($target:ty) => {
        impl $target {
            /// Returns a copy with an explicit owner override.
            #[must_use]
            pub const fn with_owner(mut self, owner: Address) -> Self {
                self.owner = Some(owner);
                self
            }

            /// Returns a copy with an explicit environment override.
            #[must_use]
            pub const fn with_env(mut self, env: CowEnv) -> Self {
                self.env = Some(env);
                self
            }

            /// Returns a copy with an explicit receiver override.
            #[must_use]
            pub const fn with_receiver(mut self, receiver: Address) -> Self {
                self.receiver = Some(receiver);
                self
            }

            /// Returns a copy with an explicit slippage tolerance in basis points.
            #[must_use]
            pub const fn with_slippage_bps(mut self, slippage_bps: u32) -> Self {
                self.slippage_bps = Some(slippage_bps);
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

            /// Returns a copy with the partial-fill flag set.
            #[must_use]
            pub const fn with_partially_fillable(mut self, partially_fillable: bool) -> Self {
                self.partially_fillable = partially_fillable;
                self
            }

            /// Returns a copy with an explicit sell-token balance source.
            #[must_use]
            pub const fn with_sell_token_balance(mut self, balance: SellTokenSource) -> Self {
                self.sell_token_balance = balance;
                self
            }

            /// Returns a copy with an explicit buy-token balance destination.
            #[must_use]
            pub const fn with_buy_token_balance(mut self, balance: BuyTokenDestination) -> Self {
                self.buy_token_balance = balance;
                self
            }

            /// Returns a copy with an explicit partner-fee entry.
            #[must_use]
            pub fn with_partner_fee(mut self, partner_fee: PartnerFee) -> Self {
                self.partner_fee = Some(partner_fee);
                self
            }
        }
    };
}

/// Swap-style trade request accepted by quote and post helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TradeParameters {
    /// Order kind.
    pub kind: OrderKind,
    /// Optional owner override. Signer address becomes the fallback in signer-backed flows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Amount interpreted according to `kind`.
    pub amount: Amount,
    /// Optional environment override for endpoint and contract resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source preserved through quote and post flows.
    #[serde(default = "default_sell_token_source")]
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination preserved through quote and post flows.
    #[serde(default = "default_buy_token_destination")]
    pub buy_token_balance: BuyTokenDestination,
    /// Optional explicit slippage tolerance in basis points.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    /// Optional receiver override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Optional relative validity duration in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    /// Optional absolute UNIX expiry timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    /// Optional partner-fee metadata merged into app-data and fee calculations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_fee: Option<PartnerFee>,
}

impl TradeParameters {
    /// Creates a swap-style trade request with the required trade fields.
    #[must_use]
    pub const fn new(
        kind: OrderKind,
        sell_token: Address,
        buy_token: Address,
        amount: Amount,
    ) -> Self {
        Self {
            kind,
            owner: None,
            sell_token,
            buy_token,
            amount,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
            partially_fillable: false,
            sell_token_balance: default_sell_token_source(),
            buy_token_balance: default_buy_token_destination(),
            slippage_bps: None,
            receiver: None,
            valid_for: None,
            valid_to: None,
            partner_fee: None,
        }
    }
}

impl_common_trade_setters!(TradeParameters);

/// Limit-order request accepted by posting and signing helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LimitTradeParameters {
    /// Order kind.
    pub kind: OrderKind,
    /// Optional owner override. Signer address becomes the fallback in signer-backed flows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    /// Sell-token address.
    pub sell_token: Address,
    /// Buy-token address.
    pub buy_token: Address,
    /// Sell amount before transformations.
    pub sell_amount: Amount,
    /// Buy amount before transformations.
    pub buy_amount: Amount,
    /// Optional quote id required by some flows such as `EthFlow` posting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_id: Option<i64>,
    /// Optional environment override for endpoint and contract resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
    /// Whether partial fills are allowed.
    #[serde(default)]
    pub partially_fillable: bool,
    /// Sell-token balance source preserved through final order construction.
    #[serde(default = "default_sell_token_source")]
    pub sell_token_balance: SellTokenSource,
    /// Buy-token balance destination preserved through final order construction.
    #[serde(default = "default_buy_token_destination")]
    pub buy_token_balance: BuyTokenDestination,
    /// Optional explicit slippage tolerance in basis points.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
    /// Optional receiver override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver: Option<Address>,
    /// Optional relative validity duration in seconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_for: Option<u32>,
    /// Optional absolute UNIX expiry timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<u32>,
    /// Optional partner-fee metadata merged into app-data and fee calculations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partner_fee: Option<PartnerFee>,
}

impl LimitTradeParameters {
    /// Creates a limit-order request with the required trade fields.
    #[must_use]
    pub const fn new(
        kind: OrderKind,
        sell_token: Address,
        buy_token: Address,
        sell_amount: Amount,
        buy_amount: Amount,
    ) -> Self {
        Self {
            kind,
            owner: None,
            sell_token,
            buy_token,
            sell_amount,
            buy_amount,
            quote_id: None,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
            partially_fillable: false,
            sell_token_balance: default_sell_token_source(),
            buy_token_balance: default_buy_token_destination(),
            slippage_bps: None,
            receiver: None,
            valid_for: None,
            valid_to: None,
            partner_fee: None,
        }
    }

    /// Returns a copy with an explicit quote id.
    #[must_use]
    pub const fn with_quote_id(mut self, quote_id: i64) -> Self {
        self.quote_id = Some(quote_id);
        self
    }
}

impl_common_trade_setters!(LimitTradeParameters);

/// Limit-order request derived from a quote response.
///
/// Carries a non-`None` quote id by construction. Produced exclusively
/// by [`crate::swap_params_to_limit_order_params`] and accepted by the
/// `EthFlow` native-currency submission entry and the `EthFlow`
/// transaction helper so the quote-id requirement is enforced at the
/// type system rather than as a runtime check on the submission path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct LimitTradeParametersFromQuote {
    #[serde(flatten)]
    inner: LimitTradeParameters,
}

impl LimitTradeParametersFromQuote {
    /// Builds the newtype from a [`LimitTradeParameters`] value.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError::MissingQuoteId`] when the source value
    /// carries `quote_id = None`. The error label preserves the public
    /// diagnostic shape consumers observed before this type was
    /// introduced.
    pub fn try_from_limit(inner: LimitTradeParameters) -> Result<Self, TradingError> {
        if inner.quote_id.is_none() {
            return Err(TradingError::MissingQuoteId("EthFlow order posting"));
        }
        Ok(Self { inner })
    }

    /// Returns the quote id. Guaranteed non-`None` by construction.
    ///
    /// # Panics
    ///
    /// Statically unreachable. Every constructor path rejects
    /// `quote_id = None` before producing a value of this type, and
    /// the inner value is private so the field cannot become `None`
    /// after construction.
    #[must_use]
    pub const fn quote_id(&self) -> i64 {
        // SAFETY: try_from_limit rejects None on entry and is the only
        // public constructor; the inner field is private and immutable
        // through the public API so the invariant cannot be broken.
        self.inner.quote_id.expect(
            "LimitTradeParametersFromQuote invariant: quote_id is always Some by construction",
        )
    }

    /// Returns a reference to the underlying limit-trade parameters.
    #[must_use]
    pub const fn as_limit(&self) -> &LimitTradeParameters {
        &self.inner
    }

    /// Consumes the newtype and returns the underlying value.
    #[must_use]
    pub fn into_limit(self) -> LimitTradeParameters {
        self.inner
    }
}

impl AsRef<LimitTradeParameters> for LimitTradeParametersFromQuote {
    fn as_ref(&self) -> &LimitTradeParameters {
        &self.inner
    }
}
