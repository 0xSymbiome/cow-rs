/// <reference lib="esnext.disposable" />
/* tslint:disable */
/* eslint-disable */

export interface OrderBookClientConfig {
    chainId: number;
    env?: string | null;
    apiKey?: string | null;
    transport: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



export interface WalletConfig {
    timeoutMs?: number;
}

export interface SigningOptions extends SdkClientOptions {
    walletConfig?: WalletConfig;
}

export type TypedDataSignerCallback = (
envelope: TypedDataEnvelopeDto,
) => Promise<string> | string;

export type Eip1193RequestCallback = (
request: { method: string; params?: unknown[] },
) => Promise<unknown> | unknown;

export type DigestSignerCallback = (
digest: string,
) => Promise<string> | string;

export type CowEip1271SignCallback = (
request: CowEip1271SignRequest,
) => Promise<string> | string;

export type CustomEip1271Callback = CowEip1271SignCallback;



export type CowFetchMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";

export interface CowFetchRequest {
    method: CowFetchMethod;
    url: string;
    headers: Record<string, string>;
    body?: string;
    timeoutMs?: number;
    signal?: AbortSignal;
}

export interface CowFetchResponse {
    status: number;
    statusText?: string;
    headers?: Record<string, string>;
    body?: string;
}

export type CowFetchCallback = (
request: CowFetchRequest,
) => Promise<CowFetchResponse> | CowFetchResponse;

export type HttpTransportConfig =
| { kind: "fetch"; fetch?: typeof globalThis.fetch }
| { kind: "callback"; callback: CowFetchCallback };



export type Value = unknown;
export type SdkError = WasmError;

export interface SdkClientOptions {
    timeoutMs?: number;
    signal?: AbortSignal;
}


/**
 * Custom EIP-1271 callback request.
 */
export interface CowEip1271SignRequest {
    /**
     * Original order input.
     */
    order: OrderInput;
    /**
     * Typed-data envelope.
     */
    typedData: TypedDataEnvelopeDto;
    /**
     * Owner or smart-account address.
     */
    owner: string;
    /**
     * Numeric chain id.
     */
    chainId: number;
}

/**
 * Deployment address output.
 */
export interface DeploymentAddressesDto {
    /**
     * Settlement contract.
     */
    settlement: string;
    /**
     * Vault relayer contract.
     */
    vaultRelayer: string;
    /**
     * EthFlow contract.
     */
    ethFlow: string;
}

/**
 * EIP-1193 request DTO.
 */
export interface Eip1193Request {
    /**
     * Provider method.
     */
    method: string;
    /**
     * Provider params.
     */
    params?: Value[];
}

/**
 * Explicit raw GraphQL query input.
 */
export interface SubgraphQueryInput {
    /**
     * Raw GraphQL document.
     */
    query: string;
    /**
     * Optional GraphQL variables.
     */
    variables?: Value;
    /**
     * Optional operation name.
     */
    operationName?: string;
}

/**
 * Generated order UID output.
 */
export interface GeneratedOrderUidDto {
    /**
     * Compact order UID.
     */
    orderUid: string;
    /**
     * Underlying order digest.
     */
    orderDigest: string;
}

/**
 * JS-visible typed error envelope for every wasm export.
 */
export type WasmError = { kind: "invalidInput"; schemaVersion: SchemaVersion; message: string; field?: string } | { kind: "unknownEnumValue"; schemaVersion: SchemaVersion; field: string; value: string } | { kind: "unsupportedChain"; schemaVersion: SchemaVersion; chainId: number } | { kind: "walletRequest"; schemaVersion: SchemaVersion; method: string; code?: number; message: string; data?: Value } | { kind: "walletTimeout"; schemaVersion: SchemaVersion; timeoutMs: number } | { kind: "transport"; schemaVersion: SchemaVersion; class: string; message: string; status?: number; headers?: [string, string][]; body?: string } | { kind: "orderbook"; schemaVersion: SchemaVersion; code?: string; message: string } | { kind: "subgraph"; schemaVersion: SchemaVersion; message: string } | { kind: "signing"; schemaVersion: SchemaVersion; message: string } | { kind: "appData"; schemaVersion: SchemaVersion; class?: string; message: string } | { kind: "forbiddenInteraction"; schemaVersion: SchemaVersion; target: string; reason: string } | { kind: "cancelled"; schemaVersion: SchemaVersion } | { kind: "internal"; schemaVersion: SchemaVersion; message: string } | { kind: "__unknown"; schemaVersion: SchemaVersion; raw: Value };

/**
 * Jitter strategy accepted by JS client constructors.
 */
export type JitterStrategyConfig = "none" | "full" | "equal" | "decorrelated";

/**
 * Order input shared by signing and UID exports.
 */
export interface OrderInput {
    /**
     * Sell token address.
     */
    sellToken: string;
    /**
     * Buy token address.
     */
    buyToken: string;
    /**
     * Optional receiver.
     */
    receiver?: string;
    /**
     * Sell amount.
     */
    sellAmount: string;
    /**
     * Buy amount.
     */
    buyAmount: string;
    /**
     * Valid-to timestamp.
     */
    validTo: number;
    /**
     * App-data hash.
     */
    appData: string;
    /**
     * Fee amount.
     */
    feeAmount: string;
    /**
     * Order side.
     */
    kind: OrderKindDto;
    /**
     * Partial fill flag.
     */
    partiallyFillable: boolean;
    /**
     * Sell balance source.
     */
    sellTokenBalance: TokenBalanceDto;
    /**
     * Buy balance destination.
     */
    buyTokenBalance: TokenBalanceDto;
}

/**
 * Order side accepted by wasm order inputs.
 */
export type OrderKindDto = "sell" | "buy";

/**
 * Order transaction helper parameters.
 */
export interface OrderTraderParametersInput {
    /**
     * Target order UID.
     */
    orderUid: string;
    /**
     * Optional chain-id override.
     */
    chainId?: number;
    /**
     * Optional environment override.
     */
    env?: string;
    /**
     * Optional settlement-contract overrides keyed by chain id.
     */
    settlementContractOverride?: Map<number, string>;
    /**
     * Optional `EthFlow` contract overrides keyed by chain id.
     */
    ethFlowContractOverride?: Map<number, string>;
}

/**
 * Orderbook order-creation input.
 */
export interface OrderCreationInput {
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Optional receiver.
     */
    receiver?: string;
    /**
     * Sell amount.
     */
    sellAmount: string;
    /**
     * Buy amount.
     */
    buyAmount: string;
    /**
     * Absolute UNIX expiry timestamp.
     */
    validTo: number;
    /**
     * Inline app-data payload.
     */
    appData?: string;
    /**
     * App-data hash.
     */
    appDataHash?: string;
    /**
     * Order-level fee amount. The orderbook accepts only zero.
     */
    feeAmount?: string;
    /**
     * Strict balance-check flag.
     */
    fullBalanceCheck?: boolean;
    /**
     * Order side.
     */
    kind: OrderKindDto;
    /**
     * Whether partial fills are allowed.
     */
    partiallyFillable?: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance?: TokenBalanceDto;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance?: TokenBalanceDto;
    /**
     * Signature scheme.
     */
    signingScheme: string;
    /**
     * Raw signature.
     */
    signature: string;
    /**
     * Effective owner.
     */
    from: string;
    /**
     * Optional quote id.
     */
    quoteId?: number;
}

/**
 * Orderbook quote request input.
 */
export interface OrderQuoteRequestInput {
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Optional explicit receiver.
     */
    receiver?: string;
    /**
     * Quote owner.
     */
    from: string;
    /**
     * Quote side.
     */
    kind: OrderKindDto;
    /**
     * Sell amount before fee for sell quotes.
     */
    sellAmountBeforeFee?: string;
    /**
     * Buy amount after fee for buy quotes.
     */
    buyAmountAfterFee?: string;
    /**
     * Relative validity duration in seconds.
     */
    validFor?: number;
    /**
     * Absolute UNIX expiry timestamp.
     */
    validTo?: number;
    /**
     * Inline app-data payload.
     */
    appData?: string;
    /**
     * App-data hash.
     */
    appDataHash?: string;
    /**
     * Whether partial fills are allowed.
     */
    partiallyFillable?: boolean;
    /**
     * Sell-token balance source.
     */
    sellTokenBalance?: TokenBalanceDto;
    /**
     * Buy-token balance destination.
     */
    buyTokenBalance?: TokenBalanceDto;
    /**
     * Quote-quality mode.
     */
    priceQuality?: string;
    /**
     * Expected signing scheme.
     */
    signingScheme?: string;
    /**
     * Whether the eventual order is expected to be on-chain.
     */
    onchainOrder?: boolean;
    /**
     * Optional verification gas limit.
     */
    verificationGasLimit?: number;
    /**
     * Optional request timeout in milliseconds.
     */
    timeout?: number;
}

/**
 * Pagination options shared by orderbook list helpers.
 */
export interface PaginationOptions {
    /**
     * Pagination offset.
     */
    offset?: number;
    /**
     * Pagination limit.
     */
    limit?: number;
}

/**
 * Rate-limiter bucket scope accepted by JS client constructors.
 */
export type LimiterScopeConfig = "global" | "perHost";

/**
 * Request-rate limiter override accepted by JS client constructors.
 */
export interface RequestRateLimiterConfig {
    /**
     * Request tokens granted per interval. Zero disables limiting.
     */
    tokensPerInterval?: number;
    /**
     * Limiter interval in milliseconds.
     */
    intervalMs?: number;
    /**
     * Bucket scope.
     */
    scope?: LimiterScopeConfig;
}

/**
 * Retry-policy override accepted by JS client constructors.
 */
export interface RetryPolicyConfig {
    /**
     * Maximum attempts, including the initial request.
     */
    maxAttempts?: number;
    /**
     * Base exponential-backoff delay in milliseconds.
     */
    baseDelayMs?: number;
    /**
     * Maximum exponential-backoff delay in milliseconds.
     */
    maxDelayMs?: number;
}

/**
 * Signed order DTO returned by wallet callback exports.
 */
export interface SignedOrderDto {
    /**
     * Compact order UID.
     */
    orderUid: string;
    /**
     * Signature payload submitted to the orderbook.
     */
    signature: string;
    /**
     * Signing scheme.
     */
    signingScheme: string;
    /**
     * Effective owner submitted as `from`.
     */
    from: string;
    /**
     * Underlying order digest.
     */
    orderDigest: string;
    /**
     * Typed-data envelope used for signing.
     */
    typedData: TypedDataEnvelopeDto;
    /**
     * Optional quote id.
     */
    quoteId?: number;
}

/**
 * Signed order-cancellation DTO.
 */
export interface SignedCancellationsInput {
    /**
     * Order UIDs to cancel.
     */
    orderUids: string[];
    /**
     * Cancellation signature.
     */
    signature: string;
    /**
     * ECDSA signing scheme.
     */
    signingScheme: string;
}

/**
 * Token-balance mode accepted by wasm order inputs.
 */
export type TokenBalanceDto = "erc20" | "external" | "internal";

/**
 * Trades query accepted by `OrderBookClient.getTrades`.
 */
export interface TradesQueryInput {
    /**
     * Owner filter. Set exactly one of `owner` or `orderUid`.
     */
    owner?: string;
    /**
     * Order UID filter. Set exactly one of `owner` or `orderUid`.
     */
    orderUid?: string;
    /**
     * Pagination offset.
     */
    offset?: number;
    /**
     * Pagination limit.
     */
    limit?: number;
}

/**
 * Transaction request DTO returned by transaction builders.
 */
export interface TransactionRequestDto {
    /**
     * Destination address.
     */
    to?: string;
    /**
     * Hex-encoded calldata.
     */
    data?: string;
    /**
     * Native value.
     */
    value?: string;
    /**
     * Gas limit.
     */
    gasLimit?: string;
}

/**
 * Transport-policy override accepted by JS client constructors.
 */
export interface TransportPolicyConfig {
    /**
     * Retry-policy override.
     */
    retryPolicy?: RetryPolicyConfig;
    /**
     * Rate-limiter override.
     */
    requestRateLimiter?: RequestRateLimiterConfig;
    /**
     * Retry jitter override.
     */
    jitterStrategy?: JitterStrategyConfig;
    /**
     * Optional transport user-agent value.
     */
    userAgent?: string;
    /**
     * Enables or disables transport tracing integration.
     */
    tracingEnabled?: boolean;
}

/**
 * Typed-data domain DTO.
 */
export interface TypedDataDomainDto {
    /**
     * Domain name.
     */
    name: string;
    /**
     * Domain version.
     */
    version: string;
    /**
     * Chain id.
     */
    chainId: number;
    /**
     * Verifying contract.
     */
    verifyingContract: string;
}

/**
 * Typed-data envelope DTO.
 */
export interface TypedDataEnvelopeDto {
    /**
     * Domain metadata.
     */
    domain: TypedDataDomainDto;
    /**
     * Primary type.
     */
    primaryType: string;
    /**
     * Type map.
     */
    types: Map<string, TypedDataFieldDto[]>;
    /**
     * Parsed message body.
     */
    message: Value;
}

/**
 * Typed-data field DTO.
 */
export interface TypedDataFieldDto {
    /**
     * Field name.
     */
    name: string;
    /**
     * Solidity field type.
     */
    type: string;
}

/**
 * Version tag carried by wasm output and error envelopes.
 */
export type SchemaVersion = "v1" | "__unknown";

/**
 * Versioned output envelope.
 */
export interface WasmEnvelope<T> {
    /**
     * Schema version.
     */
    schemaVersion: SchemaVersion;
    /**
     * Envelope payload.
     */
    value: T;
}


/**
 * Orderbook client backed by an explicitly configured HTTP transport.
 */
export class OrderBookClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Cancels orders through a signed cancellation payload.
     */
    cancelOrders(signed: SignedCancellationsInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<{ cancelled: true }>>;
    /**
     * Fetches a token's native price.
     */
    getNativePrice(token: string, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches an order by UID.
     */
    getOrder(orderUid: string, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches orders owned by an address.
     */
    getOrders(owner: string, pagination?: PaginationOptions | null, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches orders owned by an address.
     */
    getOrdersByOwner(owner: string, pagination?: PaginationOptions | null, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches a quote.
     */
    getQuote(request: OrderQuoteRequestInput, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches trades for an owner or order UID.
     */
    getTrades(query: TradesQueryInput, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Creates an orderbook client from a single config object.
     */
    constructor(config: OrderBookClientConfig);
    /**
     * Submits a signed order.
     */
    sendOrder(signed: SignedOrderDto, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    /**
     * Submits a raw order-creation payload.
     */
    sendOrderCreation(input: OrderCreationInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
}

/**
 * Initializes the wasm crate's panic hook once.
 */
export function __cow_sdk_wasm_init(): void;

/**
 * Builds a settlement cancellation transaction for an order UID.
 */
export function buildCancelOrderTx(params: OrderTraderParametersInput): WasmEnvelope<TransactionRequestDto>;

/**
 * Builds a settlement pre-sign transaction for an order UID.
 */
export function buildPresignTx(params: OrderTraderParametersInput): WasmEnvelope<TransactionRequestDto>;

/**
 * Computes the compact order UID and digest.
 */
export function computeOrderUid(input: OrderInput, chainId: number, owner: string): WasmEnvelope<GeneratedOrderUidDto>;

/**
 * Returns canonical deployment addresses for a chain and environment.
 */
export function deploymentAddresses(chainId: number, env?: string | null): WasmEnvelope<DeploymentAddressesDto>;

/**
 * Computes the EIP-712 domain separator for a supported chain.
 */
export function domainSeparator(chainId: number): string;

/**
 * Encodes a CoW EIP-1271 payload from an ECDSA signature.
 */
export function eip1271SignaturePayload(input: OrderInput, ecdsaSignature: string): WasmEnvelope<string>;

/**
 * Builds signer-facing order typed data.
 */
export function orderTypedData(input: OrderInput, chainId: number): WasmEnvelope<TypedDataEnvelopeDto>;

/**
 * Signs a cancellation digest through an explicit `eth_sign` callback.
 */
export function signCancellationEthSignDigest(orderUids: string[], chainId: number, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellationsInput>>;

/**
 * Signs cancellation typed data through an EIP-1193 callback.
 */
export function signCancellationWithEip1193(orderUids: string[], chainId: number, owner: string, requestCallback: Eip1193RequestCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellationsInput>>;

/**
 * Signs cancellation typed data through a typed-data callback.
 */
export function signCancellationWithTypedDataSigner(orderUids: string[], chainId: number, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellationsInput>>;

/**
 * Signs an order digest through an explicit `eth_sign` callback.
 */
export function signOrderEthSignDigest(input: OrderInput, chainId: number, owner: string, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through a custom EIP-1271 callback.
 */
export function signOrderWithCustomEip1271(input: OrderInput, chainId: number, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through an EIP-1193 request callback.
 */
export function signOrderWithEip1193(input: OrderInput, chainId: number, owner: string, requestCallback: Eip1193RequestCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through typed-data ECDSA and wraps it as EIP-1271.
 */
export function signOrderWithEip1271(input: OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through a typed-data callback.
 */
export function signOrderWithTypedDataSigner(input: OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Returns supported EVM chain ids.
 */
export function supportedChainIds(): Uint32Array;

/**
 * Returns the wasm crate version.
 */
export function wasmVersion(): string;
