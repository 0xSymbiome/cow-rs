use serde::{Deserialize, Serialize};

use cow_sdk_app_data::PartnerFee;
use cow_sdk_core::{
    Address, AddressPerChain, Amount, BuyTokenDestination, CowEnv, OrderKind, SellTokenSource,
};

const fn default_sell_token_source() -> SellTokenSource {
    SellTokenSource::Erc20
}

const fn default_buy_token_destination() -> BuyTokenDestination {
    BuyTokenDestination::Erc20
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
    /// Sell-token decimals used by higher-level consumers and examples.
    pub sell_token_decimals: u8,
    /// Buy-token address.
    pub buy_token: Address,
    /// Buy-token decimals used by higher-level consumers and examples.
    pub buy_token_decimals: u8,
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
        sell_token_decimals: u8,
        buy_token: Address,
        buy_token_decimals: u8,
        amount: Amount,
    ) -> Self {
        Self {
            kind,
            owner: None,
            sell_token,
            sell_token_decimals,
            buy_token,
            buy_token_decimals,
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
    /// Sell-token decimals used by higher-level consumers and examples.
    pub sell_token_decimals: u8,
    /// Buy-token address.
    pub buy_token: Address,
    /// Buy-token decimals used by higher-level consumers and examples.
    pub buy_token_decimals: u8,
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
        sell_token_decimals: u8,
        buy_token: Address,
        buy_token_decimals: u8,
        sell_amount: Amount,
        buy_amount: Amount,
    ) -> Self {
        Self {
            kind,
            owner: None,
            sell_token,
            sell_token_decimals,
            buy_token,
            buy_token_decimals,
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

    /// Returns a copy with an explicit owner override.
    #[must_use]
    pub const fn with_owner(mut self, owner: Address) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Returns a copy with an explicit quote id.
    #[must_use]
    pub const fn with_quote_id(mut self, quote_id: i64) -> Self {
        self.quote_id = Some(quote_id);
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

/// Compatibility alias for limit-order params derived from a quote.
pub type LimitTradeParametersFromQuote = LimitTradeParameters;
