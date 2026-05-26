#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]

//! High-level `CoW` Protocol trading workflows for quoting, signing, posting,
//! allowance management, and on-chain order actions.
//!
//! # Async-first entry points
//!
//! Every public free function and [`TradingSdk`] method in this crate is
//! `pub async fn` and accepts any signer implementing
//! [`cow_sdk_core::AsyncSigner`]. The crate ships one canonical async
//! entry per operation; callers in non-async contexts thread an executor
//! at the call site.
//!
//! Cooperative cancellation through
//! [`cow_sdk_core::Cancellable::cancel_with`] composes on every public
//! async entry. Each entry lifts a fired cancellation token into the
//! crate-level [`TradingError::Cancelled`] variant.
//!
//! Narrow async signer capability traits ([`cow_sdk_core::AsyncOwner`],
//! [`cow_sdk_core::AsyncTypedDataSigner`],
//! [`cow_sdk_core::AsyncDigestSigner`]) remain available for
//! callback-shaped adapters that expose only one signing operation.

#![warn(missing_docs)]

pub use cow_sdk_app_data::{PartnerFee, PartnerFeePolicy};
pub use cow_sdk_contracts::eth_flow;
pub use cow_sdk_core::{DecimalAmount, SupportedChainId};
pub use cow_sdk_orderbook::{OrderbookClient, OrderbookRuntimeBinding};

/// Allowance reads, approval transactions, and approval submission helpers.
pub mod allowance;
/// Trading app-data generation and quote-to-post merge helpers.
pub mod app_data;
/// Opt-in quote-cache seam with pass-through and TTL reference implementations.
pub mod cache;
/// Off-chain cancellation helpers.
pub mod cancel;
/// Trading crate error types.
pub mod error;
/// On-chain order actions and transaction builders.
pub mod onchain;
/// Order-construction helpers and `EthFlow` adjustments.
pub mod order;
/// Offline helper validation entry points on trade-parameter builders.
pub mod parameters;
/// Quote-to-post orchestration helpers.
pub mod post;
/// Quote construction and quote-request precedence helpers.
pub mod quote;
/// High-level `TradingSdk` facade and builder.
pub mod sdk;
/// Slippage and fee calculation helpers.
pub mod slippage;
/// Shared trading DTOs, trait seams, and settings types.
pub mod types;
/// Typed client-side validator enforcing the reviewed services
/// protocol-invariant matrix on every submission seam.
pub mod validation;
/// Broadcast-then-poll helpers for mined transaction receipts.
pub mod wait;

pub use allowance::{approval_transaction, approve_cow_protocol, get_cow_protocol_allowance};
pub use app_data::{build_app_data, merge_and_seal_app_data, params_from_doc};
pub use cache::{
    Clock, DEFAULT_QUOTE_CACHE_CAPACITY, DEFAULT_QUOTE_CACHE_TTL, InMemoryQuoteCache,
    NoopQuoteCache, QuoteCache, QuoteCacheKey, SystemClock,
};
pub use cancel::off_chain_cancel_order;
pub use error::{OrderbookContextValue, TradingError};
pub use onchain::{
    EthFlowTransaction, cancel_order_onchain, get_eth_flow_transaction, get_pre_sign_transaction,
    onchain_cancellation_transaction, protocol_options_for_order,
};
pub use order::{
    OrderToSignParams, adjust_ethflow_limit_parameters, adjust_ethflow_trade_parameters,
    calculate_unique_order_id, get_order_to_sign, is_ethflow_order,
    swap_params_to_limit_order_params,
};
pub use post::{
    post_cow_protocol_trade, post_limit_order, post_sell_native_currency_order, post_swap_order,
    post_swap_order_from_quote,
};
pub use quote::{get_quote_only, get_quote_results};
pub use sdk::{
    AppCodeSet, AppCodeUnset, ChainIdSet, ChainIdUnset, HelperOnlySdk, TradingSdk,
    TradingSdkBuilder,
};
pub use slippage::{
    DEFAULT_QUOTE_VALIDITY, DEFAULT_SLIPPAGE_BPS, GAS_LIMIT_DEFAULT, GAS_MARGIN_PERCENT,
    MAX_SLIPPAGE_BPS, calculate_quote_amounts_and_costs, default_slippage_bps, partner_fee_bps,
    resolve_slippage_suggestion, sanitize_protocol_fee_bps, suggest_slippage_bps,
    suggest_slippage_from_fee, suggest_slippage_from_volume,
};
pub use types::{
    AllowanceParameters, AppCode, AppCodeError, ApprovalParameters, EthFlowOrderExistsChecker,
    LimitOrderAdvancedSettings, LimitTradeParameters, LimitTradeParametersFromQuote,
    OrderPostingResult, OrderTraderParameters, PartialTraderParameters, PostTradeAdditionalParams,
    QuoteRequestOverride, QuoteResults, QuoterParameters, SlippageSuggestionProvider,
    SlippageToleranceRequest, SlippageToleranceResponse, SwapAdvancedSettings, TradeParameters,
    TraderParameters, TradingAppDataInfo, TradingSdkOptions, TradingTransactionParams,
};
pub use validation::{
    AmountSide, ClientRejection, OrderBoundsValidator, OrderValidityBounds, SubmissionClass,
};
pub use wait::{WaitError, WaitOptions, poll_for_receipt, submit_and_wait_for_receipt};
