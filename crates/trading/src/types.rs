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

fn default_order_balance() -> OrderBalance {
    OrderBalance::Erc20
}

/// Fully resolved trader configuration used by order-posting and on-chain flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Partial trader defaults stored on [`crate::TradingSdk`] and its builder.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Quoter configuration used by quote-only and quote-and-sign flows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Swap-style trade request accepted by quote and post helpers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Limit-order request accepted by posting and signing helpers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Compatibility alias for limit-order params derived from a quote.
pub type LimitTradeParametersFromQuote = LimitTradeParameters;
/// Compatibility alias for the transaction type returned by trading helpers.
pub type TradingTransactionParams = TransactionRequest;

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

/// Slippage-suggestion response returned by a custom suggestion provider.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SlippageToleranceResponse {
    /// Suggested slippage tolerance in basis points.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slippage_bps: Option<u32>,
}

/// Fully resolved quote result produced by trading quote helpers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Runtime binding captured from an orderbook client for quote-derived workflows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OrderbookRuntimeBinding {
    /// Chain id fixed by the orderbook client.
    pub chain_id: SupportedChainId,
    /// Environment fixed by the orderbook client.
    pub env: CowEnv,
    /// Resolved base URL used by the orderbook client when it is available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_base_url: Option<String>,
}

/// Result returned after submitting a trade or transaction-producing flow.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// App-data bundle used by trading quote and post helpers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TradingAppDataInfo {
    /// Parsed app-data document.
    pub doc: AppDataDoc,
    /// Canonically serialized app-data payload.
    pub full_app_data: String,
    /// Keccak-256 digest used in protocol order payloads.
    pub app_data_keccak256: AppDataHash,
}

/// Optional overrides applied directly to the orderbook quote request.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Optional knobs applied after quoting and before final submission.
#[derive(Clone, Default)]
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
pub struct LimitOrderAdvancedSettings {
    /// Optional direct orderbook quote-request overrides.
    pub quote_request: Option<QuoteRequestOverride>,
    /// Optional app-data overrides merged into generated app-data documents.
    pub app_data: Option<AppDataParams>,
    /// Optional submission-time behavior overrides.
    pub additional_params: Option<PostTradeAdditionalParams>,
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

/// Parameters for allowance-check helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Parameters for approval-transaction helpers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

/// Options stored on [`crate::TradingSdk`] that do not belong in trader defaults.
#[derive(Clone, Default)]
pub struct TradingSdkOptions {
    order_book_api: Option<Arc<dyn OrderbookClient>>,
}

impl fmt::Debug for TradingSdkOptions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TradingSdkOptions")
            .field("order_book_api", &self.order_book_api.is_some())
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
