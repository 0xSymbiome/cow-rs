//! High-level CoW Protocol trading workflows for quoting, signing, posting,
//! allowance management, and on-chain order actions.

/// Allowance reads, approval transactions, and approval submission helpers.
pub mod allowance;
/// Off-chain cancellation helpers.
pub mod cancel;
/// Trading crate error types.
pub mod error;
/// On-chain order actions and transaction builders.
pub mod onchain;
/// Order-construction helpers and EthFlow adjustments.
pub mod order;
/// Quote-to-post orchestration helpers.
pub mod post;
/// Quote construction, app-data generation, and quote-request precedence helpers.
pub mod quote;
/// High-level `TradingSdk` facade and builder.
pub mod sdk;
/// Slippage and fee calculation helpers.
pub mod slippage;
/// Shared trading DTOs, trait seams, and settings types.
pub mod types;

pub use allowance::{
    approval_transaction, approve_cow_protocol, approve_cow_protocol_async,
    get_cow_protocol_allowance, get_cow_protocol_allowance_async,
};
pub use cancel::{off_chain_cancel_order, off_chain_cancel_order_async};
pub use error::TradingError;
pub use onchain::{
    EthFlowTransaction, cancel_order_onchain, cancel_order_onchain_async, get_eth_flow_transaction,
    get_eth_flow_transaction_async, get_pre_sign_transaction, get_pre_sign_transaction_async,
    onchain_cancellation_transaction, onchain_cancellation_transaction_async,
    protocol_options_for_order,
};
pub use order::{
    OrderToSignParams, adjust_ethflow_limit_parameters, adjust_ethflow_trade_parameters,
    calculate_unique_order_id, get_order_to_sign, is_ethflow_order,
    swap_params_to_limit_order_params,
};
pub use post::{
    post_cow_protocol_trade, post_cow_protocol_trade_async, post_limit_order,
    post_limit_order_async, post_sell_native_currency_order, post_sell_native_currency_order_async,
    post_swap_order, post_swap_order_async, post_swap_order_from_quote,
    post_swap_order_from_quote_async,
};
pub use quote::{
    build_app_data, get_quote_only, get_quote_results, get_quote_results_async, merge_app_data_doc,
};
pub use sdk::{TradingSdk, TradingSdkBuilder};
pub use slippage::{
    DEFAULT_QUOTE_VALIDITY, DEFAULT_SLIPPAGE_BPS, GAS_LIMIT_DEFAULT, GAS_MARGIN_PERCENT,
    MAX_SLIPPAGE_BPS, calculate_quote_amounts_and_costs, default_slippage_bps, partner_fee_bps,
    resolve_slippage_suggestion, sanitize_protocol_fee_bps, suggest_slippage_bps,
    suggest_slippage_from_fee, suggest_slippage_from_volume,
};
pub use types::{
    AllowanceParameters, ApprovalParameters, Eip1271SignatureProvider, EthFlowOrderExistsChecker,
    LimitOrderAdvancedSettings, LimitTradeParameters, LimitTradeParametersFromQuote,
    OrderPostingResult, OrderTraderParameters, OrderbookClient, PartialTraderParameters,
    PostTradeAdditionalParams, QuoteRequestOverride, QuoteResults, QuoterParameters,
    SlippageSuggestionProvider, SlippageToleranceRequest, SlippageToleranceResponse,
    SwapAdvancedSettings, TradeParameters, TraderParameters, TradingAppDataInfo, TradingSdkOptions,
    TradingTransactionParams,
};
