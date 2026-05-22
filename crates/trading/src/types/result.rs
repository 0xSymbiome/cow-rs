use serde::{Deserialize, Serialize};

use cow_sdk_app_data::AppDataDoc;
use cow_sdk_core::{AppDataHash, OrderUid, QuoteAmountsAndCosts, TransactionHash, UnsignedOrder};
use cow_sdk_orderbook::{OrderQuoteResponse, OrderbookRuntimeBinding, SigningScheme};
use cow_sdk_signing::OrderTypedData;

use super::trade::TradeParameters;

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
