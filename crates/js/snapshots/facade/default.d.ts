/// <reference lib="esnext.disposable" />
import * as raw from "./raw/default.js";
import type { WasmEnvelope } from "./envelope.js";
import type { CommonClientConfig, IpfsClientConfig, SdkClientOptions, SigningOptions, SubgraphClientConfig, TradingClientConfig } from "./options.js";
import type { ContractReadCallback, CustomEip1271Callback, DigestSignerCallback, TypedDataSignerCallback } from "./callbacks.js";
export type OrderBookClientConfig = CommonClientConfig;
/**
 * Initialize the wasm module, idempotently, once per module instance.
 *
 * On the `web` build — Cloudflare Workers, Deno, Vercel Edge, and no-bundler
 * browsers — the host owns module instantiation, so it must call this once with
 * the compiled module (Workers pass the `CompiledWasm` binding) or its URL/bytes
 * (Deno and browsers). On the `bundler` and `nodejs` builds the host
 * bundler/runtime instantiates the module on import, so this resolves
 * immediately and the argument is ignored — calling it is optional and harmless,
 * which keeps one call shape working across every target.
 */
export declare function initialize(module?: WebAssembly.Module | raw.InitInput): Promise<void>;
export default initialize;
export type { InitInput } from "./raw/default.js";
export declare class IpfsClient {
    #private;
    constructor(config: IpfsClientConfig);
    fetchAppDataFromCid(cid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.AppDataDocument>>;
    fetchAppDataFromHex(appDataHex: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.AppDataDocument>>;
    dispose(): void;
    [Symbol.dispose](): void;
}
export declare class OrderBookClient {
    #private;
    constructor(config: OrderBookClientConfig);
    cancelOrders(signed: raw.SignedCancellations, options?: SdkClientOptions | null): Promise<WasmEnvelope<{
        cancelled: true;
    }>>;
    getNativePrice(token: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.NativePriceResponse>>;
    getOrder(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.Order>>;
    getOrders(owner: string, pagination?: raw.PaginationOptions | null, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.Order[]>>;
    getOrderMultiEnv(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.Order>>;
    getTxOrders(txHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.Order[]>>;
    getVersion(options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    getOrderLink(orderUid: string): WasmEnvelope<string>;
    getQuote(request: raw.OrderQuoteRequest, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.OrderQuoteResponse>>;
    getTrades(query: raw.GetTradesRequest, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.Trade[]>>;
    getOrderCompetitionStatus(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.CompetitionOrderStatus>>;
    getTotalSurplus(owner: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.TotalSurplus>>;
    getSolverCompetition(auctionId: number, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.SolverCompetitionResponse>>;
    getSolverCompetitionByTxHash(txHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.SolverCompetitionResponse>>;
    getAppData(appDataHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.AppDataObject>>;
    uploadAppData(appDataHash: string, fullAppData: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<{
        uploaded: true;
    }>>;
    sendOrder(signed: raw.SignedOrder, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    sendOrderCreation(input: raw.OrderCreation, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    dispose(): void;
    [Symbol.dispose](): void;
}
export declare class SubgraphClient {
    #private;
    constructor(config: SubgraphClientConfig);
    getLastDaysVolume(days: number, options?: SdkClientOptions | null): Promise<WasmEnvelope<unknown>>;
    getLastHoursVolume(hours: number, options?: SdkClientOptions | null): Promise<WasmEnvelope<unknown>>;
    getTotals(options?: SdkClientOptions | null): Promise<WasmEnvelope<unknown>>;
    runQuery(query: string, variables?: Record<string, unknown>, operationName?: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<unknown>>;
    dispose(): void;
    [Symbol.dispose](): void;
}
export declare class TradingClient {
    #private;
    constructor(config: TradingClientConfig);
    buildSellNativeCurrencyTx(order: raw.OrderData, quoteId: number, from: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.BuiltSellNativeCurrencyTx>>;
    buildSellNativeCurrencyTxFromQuote(quoteResults: raw.QuoteResults, from: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.BuiltSellNativeCurrencyTx>>;
    getCowProtocolAllowance(params: raw.AllowanceParams, readContractCallback: ContractReadCallback, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    buildApprovalTx(params: raw.ApprovalParams, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.TransactionRequest>>;
    buildWrapTx(amount: string): WasmEnvelope<raw.TransactionRequest>;
    buildUnwrapTx(amount: string): WasmEnvelope<raw.TransactionRequest>;
    getQuote(params: raw.TradeParams, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.QuoteResults>>;
    postLimitOrder(params: raw.LimitTradeParams, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.OrderPostingResult>>;
    postSwapOrder(params: raw.TradeParams, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.OrderPostingResult>>;
    postSwapOrderFromQuote(quoteResults: raw.QuoteResults, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.OrderPostingResult>>;
    postSwapOrderWithEip1271(params: raw.TradeParams, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.OrderPostingResult>>;
    dispose(): void;
    [Symbol.dispose](): void;
}
export declare function appDataDoc(doc: raw.AppDataParams): WasmEnvelope<raw.AppDataDocument>;
export declare function appDataHexToCid(appDataHex: string): WasmEnvelope<string>;
export declare function appDataInfo(doc: raw.AppDataParams): WasmEnvelope<raw.AppDataInfo>;
/**
 * The app-data order class. Distinct from the order-book order class — it carries
 * the additional `twap` value used by composable orders.
 */
export type AppDataOrderClass = "market" | "limit" | "liquidity" | "twap";
/**
 * Builds app-data with the SDK's standard metadata — quote slippage, the given
 * order class, and (unless overridden) the default SDK UTM attribution — and
 * returns its hash and content. The high-level counterpart to {@link appDataInfo},
 * matching what the swap and limit flows attach automatically; pass
 * `orderClass: "twap"` for a composable TWAP order.
 */
export declare function buildAppData(params: {
    appCode: string;
    slippageBps: number;
    orderClass: AppDataOrderClass;
}): WasmEnvelope<raw.AppDataInfo>;
export declare function buildCancelOrderTx(params: raw.OrderTraderParams): WasmEnvelope<raw.TransactionRequest>;
export declare function buildPresignTx(params: raw.OrderTraderParams): WasmEnvelope<raw.TransactionRequest>;
export declare function buildTwapCreateTransaction(params: raw.TwapCreateParams): WasmEnvelope<raw.TwapCreateResult>;
export declare function buildTwapRemoveTransaction(orderId: string): WasmEnvelope<raw.TransactionRequest>;
export declare function cidToAppDataHex(cid: string): WasmEnvelope<string>;
export declare function computeOrderUid(order: raw.OrderData, chainId: number, owner: string): WasmEnvelope<raw.GeneratedOrderUid>;
export declare function decodeEthFlowLog(log: raw.EventLog): WasmEnvelope<raw.EthFlowEvent>;
export declare function decodeSettlementLog(log: raw.EventLog): WasmEnvelope<raw.SettlementEvent>;
export declare function deploymentAddresses(chainId: number, env?: string | null): WasmEnvelope<raw.DeploymentAddresses>;
export declare function domainSeparator(chainId: number): WasmEnvelope<string>;
export declare function eip1271SignaturePayload(order: raw.OrderData, ecdsaSignature: string): WasmEnvelope<string>;
export type TypedDataMessage = unknown;
export declare function orderTypedData(order: raw.OrderData, chainId: number): WasmEnvelope<raw.TypedDataEnvelope<TypedDataMessage>>;
export declare function signCancellationEthSignDigest(orderUids: string[], chainId: number, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedCancellations>>;
export declare function signCancellationWithTypedDataSigner(orderUids: string[], chainId: number, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedCancellations>>;
export declare function signOrderEthSignDigest(order: raw.OrderData, chainId: number, owner: string, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrder>>;
export declare function signOrderWithCustomEip1271(order: raw.OrderData, chainId: number, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrder>>;
export declare function signOrderWithEip1271(order: raw.OrderData, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrder>>;
export declare function signOrderWithTypedDataSigner(order: raw.OrderData, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrder>>;
export declare function supportedChainIds(): Uint32Array;
export declare function validateAppDataDoc(doc: raw.AppDataParams): WasmEnvelope<raw.ValidationResult>;
export declare function wasmVersion(): string;
export declare function wrappedNativeToken(chainId: number): WasmEnvelope<raw.WrappedNativeToken>;
export type { AllowanceParams, Amounts, AppDataDocument, AppDataParams, AppDataInfo, AppDataObject, ApprovalParams, BuiltSellNativeCurrencyTx, BuyTokenDestination, CompetitionAuction, CompetitionOrderStatus, CompetitionOrderStatusKind, ContractCall, Costs, CowEip1271SignRequest, DeploymentAddresses, EthflowData, EthFlowEvent, EventLog, ExecutedAmounts, ExecutedProtocolFee, FeeComponent, GeneratedOrderUid, InteractionData, LimitTradeParams, NativePriceResponse, NetworkFee, OnchainOrderData, Order, OrderData, OrderbookBinding, OrderClass, OrderCreation, OrderInteractions, OrderKind, OrderPostingResult, OrderQuoteRequest, OrderQuoteResponse, OrderStatus, OrderTraderParams, PaginationOptions, PartnerFee, PartnerFeePolicy, QuoteAmountsAndCosts, QuoteData, QuoteResults, SellTokenSource, SettlementEvent, SignedCancellations, SignedOrder, SigningScheme, SolverCompetitionOrder, SolverCompetitionResponse, SolverExecution, SolverSettlement, StoredOrderQuote, TotalSurplus, Trade, TradeParams, GetTradesRequest, TradingAppDataInfo, TransactionRequest, TwapCreateParams, TwapCreateResult, TypedDataDomain, TypedDataEnvelope, TypedDataField, ValidationResult, WrappedNativeToken } from "./raw/default.js";
export type { ContractReadCallback, CowEnv, CustomEip1271Callback, DigestSignerCallback, TypedDataSignerCallback } from "./callbacks.js";
export { CowError, isCowError, isRetryable, isUserRejection, normalizeError, retryAfterMs } from "./errors.js";
export type { CowErrorData, OrderBookErrorType, OrderBookRejectionCategory } from "./errors.js";
export { withRetry } from "./retry.js";
export type { RetryOptions } from "./retry.js";
export type { SchemaVersion, WasmEnvelope } from "./envelope.js";
export type { CowFetchCallback, CowFetchRequest, CowFetchResponse, HttpTransportConfig, IpfsClientConfig, JitterStrategyConfig, LimiterScopeConfig, RequestRateLimiterConfig, RetryPolicyConfig, SdkClientOptions, SigningOptions, SubgraphClientConfig, TradingClientConfig, TransportPolicyConfig, WalletConfig } from "./options.js";
