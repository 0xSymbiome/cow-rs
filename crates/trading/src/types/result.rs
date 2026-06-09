#![allow(
    clippy::redundant_pub_crate,
    reason = "the orderbook-context validators intentionally stay pub(crate) and are re-exported through types::mod for unchanged crate-local call sites"
)]

//! Quote and post result types returned by the trading helpers, plus the
//! crate-internal orderbook chain/env/binding context validators.

use serde::{Deserialize, Serialize};

use cow_sdk_app_data::AppDataDoc;
use cow_sdk_core::{
    Address, Amount, AppDataHash, CowEnv, OrderData, OrderUid, QuoteAmountsAndCosts,
    SupportedChainId, TransactionHash,
};
use cow_sdk_orderbook::{OrderQuoteResponse, OrderbookClient, OrderbookRuntimeBinding, SigningScheme};
use cow_sdk_signing::OrderTypedData;

use super::params::TradeParameters;
use crate::{OrderbookContextValue, TradingError};

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
    pub order_to_sign: OrderData,
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
    pub const fn new(
        trade_parameters: TradeParameters,
        suggested_slippage_bps: u32,
        amounts_and_costs: QuoteAmountsAndCosts,
        order_to_sign: OrderData,
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
    pub order_to_sign: OrderData,
}

impl OrderPostingResult {
    /// Creates a posting result with the required identity and payload fields.
    #[must_use]
    pub fn new(
        order_id: OrderUid,
        signing_scheme: SigningScheme,
        signature: impl Into<String>,
        order_to_sign: OrderData,
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
    pub const fn with_tx_hash(mut self, tx_hash: TransactionHash) -> Self {
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

/// Slippage-suggestion request sent to a custom suggestion provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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
            requested: OrderbookContextValue::ChainId(u64::from(chain_id)),
            configured: OrderbookContextValue::ChainId(u64::from(context.chain_id)),
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
            requested: OrderbookContextValue::Env(env),
            configured: OrderbookContextValue::Env(context.env),
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
            quoted: OrderbookContextValue::ChainId(u64::from(quoted_binding.chain_id)),
            submitted: OrderbookContextValue::ChainId(u64::from(submission_binding.chain_id)),
        });
    }
    if quoted_binding.env != submission_binding.env {
        return Err(TradingError::QuoteOrderbookBindingConflict {
            field: "env",
            quoted: OrderbookContextValue::Env(quoted_binding.env),
            submitted: OrderbookContextValue::Env(submission_binding.env),
        });
    }
    if let (Some(quoted_base_url), Some(submission_base_url)) = (
        quoted_binding.resolved_base_url.as_ref(),
        submission_binding.resolved_base_url.as_ref(),
    ) && quoted_base_url != submission_base_url
    {
        return Err(TradingError::QuoteOrderbookBindingConflict {
            field: "baseUrl",
            quoted: OrderbookContextValue::BaseUrl(quoted_base_url.clone().into()),
            submitted: OrderbookContextValue::BaseUrl(submission_base_url.clone().into()),
        });
    }

    Ok(())
}
