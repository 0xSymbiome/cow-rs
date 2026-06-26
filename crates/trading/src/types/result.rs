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
    SupportedChainId, TransactionHash, TypedDataEnvelope,
};
use cow_sdk_orderbook::{
    OrderQuoteResponse, OrderbookClient, OrderbookBinding, SigningScheme,
};

use super::params::TradeParams;
use crate::{OrderbookContextValue, TradingError};

/// Fully resolved quote result produced by trading quote helpers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct QuoteResults {
    /// Effective trade parameters after SDK defaults and advanced settings were applied.
    pub trade_parameters: TradeParams,
    /// Suggested slippage in basis points after SDK or custom-provider resolution.
    pub suggested_slippage_bps: u32,
    /// Fee and amount breakdown derived from the orderbook quote.
    ///
    /// Spelled with the explicit `<Amount>` type argument on the TypeScript
    /// boundary: the native field uses the `T = Amount` default of
    /// [`QuoteAmountsAndCosts`], but TypeScript generics carry no default, so a
    /// bare reference to the emitted `QuoteAmountsAndCosts<T>` would not
    /// type-check.
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
        tsify(type = "QuoteAmountsAndCosts<Amount>")
    )]
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
    /// orderbook runtime. It is omitted from serialization when `None` and
    /// defaults back to `None` when absent, so a `QuoteResults` whose binding was
    /// not carried through — rehydrated from storage, or rebuilt without it —
    /// fails closed on resubmission with `TradingError::MissingQuoteOrderbookBinding`
    /// rather than posting against an unverified runtime. A faithful round-trip
    /// preserves a `Some` binding; the gate enforces runtime-authority match, not
    /// quote freshness (the quote's own expiry governs that).
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub orderbook_binding: Option<OrderbookBinding>,
    /// Typed order-facing envelope kept for consumers while signers use the
    /// lower-level `TypedDataPayload` seam internally.
    ///
    /// Spelled as the concrete `TypedDataEnvelope<OrderData>` rather than the
    /// `OrderTypedData` alias so the generated TypeScript boundary references
    /// the emitted `TypedDataEnvelope<OrderData>` declaration; the alias is a
    /// transparent synonym, so native construction and reads are unchanged.
    pub order_typed_data: TypedDataEnvelope<OrderData>,
}

/// Result returned after submitting a trade or transaction-producing flow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct OrderPostingResult {
    /// Final order UID.
    pub order_id: OrderUid,
    /// Settlement transaction hash when the flow submits an on-chain
    /// transaction directly (32-byte `0x`-prefixed hex string).
    ///
    /// Spelled as a viem-compatible `0x`-prefixed hex string on the TypeScript
    /// boundary: the native `TransactionHash` alias of `Hash32` is not emitted as
    /// a declaration, so the override pins the protocol-canonical `0x`-prefixed
    /// hex wire form (the same idiom the orderbook `Trade.tx_hash` field uses).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
        tsify(type = "`0x${string}`")
    )]
    pub tx_hash: Option<TransactionHash>,
    /// Signature scheme used for the posted order.
    pub signing_scheme: SigningScheme,
    /// Signature payload sent to the orderbook, or empty string for transaction-only flows.
    pub signature: String,
    /// Unsigned order payload used for signing or transaction generation.
    pub order_to_sign: OrderData,
}

/// App-data bundle used by trading quote and post helpers.
#[allow(
    clippy::derive_partial_eq_without_eq,
    reason = "the `doc: AppDataDoc` field is a `serde_json::Value` alias, and `serde_json::Value` does not implement `Eq`"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    derive(tsify::Tsify)
)]
#[cfg_attr(
    all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
    tsify(into_wasm_abi, from_wasm_abi)
)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct TradingAppDataInfo {
    /// Parsed app-data document.
    ///
    /// Spelled as the `Value` escape hatch on the TypeScript boundary because
    /// the app-data document is arbitrary JSON; the native field is the
    /// [`AppDataDoc`] alias of `serde_json::Value`.
    #[cfg_attr(
        all(target_arch = "wasm32", target_os = "unknown", feature = "ts-bindings"),
        tsify(type = "Value")
    )]
    pub doc: AppDataDoc,
    /// Canonically serialized app-data payload.
    pub full_app_data: String,
    /// Keccak-256 digest used in protocol order payloads.
    pub app_data_keccak256: AppDataHash,
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
    quoted_binding: Option<&OrderbookBinding>,
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
