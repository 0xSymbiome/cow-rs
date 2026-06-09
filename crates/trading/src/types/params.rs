#![allow(
    clippy::redundant_pub_crate,
    reason = "the quote-request override helpers intentionally stay pub(crate) and are re-exported through types::mod for unchanged crate-local call sites"
)]

//! Trade, trader, allowance, options, override, and advanced-settings parameter
//! types accepted by the trading helpers, plus their crate-internal
//! quote-request override appliers.

use std::{fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use cow_sdk_app_data::{AppDataParams, PartnerFee};
use cow_sdk_core::{
    Address, AddressPerChain, Amount, AppCode, AppCodeError, BuyTokenDestination, CowEnv, HexData,
    OrderKind, OrderUid, SellTokenSource, SupportedChainId,
};
use cow_sdk_orderbook::{OrderbookClient, PriceQuality, SigningScheme};
use cow_sdk_signing::eip1271::Eip1271SignatureProvider;

use super::seams::{EthFlowOrderExistsChecker, SlippageSuggestionProvider};
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

/// Fully resolved trader configuration used by order-posting and on-chain flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TraderParameters {
    /// Active chain id for the workflow.
    pub chain_id: SupportedChainId,
    /// App code written into generated app-data documents.
    pub app_code: AppCode,
    /// Optional environment override for endpoint and contract resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

impl TraderParameters {
    /// Creates trader parameters with the required chain and app-code fields.
    ///
    /// # Errors
    ///
    /// Returns [`AppCodeError`] when `app_code` is empty or contains
    /// forbidden control characters.
    pub fn new<T>(chain_id: SupportedChainId, app_code: T) -> Result<Self, AppCodeError>
    where
        T: TryInto<AppCode>,
        T::Error: Into<AppCodeError>,
    {
        Ok(Self {
            chain_id,
            app_code: app_code.try_into().map_err(Into::into)?,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        })
    }

    /// Returns a copy with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with settlement-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_settlement_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.settlement_contract_override = Some(overrides);
        self
    }

    /// Returns a copy with `EthFlow`-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_eth_flow_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.eth_flow_contract_override = Some(overrides);
        self
    }
}

/// Partial trader defaults stored on [`crate::Trading`] and its builder.
///
/// Carries the protocol-resolution defaults a `Trading` instance
/// applies when call-level parameters omit them: chain id, app code,
/// environment, settlement-contract overrides, and `EthFlow`-contract
/// overrides. The SDK does not store a default owner; per-call
/// [`crate::TradeParameters::owner`] (with the signer's address as the
/// implicit fallback for signer-backed flows, or
/// `advanced_settings.quote_request.from` for quote-only flows) is the
/// sole owner source.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct PartialTraderParameters {
    /// Default chain id when call-level params omit it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) chain_id: Option<SupportedChainId>,
    /// Default app code written into generated app-data documents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) app_code: Option<AppCode>,
    /// Default environment for endpoint and contract resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) eth_flow_contract_override: Option<AddressPerChain>,
}

/// Quoter configuration used by quote-only and quote-and-sign flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoterParameters {
    /// Active chain id for the workflow.
    pub chain_id: SupportedChainId,
    /// App code written into generated app-data documents.
    pub app_code: AppCode,
    /// Effective account used for quote ownership.
    pub account: Address,
    /// Optional environment override for endpoint and contract resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

impl QuoterParameters {
    /// Creates quoter parameters with the required chain, app-code, and account fields.
    ///
    /// # Errors
    ///
    /// Returns [`AppCodeError`] when `app_code` is empty or contains
    /// forbidden control characters.
    pub fn new<T>(
        chain_id: SupportedChainId,
        app_code: T,
        account: Address,
    ) -> Result<Self, AppCodeError>
    where
        T: TryInto<AppCode>,
        T::Error: Into<AppCodeError>,
    {
        Ok(Self {
            chain_id,
            app_code: app_code.try_into().map_err(Into::into)?,
            account,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        })
    }

    /// Returns a copy with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with settlement-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_settlement_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.settlement_contract_override = Some(overrides);
        self
    }

    /// Returns a copy with `EthFlow`-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_eth_flow_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.eth_flow_contract_override = Some(overrides);
        self
    }
}

/// Parameters for order lookup, cancellation, and on-chain helper flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OrderTraderParameters {
    /// Target order UID.
    pub order_uid: OrderUid,
    /// Optional chain-id override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Optional environment override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

impl OrderTraderParameters {
    /// Creates order-trader parameters with the required order UID.
    #[must_use]
    pub const fn new(order_uid: OrderUid) -> Self {
        Self {
            order_uid,
            chain_id: None,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        }
    }

    /// Returns a copy with an explicit chain-id override.
    #[must_use]
    pub const fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with settlement-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_settlement_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.settlement_contract_override = Some(overrides);
        self
    }

    /// Returns a copy with `EthFlow`-contract overrides keyed by chain id.
    #[must_use]
    pub fn with_eth_flow_contract_override(mut self, overrides: AddressPerChain) -> Self {
        self.eth_flow_contract_override = Some(overrides);
        self
    }
}

/// Parameters for allowance-check helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct AllowanceParameters {
    /// ERC-20 token address.
    pub token_address: Address,
    /// Owner whose allowance should be inspected.
    pub owner: Address,
    /// Optional chain-id override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Optional environment override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional explicit vault-relayer deployment override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_relayer_override: Option<Address>,
}

impl AllowanceParameters {
    /// Creates allowance parameters with the required token and owner fields.
    #[must_use]
    pub const fn new(token_address: Address, owner: Address) -> Self {
        Self {
            token_address,
            owner,
            chain_id: None,
            env: None,
            vault_relayer_override: None,
        }
    }

    /// Returns a copy with an explicit chain-id override.
    #[must_use]
    pub const fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with an explicit vault-relayer deployment override.
    #[must_use]
    pub const fn with_vault_relayer_override(mut self, address: Address) -> Self {
        self.vault_relayer_override = Some(address);
        self
    }
}

/// Parameters for approval-transaction helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ApprovalParameters {
    /// ERC-20 token address.
    pub token_address: Address,
    /// Approval amount.
    pub amount: Amount,
    /// Optional chain-id override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Optional environment override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional explicit vault-relayer deployment override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_relayer_override: Option<Address>,
}

impl ApprovalParameters {
    /// Creates approval parameters with the required token and amount fields.
    #[must_use]
    pub const fn new(token_address: Address, amount: Amount) -> Self {
        Self {
            token_address,
            amount,
            chain_id: None,
            env: None,
            vault_relayer_override: None,
        }
    }

    /// Returns a copy with an explicit chain-id override.
    #[must_use]
    pub const fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy with an explicit environment override.
    #[must_use]
    pub const fn with_env(mut self, env: CowEnv) -> Self {
        self.env = Some(env);
        self
    }

    /// Returns a copy with an explicit vault-relayer deployment override.
    #[must_use]
    pub const fn with_vault_relayer_override(mut self, address: Address) -> Self {
        self.vault_relayer_override = Some(address);
        self
    }
}

/// Options stored on [`crate::Trading`] that do not belong in trader defaults.
#[derive(Clone, Default)]
pub struct TradingOptions {
    order_book_api: Option<Arc<dyn OrderbookClient>>,
}

impl fmt::Debug for TradingOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradingOptions")
            .field("order_book_api", &self.order_book_api.is_some())
            .finish()
    }
}

impl TradingOptions {
    /// Creates an empty options bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of these options with an injected orderbook client.
    ///
    /// The injected client fixes chain and environment for orderbook-bound
    /// flows and carries its own [`TransportPolicy`] (retry, rate-limit, and
    /// HTTP-client tuning). Configure that resilience on the client before
    /// injecting it — build it through
    /// [`OrderbookApi::builder().transport_policy(...)`] — rather than on the
    /// trading options. On the default construction path (no client injected),
    /// the SDK builds an orderbook client with the standard
    /// [`TransportPolicy::default_orderbook`] policy.
    ///
    /// [`TransportPolicy`]: cow_sdk_core::transport::policy::TransportPolicy
    /// [`OrderbookApi::builder().transport_policy(...)`]: cow_sdk_orderbook::OrderbookApiBuilder::transport_policy
    /// [`TransportPolicy::default_orderbook`]: cow_sdk_core::transport::policy::TransportPolicy::default_orderbook
    #[must_use]
    pub fn with_orderbook_client(mut self, orderbook_client: Arc<dyn OrderbookClient>) -> Self {
        self.order_book_api = Some(orderbook_client);
        self
    }

    /// Returns a copy of these options with an injected orderbook client by value.
    ///
    /// Shares the client internally, so callers do not wrap it in [`Arc`]. Use
    /// [`TradingOptions::with_orderbook_client`] when an
    /// `Arc<dyn OrderbookClient>` is already held and is shared elsewhere.
    #[must_use]
    pub fn with_orderbook(self, orderbook: impl OrderbookClient + 'static) -> Self {
        self.with_orderbook_client(Arc::new(orderbook))
    }

    /// Returns the injected orderbook client, if one is configured.
    #[must_use]
    pub fn orderbook_client(&self) -> Option<Arc<dyn OrderbookClient>> {
        self.order_book_api.clone()
    }
}

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
    pub const fn with_sell_token(mut self, sell_token: Address) -> Self {
        self.sell_token = Some(sell_token);
        self
    }

    /// Returns a copy with an explicit buy-token replacement.
    #[must_use]
    pub const fn with_buy_token(mut self, buy_token: Address) -> Self {
        self.buy_token = Some(buy_token);
        self
    }

    /// Returns a copy with an explicit receiver replacement.
    #[must_use]
    pub const fn with_receiver(mut self, receiver: Address) -> Self {
        self.receiver = Some(receiver);
        self
    }

    /// Returns a copy with an explicit quote owner.
    #[must_use]
    pub const fn with_from(mut self, from: Address) -> Self {
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

pub(crate) const fn apply_quote_request_parameter_overrides(
    targets: &mut QuoteRequestParameterTargets<'_>,
    request_override: Option<&QuoteRequestOverride>,
) {
    let Some(request_override) = request_override else {
        return;
    };

    if let Some(sell_token_override) = &request_override.sell_token {
        *targets.sell_token = *sell_token_override;
    }
    if let Some(buy_token_override) = &request_override.buy_token {
        *targets.buy_token = *buy_token_override;
    }
    if let Some(receiver_override) = &request_override.receiver {
        *targets.receiver = Some(*receiver_override);
    }
    if let Some(from_override) = &request_override.from {
        *targets.owner = Some(*from_override);
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

/// Optional knobs applied after quoting and before final submission.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct PostTradeAdditionalParams {
    /// Optional existence checker used by `EthFlow` unique-order-id generation.
    pub check_eth_flow_order_exists: Option<Arc<dyn EthFlowOrderExistsChecker>>,
    /// Optional network cost amount folded into amount calculations.
    pub network_costs_amount: Option<Amount>,
    /// Explicit signing scheme override for submission.
    pub signing_scheme: Option<SigningScheme>,
    /// Optional custom EIP-1271 signer for smart-account signatures.
    pub custom_eip1271_signature: Option<Arc<dyn Eip1271SignatureProvider>>,
    /// Whether costs, slippage, and fees should be applied when building the order payload.
    pub apply_costs_slippage_and_fees: Option<bool>,
}

impl PostTradeAdditionalParams {
    /// Creates an empty post-trade additional-parameter bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy with an explicit `EthFlow` existence checker.
    #[must_use]
    pub fn with_check_eth_flow_order_exists(
        mut self,
        checker: Arc<dyn EthFlowOrderExistsChecker>,
    ) -> Self {
        self.check_eth_flow_order_exists = Some(checker);
        self
    }

    /// Returns a copy with an explicit network-costs amount.
    #[must_use]
    pub const fn with_network_costs_amount(mut self, amount: Amount) -> Self {
        self.network_costs_amount = Some(amount);
        self
    }

    /// Returns a copy with an explicit signing-scheme override.
    #[must_use]
    pub const fn with_signing_scheme(mut self, scheme: SigningScheme) -> Self {
        self.signing_scheme = Some(scheme);
        self
    }

    /// Returns a copy with a custom EIP-1271 signature provider.
    #[must_use]
    pub fn with_custom_eip1271_signature(
        mut self,
        provider: Arc<dyn Eip1271SignatureProvider>,
    ) -> Self {
        self.custom_eip1271_signature = Some(provider);
        self
    }

    /// Returns a copy with an explicit cost/slippage/fee application flag.
    #[must_use]
    pub const fn with_apply_costs_slippage_and_fees(mut self, apply: bool) -> Self {
        self.apply_costs_slippage_and_fees = Some(apply);
        self
    }
}

impl fmt::Debug for PostTradeAdditionalParams {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PostTradeAdditionalParams")
            .field(
                "check_eth_flow_order_exists",
                &self.check_eth_flow_order_exists.is_some(),
            )
            .field("network_costs_amount", &self.network_costs_amount)
            .field("signing_scheme", &self.signing_scheme)
            .field(
                "custom_eip1271_signature",
                &self.custom_eip1271_signature.is_some(),
            )
            .field(
                "apply_costs_slippage_and_fees",
                &self.apply_costs_slippage_and_fees,
            )
            .finish()
    }
}

/// Advanced settings shared by swap and limit-order quote and post workflows.
///
/// Limit-order flows leave `slippage_suggester` as `None` because the
/// limit submission path does not apply slippage in the same shape as
/// swaps; the field is documented but unused on that flow.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct TradeAdvancedSettings {
    /// Optional direct orderbook quote-request overrides.
    pub quote_request: Option<QuoteRequestOverride>,
    /// Optional app-data overrides merged into generated app-data documents.
    pub app_data: Option<AppDataParams>,
    /// Optional submission-time behavior overrides.
    pub additional_params: Option<PostTradeAdditionalParams>,
    /// Optional custom slippage-suggestion provider.
    ///
    /// Ignored on limit-order flows; limit orders do not apply
    /// slippage in the same shape as swaps.
    pub slippage_suggester: Option<Arc<dyn SlippageSuggestionProvider>>,
}

impl TradeAdvancedSettings {
    /// Creates an empty advanced-settings bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy with explicit quote-request overrides attached.
    #[must_use]
    pub const fn with_quote_request(mut self, overrides: QuoteRequestOverride) -> Self {
        self.quote_request = Some(overrides);
        self
    }

    /// Returns a copy with explicit app-data overrides attached.
    #[must_use]
    pub fn with_app_data(mut self, app_data: AppDataParams) -> Self {
        self.app_data = Some(app_data);
        self
    }

    /// Returns a copy with explicit submission-time additional parameters attached.
    #[must_use]
    pub fn with_additional_params(mut self, params: PostTradeAdditionalParams) -> Self {
        self.additional_params = Some(params);
        self
    }

    /// Returns a copy with a custom slippage-suggestion provider attached.
    ///
    /// Limit-order flows ignore this provider; only swap quote and
    /// post flows read it.
    #[must_use]
    pub fn with_slippage_suggester(
        mut self,
        suggester: Arc<dyn SlippageSuggestionProvider>,
    ) -> Self {
        self.slippage_suggester = Some(suggester);
        self
    }
}

impl fmt::Debug for TradeAdvancedSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradeAdvancedSettings")
            .field("quote_request", &self.quote_request)
            .field("app_data", &self.app_data)
            .field("additional_params", &self.additional_params)
            .field("slippage_suggester", &self.slippage_suggester.is_some())
            .finish()
    }
}

/// Explicit verifier and signature payload for EIP-1271 verification helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct Eip1271VerificationParameters {
    /// Smart-account verifier address.
    pub verifier: Address,
    /// Signature bytes supplied to the verifier contract.
    pub signature: HexData,
}

impl Eip1271VerificationParameters {
    /// Creates explicit verifier and signature payload for EIP-1271 verification helpers.
    #[must_use]
    pub const fn new(verifier: Address, signature: HexData) -> Self {
        Self {
            verifier,
            signature,
        }
    }
}
