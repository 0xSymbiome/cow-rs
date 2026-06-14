#![cfg_attr(any(doctest, docsrs), doc = include_str!("../README.md"))]

//! High-level `CoW` Protocol trading workflows for quoting, signing, posting,
//! allowance management, and on-chain order actions.
//!
//! # Async-first entry points
//!
//! Every public free function and [`Trading`] method in this crate is
//! `pub async fn` and accepts any signer implementing
//! [`cow_sdk_core::Signer`]. The crate ships one canonical async
//! entry per operation; callers in non-async contexts thread an executor
//! at the call site.
//!
//! Cooperative cancellation through
//! [`cow_sdk_core::Cancellable::cancel_with`] composes on every public
//! async entry. Each entry lifts a fired cancellation token into the
//! crate-level [`TradingError::Cancelled`] variant.
//!
//! Narrow async signer capability traits ([`cow_sdk_core::Owner`],
//! [`cow_sdk_core::TypedDataSigner`],
//! [`cow_sdk_core::DigestSigner`]) remain available for
//! callback-shaped adapters that expose only one signing operation.
//!
//! # Fluent swap lifecycle
//!
//! [`Trading::swap`] opens a typed [`SwapBuilder`] for the common swap path:
//! named sell/buy/amount setters that cannot be transposed, then a single
//! asynchronous terminal — [`SwapBuilder::execute`] for one-call quote-sign-post,
//! or [`SwapBuilder::quote`] to inspect a [`QuotedSwap`] before
//! [`QuotedSwap::submit`]. The flat free functions and [`Trading`] methods remain
//! the full surface; the builder is an additive ergonomic entry over them.

#![warn(missing_docs)]

/// Re-exported so seam-trait implementors need no direct `async-trait`
/// dependency, mirroring serde's derive re-export.
pub use async_trait::async_trait;
pub use cow_sdk_app_data::{PartnerFee, PartnerFeePolicy};
pub use cow_sdk_contracts::eth_flow;
pub use cow_sdk_core::SupportedChainId;
pub use cow_sdk_orderbook::{OrderbookClient, OrderbookRuntimeBinding};

/// Allowance reads, approval transactions, and approval submission helpers.
pub mod allowance;
/// Trading app-data generation and quote-to-post merge helpers.
pub mod app_data;
/// Off-chain cancellation helpers.
pub mod cancel;
/// Trading crate error types.
pub mod error;
/// On-chain order actions and transaction builders.
pub mod onchain;
/// Order-construction helpers and `EthFlow` adjustments.
pub mod order;
/// Offline helper validation entry points on trade-parameter builders.
pub mod params;
/// Quote-to-post orchestration helpers.
pub mod post;
/// Quote construction and quote-request precedence helpers.
pub mod quote;
// The stateful, high-level `Trading` client; its public types are re-exported below.
mod client;
/// Slippage and fee calculation helpers.
pub mod slippage;
/// Shared trading DTOs, trait seams, and settings types.
pub mod types;
/// Typed client-side validator enforcing the reviewed services
/// protocol-invariant matrix on every submission seam.
pub mod validation;
/// Broadcast-then-poll helpers for mined transaction receipts.
pub mod wait;

pub use allowance::{approval_transaction, approve_cow_protocol, cow_protocol_allowance};
pub use app_data::{build_app_data, merge_and_seal_app_data, params_from_doc};
pub use cancel::offchain_cancel_order;
pub use client::{
    AppCodeSet, AppCodeUnset, ChainIdSet, ChainIdUnset, LimitBuilder, QuotedSwap, Set, SwapBuilder,
    Trading, TradingBuilder, Unset,
};
pub use error::{OrderbookContextValue, TradingError};
pub use onchain::{
    EthFlowTransaction, PreparedTransaction, eth_flow_transaction, onchain_cancel_order,
    onchain_cancellation_transaction, pre_sign_transaction,
};
pub use order::{
    OrderToSignParams, calculate_unique_order_id, order_to_sign, swap_params_to_limit_order_params,
};
pub(crate) use order::{
    adjust_eth_flow_limit_params, adjust_eth_flow_trade_params, is_eth_flow_order,
};
pub use post::{
    post_cow_protocol_trade, post_limit_order, post_limit_order_presign,
    post_sell_native_currency_order, post_swap_order, post_swap_order_from_quote,
};
pub use quote::{quote_only, quote_results};
pub use slippage::{
    DEFAULT_GAS_LIMIT, DEFAULT_QUOTE_VALIDITY, DEFAULT_SLIPPAGE_BPS, GAS_MARGIN_PERCENT,
    MAX_SLIPPAGE_BPS, calculate_quote_amounts_and_costs, default_slippage_bps, partner_fee_bps,
    resolve_slippage_suggestion, sanitize_protocol_fee_bps, suggest_slippage_bps,
    suggest_slippage_from_fee, suggest_slippage_from_volume,
};
pub use types::{
    AllowanceParams, ApprovalParams, EthFlowOrderExistsChecker, LimitTradeParams,
    LimitTradeParamsFromQuote, OrderPostingResult, OrderTraderParams, PostTradeAdditionalParams,
    QuoteRequestOverride, QuoteResults, QuoterParams, SlippageSuggester, SlippageToleranceRequest,
    SlippageToleranceResponse, TradeAdvancedSettings, TradeParams, TraderParams,
    TradingAppDataInfo,
};
// Crate-internal: the partial trader defaults are stored state, not a public
// construction shape (see ADR 0011). Internal modules reach it through the crate root.
pub(crate) use types::PartialTraderParams;
pub use validation::{AmountSide, ClientRejection, OrderBoundsValidator};
pub use wait::{WaitError, WaitOptions, poll_for_receipt, submit_and_wait_for_receipt};
