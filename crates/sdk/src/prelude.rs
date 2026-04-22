//! Curated re-exports for consumers that want the primary `cow-sdk` surface from one import.
//!
//! The facade remains a re-export layer. Package-specific implementation details stay in the
//! leaf crates that own them.

pub use crate::{ErrorClass, SdkError};
pub use cow_sdk_app_data::{
    AppDataDoc, AppDataError, AppDataInfo, AppDataParams, IpfsConfig, IpfsFetchTransport,
    IpfsUploadResult, IpfsUploadTransport, PartnerFee, PartnerFeePolicy, SchemaVersion,
    ValidationResult, app_data_hex_to_cid, cid_to_app_data_hex, fetch_doc_from_app_data_hex,
    fetch_doc_from_cid, generate_app_data_doc, get_app_data_info, get_app_data_schema,
    validate_app_data_doc,
};
#[cfg(feature = "browser-wallet")]
#[cfg_attr(docsrs, doc(cfg(feature = "browser-wallet")))]
pub use cow_sdk_browser_wallet::{
    BrowserWallet, BrowserWalletError, Eip1193Provider, Eip1193Signer, InjectedWalletInfo,
    MockEip1193Transport, RpcErrorPayload, WalletEvent, WalletSession,
};
pub use cow_sdk_contracts::{
    BUY_ETH_ADDRESS, ContractId, ContractsError, ORDER_TYPE_HASH, ORDER_UID_LENGTH, Registry,
    compute_order_uid, deployment_for_chain, hash_order_for_contract, uid_for_contract,
};
pub use cow_sdk_core::{
    Address, Amount, ApiBaseUrls, AppDataHash, AppDataHex, AsyncProvider, AsyncSigner, BlockHash,
    BuyTokenDestination, Cancellable, Cancelled, CoreError, CowEnv, Hash32, HexData, HttpTransport,
    OrderDigest, OrderKind, OrderModel, OrderUid, ProtocolOptions, QuoteModel, SellTokenSource,
    SignedAmount, SupportedChainId, TradeModel, TransactionHash, TransportError, UnsignedOrder,
    ValidationError, ValidationReason,
};
#[cfg(not(target_arch = "wasm32"))]
pub use cow_sdk_core::{ReqwestTransport, ReqwestTransportConfig};
pub use cow_sdk_orderbook::{
    ApiContext, AppDataObject, CompetitionOrderStatus, GetOrdersRequest, GetTradesRequest,
    NativePriceResponse, OrderBookApi, OrderBookApiBuilder, OrderBookApiError, OrderCancellations,
    OrderCreation, OrderQuoteRequest, OrderQuoteResponse, OrderbookClient, OrderbookError,
    OrderbookRejection, PriceQuality, QuoteData, QuoteSide, SolverCompetitionResponse,
    TotalSurplus, parse_rejection,
};
pub use cow_sdk_signing::{
    Eip1271VerificationCache, GeneratedOrderId, ORDER_PRIMARY_TYPE, OrderTypedData, SigningError,
    SigningResult, SigningScheme, domain_separator, eip1271_signature_payload, generate_order_id,
    get_domain, order_typed_data, sign_order, sign_order_async, sign_order_cancellation,
    sign_order_cancellation_async, sign_order_cancellations, sign_order_cancellations_async,
    sign_order_cancellations_with_scheme_async, sign_order_with_scheme,
    sign_order_with_scheme_async,
};
pub use cow_sdk_trading::{
    AllowanceParameters, AmountSide, AppCodeSet, AppCodeUnset, ApprovalParameters, ChainIdSet,
    ChainIdUnset, ClientRejection, DEFAULT_QUOTE_VALIDITY, DEFAULT_SLIPPAGE_BPS, GAS_LIMIT_DEFAULT,
    GAS_MARGIN_PERCENT, LimitOrderAdvancedSettings, LimitTradeParameters,
    LimitTradeParametersFromQuote, MAX_SLIPPAGE_BPS, OrderBoundsValidator, OrderPostingResult,
    OrderTraderParameters, OrderValidityBounds, OrderbookRuntimeBinding, PartialTraderParameters,
    PostTradeAdditionalParams, QuoteRequestOverride, QuoteResults, QuoterParameters,
    SubmissionClass, SwapAdvancedSettings, TradeParameters, TraderParameters, TradingAppDataInfo,
    TradingError, TradingSdk, TradingSdkBuilder, TradingSdkMode, TradingSdkOptions,
    TradingTransactionParams, approval_transaction, approve_cow_protocol,
    approve_cow_protocol_async, build_app_data, calculate_quote_amounts_and_costs,
    calculate_unique_order_id, cancel_order_onchain_async, default_slippage_bps,
    get_cow_protocol_allowance, get_cow_protocol_allowance_async, get_eth_flow_transaction_async,
    get_order_to_sign, get_pre_sign_transaction_async, get_quote_only, get_quote_results,
    get_quote_results_async, is_ethflow_order, merge_and_seal_app_data, off_chain_cancel_order,
    off_chain_cancel_order_async, params_from_doc, partner_fee_bps, post_cow_protocol_trade,
    post_cow_protocol_trade_async, post_limit_order, post_limit_order_async,
    post_sell_native_currency_order, post_sell_native_currency_order_async, post_swap_order,
    post_swap_order_async, post_swap_order_from_quote, post_swap_order_from_quote_async,
    resolve_slippage_suggestion, sanitize_protocol_fee_bps, suggest_slippage_bps,
    suggest_slippage_from_fee, suggest_slippage_from_volume, swap_params_to_limit_order_params,
};
