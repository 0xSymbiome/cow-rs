/// <reference lib="esnext.disposable" />
import * as raw from "./raw/orderbook.js";
import type { WasmEnvelope } from "./envelope.js";
import type { CommonClientConfig, SdkClientOptions, SigningOptions } from "./options.js";
import type { CustomEip1271Callback, DigestSignerCallback, Eip1193RequestCallback, TypedDataSignerCallback } from "./callbacks.js";
export interface OrderBookClientConfig extends CommonClientConfig {
}
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
export type { InitInput } from "./raw/orderbook.js";
export declare class OrderBookClient {
    #private;
    constructor(config: OrderBookClientConfig);
    cancelOrders(signed: raw.SignedCancellationsInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<{
        cancelled: true;
    }>>;
    getNativePrice(token: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.NativePriceResponseDto>>;
    getOrder(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.OrderDto>>;
    getOrders(owner: string, pagination?: raw.PaginationOptions | null, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.OrderDto[]>>;
    getOrderMultiEnv(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.OrderDto>>;
    getTxOrders(txHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.OrderDto[]>>;
    getVersion(options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    getOrderLink(orderUid: string): WasmEnvelope<string>;
    getQuote(request: raw.OrderQuoteRequestInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.OrderQuoteResponseDto>>;
    getTrades(query: raw.TradesQueryInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.TradeDto[]>>;
    getOrderCompetitionStatus(orderUid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.CompetitionOrderStatusDto>>;
    getTotalSurplus(owner: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.TotalSurplusDto>>;
    getSolverCompetition(auctionId: number, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.SolverCompetitionResponseDto>>;
    getSolverCompetitionByTxHash(txHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.SolverCompetitionResponseDto>>;
    getAppData(appDataHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.AppDataObjectDto>>;
    uploadAppData(appDataHash: string, fullAppData: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<{
        uploaded: true;
    }>>;
    sendOrder(signed: raw.SignedOrderDto, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    sendOrderCreation(input: raw.OrderCreationInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    dispose(): void;
    [Symbol.dispose](): void;
}
export declare function buildCancelOrderTx(params: raw.OrderTraderParametersInput): WasmEnvelope<raw.TransactionRequestDto>;
export declare function buildPresignTx(params: raw.OrderTraderParametersInput): WasmEnvelope<raw.TransactionRequestDto>;
export declare function computeOrderUid(input: raw.OrderInput, chainId: number, owner: string): WasmEnvelope<raw.GeneratedOrderUidDto>;
export declare function decodeEthFlowLog(log: raw.EventLogInput): WasmEnvelope<raw.EthFlowEventDto>;
export declare function decodeSettlementLog(log: raw.EventLogInput): WasmEnvelope<raw.SettlementEventDto>;
export declare function deploymentAddresses(chainId: number, env?: string | null): WasmEnvelope<raw.DeploymentAddressesDto>;
export declare function domainSeparator(chainId: number): string;
export declare function eip1271SignaturePayload(input: raw.OrderInput, ecdsaSignature: string): WasmEnvelope<string>;
export declare function orderTypedData(input: raw.OrderInput, chainId: number): WasmEnvelope<raw.TypedDataEnvelopeDto>;
export declare function signCancellationEthSignDigest(orderUids: string[], chainId: number, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedCancellationsInput>>;
export declare function signCancellationWithEip1193(orderUids: string[], chainId: number, owner: string, requestCallback: Eip1193RequestCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedCancellationsInput>>;
export declare function signCancellationWithTypedDataSigner(orderUids: string[], chainId: number, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedCancellationsInput>>;
export declare function signOrderEthSignDigest(input: raw.OrderInput, chainId: number, owner: string, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrderDto>>;
export declare function signOrderWithCustomEip1271(input: raw.OrderInput, chainId: number, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrderDto>>;
export declare function signOrderWithEip1193(input: raw.OrderInput, chainId: number, owner: string, requestCallback: Eip1193RequestCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrderDto>>;
export declare function signOrderWithEip1271(input: raw.OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrderDto>>;
export declare function signOrderWithTypedDataSigner(input: raw.OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrderDto>>;
export declare function supportedChainIds(): Uint32Array;
export declare function wasmVersion(): string;
export declare function wrappedNativeToken(chainId: number): WasmEnvelope<raw.WrappedNativeTokenDto>;
export type { AppDataObjectDto, CompetitionAuctionDto, CompetitionOrderStatusDto, CompetitionOrderStatusKindDto, CowEip1271SignRequest, DeploymentAddressesDto, Eip1193Request, EthFlowEventDto, EthflowDataDto, EventLogInput, ExecutedAmountsDto, ExecutedProtocolFeeDto, GeneratedOrderUidDto, InteractionDataDto, NativePriceResponseDto, OnchainOrderDataDto, OrderClassDto, OrderCreationInput, OrderDto, OrderInput, OrderInteractionsDto, OrderKindDto, OrderQuoteRequestInput, OrderQuoteResponseDto, OrderStatusDto, OrderTraderParametersInput, PaginationOptions, QuoteDataDto, SettlementEventDto, SignedCancellationsInput, SignedOrderDto, SigningSchemeDto, SolverCompetitionOrderDto, SolverCompetitionResponseDto, SolverExecutionDto, SolverSettlementDto, StoredOrderQuoteDto, TokenBalanceDto, TotalSurplusDto, TradeDto, TradesQueryInput, TransactionRequestDto, TypedDataDomainDto, TypedDataEnvelopeDto, TypedDataFieldDto, WrappedNativeTokenDto } from "./raw/orderbook.js";
export type { CowEip1271SignCallback, CowEnv, CustomEip1271Callback, DigestSignerCallback, Eip1193RequestCallback, TypedDataSignerCallback } from "./callbacks.js";
export { CowError, isCowError, isRetryable, isUserRejection, normalizeError, retryAfterMs } from "./errors.js";
export type { CowErrorData, OrderBookErrorType, OrderBookRejectionCategory } from "./errors.js";
export { withRetry } from "./retry.js";
export type { RetryOptions } from "./retry.js";
export type { SchemaVersion, WasmEnvelope } from "./envelope.js";
export type { CowFetchCallback, CowFetchRequest, CowFetchResponse, HttpTransportConfig, JitterStrategyConfig, LimiterScopeConfig, RequestRateLimiterConfig, RetryPolicyConfig, SdkClientOptions, SigningOptions, TransportPolicyConfig, WalletConfig } from "./options.js";
