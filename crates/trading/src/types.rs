use std::{fmt, sync::Arc};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use cow_sdk_app_data::{AppDataDoc, AppDataParams, PartnerFee};
use cow_sdk_core::{
    Address, AddressPerChain, Amount, ApiContext, AppDataHash, CowEnv, HexData, OrderBalance,
    OrderDigest, OrderKind, OrderUid, QuoteAmountsAndCosts, SupportedChainId, TransactionHash,
    TransactionRequest, UnsignedOrder,
};
use cow_sdk_orderbook::{
    AppDataObject, Order, OrderBookApi, OrderCancellations, OrderCreation, OrderQuoteRequest,
    OrderQuoteResponse, OrderbookError, PriceQuality, SigningScheme,
};
use cow_sdk_signing::OrderTypedData;

use crate::TradingError;

const fn default_order_balance() -> OrderBalance {
    OrderBalance::Erc20
}

/// Fully resolved trader configuration used by order-posting and on-chain flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TraderParameters {
    /// Active chain id for the workflow.
    pub chain_id: SupportedChainId,
    /// App code written into generated app-data documents.
    pub app_code: String,
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
    #[must_use]
    pub fn new(chain_id: SupportedChainId, app_code: impl Into<String>) -> Self {
        Self {
            chain_id,
            app_code: app_code.into(),
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        }
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

/// Partial trader defaults stored on [`crate::TradingSdk`] and its builder.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct PartialTraderParameters {
    /// Default chain id when call-level params omit it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chain_id: Option<SupportedChainId>,
    /// Default app code written into generated app-data documents.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app_code: Option<String>,
    /// Default owner for quote and post flows.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<Address>,
    /// Default environment for endpoint and contract resolution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<CowEnv>,
    /// Optional settlement contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub settlement_contract_override: Option<AddressPerChain>,
    /// Optional `EthFlow` contract overrides keyed by chain id.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub eth_flow_contract_override: Option<AddressPerChain>,
}

impl PartialTraderParameters {
    /// Creates an empty partial-trader-parameters bundle; attach values through the `with_*` setters.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy with an explicit default chain id.
    #[must_use]
    pub const fn with_chain_id(mut self, chain_id: SupportedChainId) -> Self {
        self.chain_id = Some(chain_id);
        self
    }

    /// Returns a copy with an explicit default app code.
    #[must_use]
    pub fn with_app_code(mut self, app_code: impl Into<String>) -> Self {
        self.app_code = Some(app_code.into());
        self
    }

    /// Returns a copy with an explicit default owner.
    #[must_use]
    pub fn with_owner(mut self, owner: Address) -> Self {
        self.owner = Some(owner);
        self
    }

    /// Returns a copy with an explicit default environment.
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

/// Quoter configuration used by quote-only and quote-and-sign flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoterParameters {
    /// Active chain id for the workflow.
    pub chain_id: SupportedChainId,
    /// App code written into generated app-data documents.
    pub app_code: String,
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
    #[must_use]
    pub fn new(chain_id: SupportedChainId, app_code: impl Into<String>, account: Address) -> Self {
        Self {
            chain_id,
            app_code: app_code.into(),
            account,
            env: None,
            settlement_contract_override: None,
            eth_flow_contract_override: None,
        }
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
    #[serde(default = "default_order_balance")]
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance destination preserved through quote and post flows.
    #[serde(default = "default_order_balance")]
    pub buy_token_balance: OrderBalance,
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
    #[allow(clippy::too_many_arguments)]
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
            sell_token_balance: default_order_balance(),
            buy_token_balance: default_order_balance(),
            slippage_bps: None,
            receiver: None,
            valid_for: None,
            valid_to: None,
            partner_fee: None,
        }
    }

    /// Returns a copy with an explicit owner override.
    #[must_use]
    pub fn with_owner(mut self, owner: Address) -> Self {
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
    pub fn with_receiver(mut self, receiver: Address) -> Self {
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
    pub const fn with_sell_token_balance(mut self, balance: OrderBalance) -> Self {
        self.sell_token_balance = balance;
        self
    }

    /// Returns a copy with an explicit buy-token balance destination.
    #[must_use]
    pub const fn with_buy_token_balance(mut self, balance: OrderBalance) -> Self {
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
    #[serde(default = "default_order_balance")]
    pub sell_token_balance: OrderBalance,
    /// Buy-token balance destination preserved through final order construction.
    #[serde(default = "default_order_balance")]
    pub buy_token_balance: OrderBalance,
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
    #[allow(clippy::too_many_arguments)]
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
            sell_token_balance: default_order_balance(),
            buy_token_balance: default_order_balance(),
            slippage_bps: None,
            receiver: None,
            valid_for: None,
            valid_to: None,
            partner_fee: None,
        }
    }

    /// Returns a copy with an explicit owner override.
    #[must_use]
    pub fn with_owner(mut self, owner: Address) -> Self {
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
    pub fn with_receiver(mut self, receiver: Address) -> Self {
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
    pub const fn with_sell_token_balance(mut self, balance: OrderBalance) -> Self {
        self.sell_token_balance = balance;
        self
    }

    /// Returns a copy with an explicit buy-token balance destination.
    #[must_use]
    pub const fn with_buy_token_balance(mut self, balance: OrderBalance) -> Self {
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
/// Compatibility alias for the transaction type returned by trading helpers.
pub type TradingTransactionParams = TransactionRequest;

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
    pub fn with_sell_amount(mut self, sell_amount: Amount) -> Self {
        self.sell_amount = Some(sell_amount);
        self
    }

    /// Returns a copy of this request with an explicit buy amount.
    #[must_use]
    pub fn with_buy_amount(mut self, buy_amount: Amount) -> Self {
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

/// Fully resolved quote result produced by trading quote helpers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteResults {
    /// Effective trade parameters after SDK defaults and advanced settings were applied.
    pub trade_parameters: TradeParameters,
    /// Suggested slippage in basis points after SDK or custom-provider resolution.
    pub suggested_slippage_bps: u32,
    /// Fee and amount breakdown derived from the orderbook quote.
    pub amounts_and_costs: QuoteAmountsAndCosts,
    /// Unsigned order payload produced for signing or on-chain submission.
    pub order_to_sign: UnsignedOrder,
    /// Raw orderbook quote response.
    pub quote_response: OrderQuoteResponse,
    /// App-data document, serialized payload, and digest used by the quote flow.
    pub app_data_info: TradingAppDataInfo,
    /// Originating orderbook runtime binding captured by the quote flow.
    ///
    /// Quote-derived posting requires this binding to match the submission-time
    /// orderbook runtime.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub orderbook_binding: Option<OrderbookRuntimeBinding>,
    /// Typed order-facing envelope kept for consumers while signers use the
    /// lower-level `TypedDataPayload` seam internally.
    pub order_typed_data: OrderTypedData,
}

impl QuoteResults {
    /// Creates a quote-results payload from its required fields.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub const fn new(
        trade_parameters: TradeParameters,
        suggested_slippage_bps: u32,
        amounts_and_costs: QuoteAmountsAndCosts,
        order_to_sign: UnsignedOrder,
        quote_response: OrderQuoteResponse,
        app_data_info: TradingAppDataInfo,
        order_typed_data: OrderTypedData,
    ) -> Self {
        Self {
            trade_parameters,
            suggested_slippage_bps,
            amounts_and_costs,
            order_to_sign,
            quote_response,
            app_data_info,
            orderbook_binding: None,
            order_typed_data,
        }
    }

    /// Returns a copy with an explicit orderbook runtime binding attached.
    #[must_use]
    pub fn with_orderbook_binding(mut self, binding: OrderbookRuntimeBinding) -> Self {
        self.orderbook_binding = Some(binding);
        self
    }
}

/// Runtime binding captured from an orderbook client for quote-derived workflows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OrderbookRuntimeBinding {
    /// Chain id fixed by the orderbook client.
    pub chain_id: SupportedChainId,
    /// Environment fixed by the orderbook client.
    pub env: CowEnv,
    /// Resolved base URL used by the orderbook client when it is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_base_url: Option<String>,
}

impl OrderbookRuntimeBinding {
    /// Creates a runtime binding with the required chain and environment identifiers.
    #[must_use]
    pub const fn new(chain_id: SupportedChainId, env: CowEnv) -> Self {
        Self {
            chain_id,
            env,
            resolved_base_url: None,
        }
    }

    /// Returns a copy of this binding with an explicit resolved base URL.
    #[must_use]
    pub fn with_resolved_base_url(mut self, url: impl Into<String>) -> Self {
        self.resolved_base_url = Some(url.into());
        self
    }
}

/// Result returned after submitting a trade or transaction-producing flow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OrderPostingResult {
    /// Final order UID.
    pub order_id: OrderUid,
    /// Transaction hash when the flow submits an on-chain transaction directly.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tx_hash: Option<TransactionHash>,
    /// Signature scheme used for the posted order.
    pub signing_scheme: SigningScheme,
    /// Signature payload sent to the orderbook, or empty string for transaction-only flows.
    pub signature: String,
    /// Unsigned order payload used for signing or transaction generation.
    pub order_to_sign: UnsignedOrder,
}

impl OrderPostingResult {
    /// Creates a posting result with the required identity and payload fields.
    #[must_use]
    pub fn new(
        order_id: OrderUid,
        signing_scheme: SigningScheme,
        signature: impl Into<String>,
        order_to_sign: UnsignedOrder,
    ) -> Self {
        Self {
            order_id,
            tx_hash: None,
            signing_scheme,
            signature: signature.into(),
            order_to_sign,
        }
    }

    /// Returns a copy of this result with an explicit transaction hash.
    #[must_use]
    pub fn with_tx_hash(mut self, tx_hash: TransactionHash) -> Self {
        self.tx_hash = Some(tx_hash);
        self
    }
}

/// App-data bundle used by trading quote and post helpers.
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `doc: AppDataDoc` field is a `serde_json::Value` alias, and `serde_json::Value` does not implement `Eq`"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TradingAppDataInfo {
    /// Parsed app-data document.
    pub doc: AppDataDoc,
    /// Canonically serialized app-data payload.
    pub full_app_data: String,
    /// Keccak-256 digest used in protocol order payloads.
    pub app_data_keccak256: AppDataHash,
}

impl TradingAppDataInfo {
    /// Creates a trading app-data bundle from its component fields.
    #[must_use]
    pub fn new(doc: AppDataDoc, full_app_data: impl Into<String>, hash: AppDataHash) -> Self {
        Self {
            doc,
            full_app_data: full_app_data.into(),
            app_data_keccak256: hash,
        }
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
    pub sell_token_balance: Option<OrderBalance>,
    /// Replacement buy-token balance destination.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buy_token_balance: Option<OrderBalance>,
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
    pub const fn with_sell_token_balance(mut self, balance: OrderBalance) -> Self {
        self.sell_token_balance = Some(balance);
        self
    }

    /// Returns a copy with an explicit buy-token balance replacement.
    #[must_use]
    pub const fn with_buy_token_balance(mut self, balance: OrderBalance) -> Self {
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
    pub fn with_network_costs_amount(mut self, amount: Amount) -> Self {
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

/// Explicit verifier and signature payload for EIP-1271 verification helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Eip1271VerificationParameters {
    /// Smart-account verifier address.
    pub verifier: Address,
    /// Signature bytes supplied to the verifier contract.
    pub signature: HexData,
}

/// Advanced settings for swap quote and post flows.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct SwapAdvancedSettings {
    /// Optional direct orderbook quote-request overrides.
    pub quote_request: Option<QuoteRequestOverride>,
    /// Optional app-data overrides merged into generated app-data documents.
    pub app_data: Option<AppDataParams>,
    /// Optional submission-time behavior overrides.
    pub additional_params: Option<PostTradeAdditionalParams>,
    /// Optional custom slippage-suggestion provider.
    pub slippage_suggester: Option<Arc<dyn SlippageSuggestionProvider>>,
}

impl SwapAdvancedSettings {
    /// Creates an empty swap-advanced-settings bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy with explicit quote-request overrides attached.
    #[must_use]
    pub fn with_quote_request(mut self, overrides: QuoteRequestOverride) -> Self {
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
    #[must_use]
    pub fn with_slippage_suggester(
        mut self,
        suggester: Arc<dyn SlippageSuggestionProvider>,
    ) -> Self {
        self.slippage_suggester = Some(suggester);
        self
    }
}

impl fmt::Debug for SwapAdvancedSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SwapAdvancedSettings")
            .field("quote_request", &self.quote_request)
            .field("app_data", &self.app_data)
            .field("additional_params", &self.additional_params)
            .field("slippage_suggester", &self.slippage_suggester.is_some())
            .finish()
    }
}

/// Advanced settings for limit-order post flows.
#[derive(Clone, Default)]
#[non_exhaustive]
pub struct LimitOrderAdvancedSettings {
    /// Optional direct orderbook quote-request overrides.
    pub quote_request: Option<QuoteRequestOverride>,
    /// Optional app-data overrides merged into generated app-data documents.
    pub app_data: Option<AppDataParams>,
    /// Optional submission-time behavior overrides.
    pub additional_params: Option<PostTradeAdditionalParams>,
}

impl LimitOrderAdvancedSettings {
    /// Creates an empty limit-order-advanced-settings bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy with explicit quote-request overrides attached.
    #[must_use]
    pub fn with_quote_request(mut self, overrides: QuoteRequestOverride) -> Self {
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
}

impl fmt::Debug for LimitOrderAdvancedSettings {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LimitOrderAdvancedSettings")
            .field("quote_request", &self.quote_request)
            .field("app_data", &self.app_data)
            .field("additional_params", &self.additional_params)
            .finish()
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
    /// Optional explicit vault relayer address override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_relayer_address: Option<Address>,
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
            vault_relayer_address: None,
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

    /// Returns a copy with an explicit vault-relayer address override.
    #[must_use]
    pub fn with_vault_relayer_address(mut self, address: Address) -> Self {
        self.vault_relayer_address = Some(address);
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
    /// Optional explicit vault relayer address override.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vault_relayer_address: Option<Address>,
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
            vault_relayer_address: None,
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

    /// Returns a copy with an explicit vault-relayer address override.
    #[must_use]
    pub fn with_vault_relayer_address(mut self, address: Address) -> Self {
        self.vault_relayer_address = Some(address);
        self
    }
}

/// Options stored on [`crate::TradingSdk`] that do not belong in trader defaults.
#[derive(Clone, Default)]
pub struct TradingSdkOptions {
    order_book_api: Option<Arc<dyn OrderbookClient>>,
    quote_cache: Option<Arc<dyn crate::cache::QuoteCache>>,
}

impl fmt::Debug for TradingSdkOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradingSdkOptions")
            .field("order_book_api", &self.order_book_api.is_some())
            .field("quote_cache", &self.quote_cache.is_some())
            .finish()
    }
}

impl TradingSdkOptions {
    /// Creates an empty options bundle.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns a copy of these options with an injected orderbook client.
    ///
    /// The injected client fixes chain and environment for orderbook-bound flows.
    #[must_use]
    pub fn with_orderbook_client(mut self, orderbook_client: Arc<dyn OrderbookClient>) -> Self {
        self.order_book_api = Some(orderbook_client);
        self
    }

    /// Returns the injected orderbook client, if one is configured.
    #[must_use]
    pub fn orderbook_client(&self) -> Option<Arc<dyn OrderbookClient>> {
        self.order_book_api.clone()
    }

    /// Returns a copy of these options with an injected quote cache.
    ///
    /// The cache is instance-scoped; the trading SDK never registers a global
    /// cache on the caller's behalf. Passing `None` through
    /// [`TradingSdkBuilder::with_quote_cache`] keeps the pass-through
    /// [`crate::NoopQuoteCache`] default behaviour.
    #[must_use]
    pub fn with_quote_cache(mut self, quote_cache: Arc<dyn crate::cache::QuoteCache>) -> Self {
        self.quote_cache = Some(quote_cache);
        self
    }

    /// Returns the injected quote cache, if one is configured.
    #[must_use]
    pub fn quote_cache(&self) -> Option<Arc<dyn crate::cache::QuoteCache>> {
        self.quote_cache.clone()
    }
}

pub(crate) fn validate_orderbook_chain_context<O>(
    orderbook_client: &O,
    requested_chain: Option<SupportedChainId>,
) -> Result<(), TradingError>
where
    O: OrderbookClient + ?Sized,
{
    let context = orderbook_client.context();

    if let Some(chain_id) = requested_chain
        && chain_id != context.chain_id
    {
        return Err(TradingError::InjectedOrderbookContextConflict {
            field: "chainId",
            requested: u64::from(chain_id).to_string(),
            configured: u64::from(context.chain_id).to_string(),
        });
    }

    Ok(())
}

pub(crate) fn validate_orderbook_env_context<O>(
    orderbook_client: &O,
    requested_env: Option<CowEnv>,
) -> Result<(), TradingError>
where
    O: OrderbookClient + ?Sized,
{
    let context = orderbook_client.context();

    if let Some(env) = requested_env
        && env != context.env
    {
        return Err(TradingError::InjectedOrderbookContextConflict {
            field: "env",
            requested: env.as_str().to_owned(),
            configured: context.env.as_str().to_owned(),
        });
    }

    Ok(())
}

pub(crate) fn validate_orderbook_context<O>(
    orderbook_client: &O,
    requested_chain: Option<SupportedChainId>,
    requested_env: Option<CowEnv>,
) -> Result<(), TradingError>
where
    O: OrderbookClient + ?Sized,
{
    validate_orderbook_chain_context(orderbook_client, requested_chain)?;
    validate_orderbook_env_context(orderbook_client, requested_env)
}

pub(crate) fn validate_quote_orderbook_binding<O>(
    orderbook_client: &O,
    quoted_binding: Option<&OrderbookRuntimeBinding>,
) -> Result<(), TradingError>
where
    O: OrderbookClient + ?Sized,
{
    let Some(quoted_binding) = quoted_binding else {
        return Err(TradingError::MissingQuoteOrderbookBinding);
    };
    let submission_binding = orderbook_client.runtime_binding();

    if quoted_binding.chain_id != submission_binding.chain_id {
        return Err(TradingError::QuoteOrderbookBindingConflict {
            field: "chainId",
            quoted: u64::from(quoted_binding.chain_id).to_string(),
            submitted: u64::from(submission_binding.chain_id).to_string(),
        });
    }
    if quoted_binding.env != submission_binding.env {
        return Err(TradingError::QuoteOrderbookBindingConflict {
            field: "env",
            quoted: quoted_binding.env.as_str().to_owned(),
            submitted: submission_binding.env.as_str().to_owned(),
        });
    }
    if let (Some(quoted_base_url), Some(submission_base_url)) = (
        quoted_binding.resolved_base_url.as_ref(),
        submission_binding.resolved_base_url.as_ref(),
    ) && quoted_base_url != submission_base_url
    {
        return Err(TradingError::QuoteOrderbookBindingConflict {
            field: "baseUrl",
            quoted: quoted_base_url.clone(),
            submitted: submission_base_url.clone(),
        });
    }

    Ok(())
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
            PartnerFee::from_value(partner_fee_override.clone()).map_err(|error| {
                TradingError::InvalidInput(format!(
                    "appData.metadata.partnerFee must match the partner-fee schema: {error}"
                ))
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
    pub sell_token_balance: &'a mut OrderBalance,
    pub buy_token_balance: &'a mut OrderBalance,
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

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// Minimal orderbook capability required by the trading crate.
pub trait OrderbookClient: Send + Sync {
    /// Returns the effective orderbook API context.
    fn context(&self) -> &ApiContext;

    /// Returns the runtime binding used by this orderbook client.
    ///
    /// Implementations that apply additional endpoint overrides should override
    /// this method so quote-derived posting can validate the originating
    /// runtime authority precisely.
    fn runtime_binding(&self) -> OrderbookRuntimeBinding {
        OrderbookRuntimeBinding {
            chain_id: self.context().chain_id,
            env: self.context().env,
            resolved_base_url: self.context().resolved_base_url().ok(),
        }
    }

    /// Requests a quote from the orderbook.
    ///
    /// # Errors
    ///
    /// Returns the underlying orderbook error from the implementation.
    async fn get_quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError>;

    /// Submits an order to the orderbook.
    ///
    /// # Errors
    ///
    /// Returns the underlying orderbook error from the implementation.
    async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError>;

    /// Submits signed order cancellations to the orderbook.
    ///
    /// # Errors
    ///
    /// Returns the underlying orderbook error from the implementation.
    async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), OrderbookError>;

    /// Fetches an order by UID.
    ///
    /// # Errors
    ///
    /// Returns the underlying orderbook error from the implementation.
    async fn get_order(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError>;

    /// Uploads full app-data for a specific app-data hash.
    ///
    /// # Errors
    ///
    /// Returns the underlying orderbook error from the implementation.
    async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<AppDataObject, OrderbookError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// External slippage-suggestion provider used by advanced swap settings.
pub trait SlippageSuggestionProvider: Send + Sync {
    /// Returns an optional slippage suggestion for the supplied request.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when the provider cannot compute a suggestion.
    async fn get_slippage_suggestion(
        &self,
        request: SlippageToleranceRequest,
    ) -> Result<SlippageToleranceResponse, TradingError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// External existence checker used during `EthFlow` unique-order-id generation.
pub trait EthFlowOrderExistsChecker: Send + Sync {
    /// Returns `true` when the generated `EthFlow` order id already exists.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when the existence check fails.
    async fn order_exists(
        &self,
        order_id: &OrderUid,
        order_digest: &OrderDigest,
    ) -> Result<bool, TradingError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
/// Custom EIP-1271 signature provider used during order submission.
pub trait Eip1271SignatureProvider: Send + Sync {
    /// Produces an order signature payload for the provided unsigned order.
    ///
    /// # Errors
    ///
    /// Returns [`TradingError`] when signing fails.
    async fn sign(&self, order_to_sign: &UnsignedOrder) -> Result<String, TradingError>;
}

#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
impl OrderbookClient for OrderBookApi {
    fn context(&self) -> &ApiContext {
        self.context()
    }

    fn runtime_binding(&self) -> OrderbookRuntimeBinding {
        OrderbookRuntimeBinding {
            chain_id: self.context().chain_id,
            env: self.context().env,
            resolved_base_url: self.effective_base_url().ok(),
        }
    }

    async fn get_quote(
        &self,
        request: &OrderQuoteRequest,
    ) -> Result<OrderQuoteResponse, OrderbookError> {
        Self::get_quote(self, request).await
    }

    async fn send_order(&self, request: &OrderCreation) -> Result<OrderUid, OrderbookError> {
        Self::send_order(self, request).await
    }

    async fn send_signed_order_cancellations(
        &self,
        request: &OrderCancellations,
    ) -> Result<(), OrderbookError> {
        Self::send_signed_order_cancellations(self, request).await
    }

    async fn get_order(&self, order_uid: &OrderUid) -> Result<Order, OrderbookError> {
        Self::get_order(self, order_uid).await
    }

    async fn upload_app_data(
        &self,
        app_data_hash: &AppDataHash,
        full_app_data: &str,
    ) -> Result<AppDataObject, OrderbookError> {
        Self::upload_app_data(self, app_data_hash, full_app_data).await
    }
}
