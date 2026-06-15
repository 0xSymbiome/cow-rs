import * as raw from "./raw/cloudflare.js";
import type { WasmEnvelope } from "./envelope.js";
import type { CommonClientConfig, SdkClientOptions, SigningOptions, TradingClientConfig } from "./options.js";
import type { ContractReadCallback, CustomEip1271Callback, DigestSignerCallback, Eip1193RequestCallback, TypedDataSignerCallback } from "./callbacks.js";
export interface OrderBookClientConfig extends CommonClientConfig {
}
export declare function initialize(module: WebAssembly.Module | raw.InitInput): Promise<void>;
export default initialize;
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
    getAppData(appDataHash: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.AppDataObjectDto>>;
    uploadAppData(appDataHash: string, fullAppData: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<{
        uploaded: true;
    }>>;
    sendOrder(signed: raw.SignedOrderDto, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    sendOrderCreation(input: raw.OrderCreationInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    dispose(): void;
}
export declare class TradingClient {
    #private;
    constructor(config: TradingClientConfig);
    buildSellNativeCurrencyTx(order: raw.OrderInput, quoteId: number, from: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.BuiltSellNativeCurrencyTxDto>>;
    getCowProtocolAllowance(params: raw.AllowanceParametersInput, readContractCallback: ContractReadCallback, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    buildApprovalTx(params: raw.ApprovalParametersInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.TransactionRequestDto>>;
    getQuote(params: raw.SwapParametersInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<raw.QuoteResultsDto>>;
    postLimitOrder(params: raw.LimitTradeParametersInput, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.OrderPostingResultDto>>;
    postSwapOrder(params: raw.SwapParametersInput, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.OrderPostingResultDto>>;
    postSwapOrderFromQuote(quoteResults: raw.QuoteResultsDto, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.OrderPostingResultDto>>;
    postSwapOrderWithEip1271(params: raw.SwapParametersInput, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.OrderPostingResultDto>>;
    dispose(): void;
}
export declare function appDataDoc(doc: raw.AppDataDocInput): WasmEnvelope<raw.AppDataDocDto>;
export declare function appDataHexToCid(appDataHex: string): WasmEnvelope<string>;
export declare function appDataInfo(doc: raw.AppDataDocInput): WasmEnvelope<raw.AppDataInfoDto>;
export declare function buildCancelOrderTx(params: raw.OrderTraderParametersInput): WasmEnvelope<raw.TransactionRequestDto>;
export declare function buildPresignTx(params: raw.OrderTraderParametersInput): WasmEnvelope<raw.TransactionRequestDto>;
export declare function cidToAppDataHex(cid: string): WasmEnvelope<string>;
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
export declare function validateAppDataDoc(doc: raw.AppDataDocInput): WasmEnvelope<raw.ValidationResultDto>;
export declare function wasmVersion(): string;
export type { AllowanceParametersInput, AmountsDto, AppDataDocDto, AppDataDocInput, AppDataInfoDto, AppDataObjectDto, ApprovalParametersInput, BuiltSellNativeCurrencyTxDto, CompetitionOrderStatusDto, CompetitionOrderStatusKindDto, ContractCallDto, CostsDto, CowEip1271SignRequest, CowEnvDto, DeploymentAddressesDto, Eip1193Request, EthFlowEventDto, EthflowDataDto, EventLogInput, ExecutedAmountsDto, ExecutedProtocolFeeDto, FeeComponentDto, GeneratedOrderUidDto, InitInput, InteractionDataDto, LimitTradeParametersInput, NativePriceResponseDto, NetworkFeeDto, OnchainOrderDataDto, OrderClassDto, OrderCreationInput, OrderDataDto, OrderDto, OrderInput, OrderInteractionsDto, OrderKindDto, OrderPostingResultDto, OrderQuoteRequestInput, OrderQuoteResponseDto, OrderStatusDto, OrderTraderParametersInput, OrderBookRuntimeBindingDto, PaginationOptions, PartnerFeeDto, PartnerFeeInput, PartnerFeePolicyDto, PartnerFeePolicyInput, QuoteAmountsAndCostsDto, QuoteDataDto, QuoteResultsDto, SettlementEventDto, SignedCancellationsInput, SignedOrderDto, SigningSchemeDto, SolverExecutionDto, StoredOrderQuoteDto, SwapParametersInput, TokenBalanceDto, TotalSurplusDto, TradeDto, TradeParametersDto, TradesQueryInput, TradingAppDataInfoDto, TransactionRequestDto, TypedDataDomainDto, TypedDataEnvelopeDto, TypedDataFieldDto, ValidationResultDto } from "./raw/cloudflare.js";
export type { ContractReadCallback, CowEip1271SignCallback, CustomEip1271Callback, DigestSignerCallback, Eip1193RequestCallback, TypedDataSignerCallback } from "./callbacks.js";
export type { OrderBookRejectionCategory, CowError } from "./errors.js";
export type { SchemaVersion, WasmEnvelope } from "./envelope.js";
export type { HttpTransportConfig, SdkClientOptions, SigningOptions, TradingClientConfig, TransportPolicyConfig, WalletConfig } from "./options.js";
