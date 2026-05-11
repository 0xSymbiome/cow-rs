/// <reference lib="esnext.disposable" />
/* tslint:disable */
/* eslint-disable */

export interface IpfsClientConfig {
    ipfsUri?: string | null;
    transport: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



export interface OrderBookClientConfig {
    chainId: number;
    env?: string | null;
    apiKey?: string | null;
    transport: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



export interface SubgraphClientConfig {
    chainId: number;
    apiKey: string;
    transport: HttpTransportConfig;
    transportPolicy?: TransportPolicyConfig | null;
    timeoutMs?: number | null;
}



export interface TradingClientConfig {
    chainId: number;
    env?: string | null;
    appCode: string;
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



export type ContractReadCallback = (
request: ContractCallDto,
) => Promise<string> | string;



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
 * Allowance helper parameters.
 */
export interface AllowanceParametersInput {
    /**
     * ERC-20 token address.
     */
    tokenAddress: string;
    /**
     * Owner whose allowance should be inspected.
     */
    owner: string;
    /**
     * Optional chain-id override.
     */
    chainId?: number;
    /**
     * Optional environment override.
     */
    env?: string;
    /**
     * Optional vault-relayer deployment override.
     */
    vaultRelayerOverride?: string;
}

/**
 * App-data document input.
 */
export interface AppDataDocInput {
    /**
     * Application code.
     */
    appCode: string;
    /**
     * Metadata object.
     */
    metadata: Value;
    /**
     * Schema version.
     */
    version: string;
    /**
     * Optional environment label.
     */
    environment?: string;
}

/**
 * App-data document output.
 */
export interface AppDataDocDto {
    /**
     * App-data document.
     */
    document: Value;
}

/**
 * App-data info output.
 */
export interface AppDataInfoDto {
    /**
     * CID representation.
     */
    cid: string;
    /**
     * Deterministic app-data content.
     */
    appDataContent: string;
    /**
     * App-data hash.
     */
    appDataHex: string;
}

/**
 * App-data validation result.
 */
export interface ValidationResultDto {
    /**
     * Whether validation succeeded.
     */
    success: boolean;
    /**
     * Errors when validation failed.
     */
    errors?: string;
}

/**
 * Contract-read callback request.
 */
export interface ContractCallDto {
    /**
     * Target contract address.
     */
    address: string;
    /**
     * ABI method name.
     */
    method: string;
    /**
     * JSON ABI fragment.
     */
    abiJson: string;
    /**
     * JSON-encoded function arguments.
     */
    argsJson: string;
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
export type WasmError = { kind: "invalidInput"; schemaVersion: SchemaVersion; message: string; field?: string } | { kind: "unknownEnumValue"; schemaVersion: SchemaVersion; message: string; field: string; value: string } | { kind: "unsupportedChain"; schemaVersion: SchemaVersion; message: string; chainId: number } | { kind: "walletRequest"; schemaVersion: SchemaVersion; method: string; code?: number; message: string; data?: Value } | { kind: "walletTimeout"; schemaVersion: SchemaVersion; message: string; timeoutMs: number } | { kind: "transport"; schemaVersion: SchemaVersion; class: string; message: string; status?: number; headers?: [string, string][]; body?: string } | { kind: "orderbook"; schemaVersion: SchemaVersion; code?: string; message: string } | { kind: "subgraph"; schemaVersion: SchemaVersion; message: string } | { kind: "signing"; schemaVersion: SchemaVersion; message: string } | { kind: "appData"; schemaVersion: SchemaVersion; class?: string; message: string } | { kind: "forbiddenInteraction"; schemaVersion: SchemaVersion; message: string; target: string; reason: string } | { kind: "cancelled"; schemaVersion: SchemaVersion; message: string } | { kind: "internal"; schemaVersion: SchemaVersion; message: string } | { kind: "__unknown"; schemaVersion: SchemaVersion; message: string; raw: Value };

/**
 * Jitter strategy accepted by JS client constructors.
 */
export type JitterStrategyConfig = "none" | "full" | "equal" | "decorrelated";

/**
 * Limit-order parameters accepted by trading posting helpers.
 */
export interface LimitTradeParametersInput {
    /**
     * Order side.
     */
    kind: OrderKindDto;
    /**
     * Optional owner override.
     */
    owner?: string;
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Sell-token decimals.
     */
    sellTokenDecimals: number;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Buy-token decimals.
     */
    buyTokenDecimals: number;
    /**
     * Sell amount before transformations.
     */
    sellAmount: string;
    /**
     * Buy amount before transformations.
     */
    buyAmount: string;
    /**
     * Optional quote id.
     */
    quoteId?: number;
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
     * Optional slippage tolerance in basis points.
     */
    slippageBps?: number;
    /**
     * Optional receiver override.
     */
    receiver?: string;
    /**
     * Optional relative validity duration.
     */
    validFor?: number;
    /**
     * Optional absolute UNIX expiry timestamp.
     */
    validTo?: number;
    /**
     * Optional partner-fee metadata.
     */
    partnerFee?: PartnerFeeInput;
}

/**
 * Minimal quote-results payload accepted by `TradingClient.postSwapOrderFromQuote`.
 */
export interface QuoteResultsInput {
    /**
     * Order returned by a previous quote response.
     */
    orderToSign: OrderInput;
    /**
     * Upstream quote response reference.
     */
    quoteResponse?: QuoteResponseRefInput;
    /**
     * Direct quote id fallback.
     */
    quoteId?: number;
}

/**
 * Native-currency sell transaction bundle.
 */
export interface BuiltSellNativeCurrencyTxDto {
    /**
     * Deterministic order UID.
     */
    orderUid: string;
    /**
     * Transaction request to submit.
     */
    transaction: TransactionRequestDto;
    /**
     * Unsigned order encoded by the transaction.
     */
    orderToSign: OrderInput;
    /**
     * Effective order owner.
     */
    from: string;
}

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
 * Partner-fee input accepted by trading swap parameters.
 */
export type PartnerFeeInput = PartnerFeePolicyInput | PartnerFeePolicyInput[];

/**
 * Partner-fee policy input for trading swap parameters.
 */
export interface PartnerFeePolicyInput {
    /**
     * Volume fee in basis points.
     */
    volumeBps?: number;
    /**
     * Surplus fee in basis points.
     */
    surplusBps?: number;
    /**
     * Price-improvement fee in basis points.
     */
    priceImprovementBps?: number;
    /**
     * Maximum volume fee in basis points.
     */
    maxVolumeBps?: number;
    /**
     * Fee recipient address.
     */
    recipient: string;
}

/**
 * Quote-response reference accepted by quote-derived posting helpers.
 */
export interface QuoteResponseRefInput {
    /**
     * Upstream quote id.
     */
    id?: number;
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
 * Trading swap-parameter input.
 */
export interface SwapParametersInput {
    /**
     * Order side.
     */
    kind: OrderKindDto;
    /**
     * Optional owner override.
     */
    owner?: string;
    /**
     * Sell-token address.
     */
    sellToken: string;
    /**
     * Sell-token decimals.
     */
    sellTokenDecimals: number;
    /**
     * Buy-token address.
     */
    buyToken: string;
    /**
     * Buy-token decimals.
     */
    buyTokenDecimals: number;
    /**
     * Amount interpreted according to `kind`.
     */
    amount: string;
    /**
     * Optional environment override.
     */
    env?: string;
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
     * Optional slippage tolerance in basis points.
     */
    slippageBps?: number;
    /**
     * Optional receiver override.
     */
    receiver?: string;
    /**
     * Optional relative validity duration.
     */
    validFor?: number;
    /**
     * Optional absolute UNIX expiry timestamp.
     */
    validTo?: number;
    /**
     * Optional partner-fee metadata.
     */
    partnerFee?: PartnerFeeInput;
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
 * IPFS app-data client backed by an explicitly configured HTTP transport.
 *
 * Construct this client when JavaScript needs to fetch app-data documents by
 * CID or app-data hash while preserving SDK retry, timeout, and cancellation
 * behavior.
 */
export class IpfsClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Fetches and parses an app-data document by CID.
     *
     * The CID is resolved through the configured gateway and transport. The
     * returned document is normalized into the SDK app-data DTO shape.
     *
     * @param cid Canonical IPFS CID for the app-data document.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the app-data document.
     * @throws SdkError for invalid CID, transport failure, timeout, or parse failure.
     */
    fetchAppDataFromCid(cid: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<AppDataDocDto>>;
    /**
     * Fetches and parses an app-data document by app-data hash.
     *
     * The helper converts the app-data hash to the canonical CID before
     * fetching through the configured gateway.
     *
     * @param appDataHex App-data hash as a `0x`-prefixed hex string.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the app-data document.
     * @throws SdkError for invalid hash, transport failure, timeout, or parse failure.
     */
    fetchAppDataFromHex(appDataHex: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<AppDataDocDto>>;
    /**
     * Creates an IPFS app-data client from a single config object.
     *
     * The config must include `transport`. Optional `ipfsUri` overrides the
     * default gateway base, while timeout, signal, and policy fields become
     * defaults for method calls.
     *
     * @param config IPFS client configuration.
     * @throws SdkError when transport, policy, timeout, or gateway config is invalid.
     */
    constructor(config: IpfsClientConfig);
}

/**
 * Orderbook client backed by an explicitly configured HTTP transport.
 *
 * Construct this client when JavaScript needs direct access to quote,
 * submission, lookup, trade, native-price, app-data, and cancellation orderbook
 * endpoints. The client owns one callback registration and releases raw wasm
 * resources through the facade `dispose()` method.
 */
export class OrderBookClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Submits signed off-chain order cancellations.
     *
     * Build the signed cancellation payload with one of the cancellation
     * signing helpers, then submit it through the same orderbook runtime
     * configuration used for order operations.
     *
     * @param signed Signed cancellation payload.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing `{ cancelled: true }` on success.
     * @throws SdkError for invalid UID, signature, transport failure, or timeout.
     */
    cancelOrders(signed: SignedCancellationsInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<{ cancelled: true }>>;
    /**
     * Fetches a token's native price from the orderbook API.
     *
     * The token must be an EVM address. The returned value follows the
     * orderbook native-price response shape.
     *
     * @param token Token address to price.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing native price data.
     * @throws SdkError for invalid token address, transport failure, or timeout.
     */
    getNativePrice(token: string, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches one order by its canonical order UID.
     *
     * The UID must be the full 56-byte CoW order UID encoded as a `0x`-prefixed
     * string. The response is returned in the orderbook wire DTO shape.
     *
     * @param orderUid Full order UID to look up.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the order response.
     * @throws SdkError for invalid UID, not-found responses, transport failure, or timeout.
     */
    getOrder(orderUid: string, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches orders owned by an address with optional pagination.
     *
     * The owner address is validated before the request is dispatched. The
     * response preserves the typed orderbook order shape.
     *
     * @param owner Owner address to query.
     * @param pagination Optional offset and limit.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing matching orders.
     * @throws SdkError for invalid owner, transport failure, timeout, or cancellation.
     */
    getOrders(owner: string, pagination?: PaginationOptions | null, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches orders owned by an address.
     *
     * This compatibility method is equivalent to `getOrders` and accepts the
     * same pagination options. New TypeScript code can use `getOrders`.
     *
     * @param owner Owner address to query.
     * @param pagination Optional offset and limit.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing matching orders.
     * @throws SdkError for invalid owner, transport failure, timeout, or cancellation.
     */
    getOrdersByOwner(owner: string, pagination?: PaginationOptions | null, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches a price quote from the orderbook API.
     *
     * The request is converted to the typed orderbook quote request and sent
     * through the configured transport. Per-call options can override the
     * constructor timeout or attach an `AbortSignal`.
     *
     * @param request Quote request DTO.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the quote response.
     * @throws SdkError for invalid input, transport failure, timeout, or cancellation.
     */
    getQuote(request: OrderQuoteRequestInput, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches trades for exactly one owner address or order UID.
     *
     * The query must set one of `owner` or `orderUid`, not both. Optional
     * pagination fields are forwarded to the orderbook request.
     *
     * @param query Trade query DTO.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing matching trades.
     * @throws SdkError when the query is ambiguous or transport fails.
     */
    getTrades(query: TradesQueryInput, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Creates an orderbook client from a single config object.
     *
     * The config must include `chainId` and `transport`. The optional
     * `timeoutMs`, `signal`, and `transportPolicy` fields become defaults for
     * calls made through this client unless a method call overrides them.
     *
     * @param config Orderbook client configuration.
     * @throws SdkError when the chain, environment, transport, or policy is invalid.
     */
    constructor(config: OrderBookClientConfig);
    /**
     * Submits a signed order to the orderbook.
     *
     * The signed DTO normally comes from a signing helper in the same package.
     * The SDK reconstructs the typed order creation payload and returns the
     * order UID assigned by the orderbook service.
     *
     * @param signed Signed order DTO including typed data, signature, owner, and scheme.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the submitted order UID.
     * @throws SdkError for invalid signatures, transport failure, timeout, or rejection.
     */
    sendOrder(signed: SignedOrderDto, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    /**
     * Submits a raw order-creation payload to the orderbook.
     *
     * Use this method when the host already has a complete orderbook
     * `OrderCreation` shape and does not need the facade to reconstruct it
     * from a signed-order DTO.
     *
     * @param input Raw order-creation DTO.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the submitted order UID.
     * @throws SdkError for malformed input, transport failure, timeout, or rejection.
     */
    sendOrderCreation(input: OrderCreationInput, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
}

/**
 * Read-only subgraph client backed by an explicitly configured transport.
 *
 * Construct this client when JavaScript needs protocol totals, recent volume,
 * or custom GraphQL query execution through the same transport and policy
 * model as the orderbook clients.
 */
export class SubgraphClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Fetches recent daily volume rows.
     *
     * The `days` value controls how many recent daily buckets the subgraph
     * query requests.
     *
     * @param days Number of daily buckets to fetch.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing daily volume rows.
     * @throws SdkError for invalid query shape, transport failure, or timeout.
     */
    getLastDaysVolume(days: number, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches recent hourly volume rows.
     *
     * The `hours` value controls how many recent hourly buckets the subgraph
     * query requests.
     *
     * @param hours Number of hourly buckets to fetch.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing hourly volume rows.
     * @throws SdkError for invalid query shape, transport failure, or timeout.
     */
    getLastHoursVolume(hours: number, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Fetches aggregate protocol totals from the subgraph.
     *
     * The request uses the client's configured chain, API key, transport, and
     * transport policy.
     *
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing aggregate totals.
     * @throws SdkError for transport, cancellation, timeout, or subgraph errors.
     */
    getTotals(options?: SdkClientOptions | null): Promise<any>;
    /**
     * Creates a subgraph client from a single config object.
     *
     * The config must include `chainId`, `apiKey`, and `transport`. Optional
     * timeout, signal, and policy fields become client defaults for later
     * method calls.
     *
     * @param config Subgraph client configuration.
     * @throws SdkError when the chain, API key, transport, or policy is invalid.
     */
    constructor(config: SubgraphClientConfig);
    /**
     * Runs a caller-provided GraphQL query against the configured subgraph.
     *
     * Use this method when the built-in totals or volume helpers are too
     * narrow. Variables and operation name are forwarded when present.
     *
     * @param request GraphQL query, variables, and optional operation name.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the JSON GraphQL response.
     * @throws SdkError for transport, timeout, cancellation, or GraphQL errors.
     */
    runQuery(request: SubgraphQueryInput, options?: SdkClientOptions | null): Promise<any>;
}

/**
 * High-level trading client backed by an explicitly configured orderbook.
 *
 * Construct this client when JavaScript needs quote, sign, post, allowance,
 * and native-sell helper workflows rather than direct orderbook calls. The
 * client keeps app-code, chain, environment, transport, and policy defaults.
 */
export class TradingClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Builds the transaction for a native-currency sell order.
     *
     * The helper validates that the order sells the native-token sentinel,
     * resolves the EthFlow deployment, and returns a transaction request for
     * the host wallet to submit.
     *
     * @param order Unsigned native-sell order DTO.
     * @param quoteId Quote identifier returned by the orderbook.
     * @param from Transaction sender address.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing order UID and transaction request.
     * @throws SdkError when the order, chain, deployment, or sender is invalid.
     */
    buildSellNativeCurrencyTx(order: OrderInput, quoteId: bigint, from: string, options?: SdkClientOptions | null): Promise<WasmEnvelope<BuiltSellNativeCurrencyTxDto>>;
    /**
     * Reads CoW Protocol allowance through a read-only contract callback.
     *
     * The SDK builds the contract call while the JavaScript host performs the
     * actual chain read. Use this when a TypeScript runtime owns the RPC
     * provider.
     *
     * @param params Allowance parameters DTO.
     * @param readContractCallback Callback that executes the read-only call.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing the allowance amount string.
     * @throws SdkError for invalid parameters, callback failure, timeout, or cancellation.
     */
    getCowProtocolAllowance(params: AllowanceParametersInput, readContractCallback: ContractReadCallback, options?: SdkClientOptions | null): Promise<WasmEnvelope<string>>;
    /**
     * Fetches a quote without signing or submitting an order.
     *
     * Use this method when a host wants to preview the quote response before
     * asking a wallet to sign or before constructing a post request.
     *
     * @param params Swap parameters DTO.
     * @param options Optional per-call cancellation and timeout settings.
     * @returns A versioned envelope containing quote results.
     * @throws SdkError for invalid parameters, transport failure, timeout, or cancellation.
     */
    getQuote(params: SwapParametersInput, options?: SdkClientOptions | null): Promise<any>;
    /**
     * Creates a trading client from a single config object.
     *
     * The config must include `chainId`, `appCode`, and `transport`. Optional
     * environment, API key, timeout, signal, and transport policy fields become
     * defaults for all trading methods.
     *
     * @param config Trading client configuration.
     * @throws SdkError when chain, app-code, environment, transport, or policy validation fails.
     */
    constructor(config: TradingClientConfig);
    /**
     * Signs and posts a limit order through a typed-data callback.
     *
     * This helper follows the native limit-order trading path and lets the SDK
     * build, sign, and submit the order using the configured orderbook.
     *
     * @param params Limit-order parameters DTO.
     * @param owner Owner address to bind to the order when absent from params.
     * @param signerCallback Callback that signs the typed-data envelope.
     * @param options Optional cancellation, timeout, and wallet timeout settings.
     * @returns A versioned envelope containing order posting output.
     * @throws SdkError for invalid input, wallet failure, timeout, or rejection.
     */
    postLimitOrder(params: LimitTradeParametersInput, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<any>;
    /**
     * Quotes, signs, and posts a swap order through a typed-data callback.
     *
     * The SDK fetches a quote, builds the order to sign, invokes the callback
     * with the EIP-712 envelope, posts the signed order, and returns posting
     * output from the trading workflow.
     *
     * @param params Swap parameters DTO.
     * @param owner Owner address to bind to the order.
     * @param signerCallback Callback that signs the typed-data envelope.
     * @param options Optional cancellation, timeout, and wallet timeout settings.
     * @returns A versioned envelope containing order posting output.
     * @throws SdkError for invalid input, quote failure, wallet failure, timeout, or rejection.
     */
    postSwapOrder(params: SwapParametersInput, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<any>;
    /**
     * Signs and posts a previously quoted swap order.
     *
     * Use this method when a host has already called `getQuote` and wants to
     * reuse that quote result for posting without requesting a new quote.
     *
     * @param quoteResults Quote result DTO returned by `getQuote`.
     * @param owner Owner address to bind to the order.
     * @param signerCallback Callback that signs the typed-data envelope.
     * @param options Optional cancellation, timeout, and wallet timeout settings.
     * @returns A versioned envelope containing order posting output.
     * @throws SdkError for invalid quote data, wallet failure, timeout, or rejection.
     */
    postSwapOrderFromQuote(quoteResults: QuoteResultsInput, owner: string, signerCallback: TypedDataSignerCallback, options?: SigningOptions | null): Promise<any>;
    /**
     * Quotes and posts a swap order with a custom EIP-1271 signature callback.
     *
     * Use this method when a smart-account runtime owns final contract
     * signature production. The SDK still quotes the swap, builds typed data,
     * posts the signed order, and returns posting output.
     *
     * @param params Swap parameters DTO.
     * @param owner Smart-account owner address.
     * @param customCallback Callback that returns the final EIP-1271 signature.
     * @param options Optional cancellation, timeout, and wallet timeout settings.
     * @returns A versioned envelope containing order posting output.
     * @throws SdkError for invalid input, quote failure, callback failure, timeout, or rejection.
     */
    postSwapOrderWithEip1271(params: SwapParametersInput, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<any>;
}

/**
 * Initializes the wasm crate's panic hook once.
 */
export function __cow_sdk_wasm_init(): void;

/**
 * Builds a normalized app-data document without deriving storage metadata.
 *
 * This helper is useful when a host wants to inspect or modify the canonical
 * document shape before separately deriving app-data information.
 *
 * @param doc App-data document input accepted by the SDK schema.
 * @returns A versioned envelope containing the normalized document.
 * @throws SdkError when the input cannot be normalized.
 */
export function appDataDoc(doc: AppDataDocInput): WasmEnvelope<AppDataDocDto>;

/**
 * Converts a `0x`-prefixed app-data hash into the canonical IPFS CID.
 *
 * The conversion is pure and uses the same app-data multicodec and multihash
 * rules as the Rust app-data crate.
 *
 * @param appDataHex App-data hash as a `0x`-prefixed hex string.
 * @returns A versioned envelope containing the CID string.
 * @throws SdkError when the hash is malformed.
 */
export function appDataHexToCid(appDataHex: string): WasmEnvelope<string>;

/**
 * Builds app-data content and returns its deterministic hash and CID.
 *
 * Use this when a JavaScript host wants the SDK to construct the canonical
 * document and expose the values needed for order submission and storage.
 *
 * @param doc App-data document input accepted by the SDK schema.
 * @returns A versioned envelope containing document, hash, CID, and hex data.
 * @throws SdkError when the document cannot be normalized or hashed.
 */
export function appDataInfo(doc: AppDataDocInput): WasmEnvelope<AppDataInfoDto>;

/**
 * Builds a settlement cancellation transaction for an order UID.
 *
 * The returned transaction request targets the Settlement contract and encodes
 * `invalidateOrder(bytes)`. The host wallet remains responsible for submitting
 * and observing the transaction.
 *
 * @param params Order UID, chain, environment, and optional deployment override.
 * @returns A versioned envelope containing the transaction request DTO.
 * @throws SdkError when the chain, deployment, or order UID is invalid.
 */
export function buildCancelOrderTx(params: OrderTraderParametersInput): WasmEnvelope<TransactionRequestDto>;

/**
 * Builds a settlement pre-sign transaction for an order UID.
 *
 * The returned transaction request targets the Settlement contract and encodes
 * `setPreSignature(bytes,bool)` with the order UID and `true` flag. The host
 * wallet remains responsible for transaction submission.
 *
 * @param params Order UID, chain, environment, and optional deployment override.
 * @returns A versioned envelope containing the transaction request DTO.
 * @throws SdkError when the chain, deployment, or order UID is invalid.
 */
export function buildPresignTx(params: OrderTraderParametersInput): WasmEnvelope<TransactionRequestDto>;

/**
 * Converts a canonical IPFS CID into a `0x`-prefixed app-data hash.
 *
 * Use this helper when an order or metadata path starts from a CID but the
 * orderbook request needs the app-data hash form.
 *
 * @param cid Canonical CID string for an app-data document.
 * @returns A versioned envelope containing the `0x`-prefixed hash.
 * @throws SdkError when the CID does not match the supported app-data shape.
 */
export function cidToAppDataHex(cid: string): WasmEnvelope<string>;

/**
 * Computes the canonical order UID and order digest for an unsigned order.
 *
 * The UID combines the EIP-712 order digest, owner address, and validity
 * timestamp using the same packing rules as the native Rust SDK.
 *
 * @param input Unsigned order fields to hash and pack.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @param owner Order owner address included in the UID suffix.
 * @returns A versioned envelope with `orderUid` and `orderDigest`.
 * @throws SdkError when the order, owner, or chain id is invalid.
 */
export function computeOrderUid(input: OrderInput, chainId: number, owner: string): WasmEnvelope<GeneratedOrderUidDto>;

/**
 * Returns canonical CoW Protocol deployment addresses for a chain.
 *
 * The optional environment selects production or staging deployment data. When
 * omitted, the helper uses the SDK default environment.
 *
 * @param chainId EVM chain id to resolve.
 * @param env Optional CoW environment name, such as `prod` or `staging`.
 * @returns Settlement, VaultRelayer, EthFlow, and AllowListAuth addresses.
 * @throws SdkError when the chain or environment is unsupported.
 */
export function deploymentAddresses(chainId: number, env?: string | null): WasmEnvelope<DeploymentAddressesDto>;

/**
 * Computes the CoW Protocol EIP-712 domain separator for a supported chain.
 *
 * Use this helper when a JavaScript host needs to compare the domain hash used
 * by the Rust SDK with another signing stack. The input is an EVM chain id,
 * not a CoW environment selector.
 *
 * @param chainId EVM chain id supported by the deployment registry.
 * @returns The `0x`-prefixed 32-byte domain separator.
 * @throws SdkError when the chain is not supported.
 */
export function domainSeparator(chainId: number): string;

/**
 * Encodes a CoW EIP-1271 payload from an ECDSA order signature.
 *
 * Use this pure helper when a smart-account flow already has the wrapped ECDSA
 * signature and needs the contract-signature payload bytes expected by CoW
 * Protocol order submission.
 *
 * @param input Unsigned order used to derive the EIP-1271 payload.
 * @param ecdsaSignature Wrapped ECDSA signature as a `0x`-prefixed string.
 * @returns A versioned envelope containing the encoded EIP-1271 payload.
 * @throws SdkError when the order or signature is invalid.
 */
export function eip1271SignaturePayload(input: OrderInput, ecdsaSignature: string): WasmEnvelope<string>;

/**
 * Builds signer-facing EIP-712 typed data for an unsigned order.
 *
 * The returned envelope contains the domain, type map, primary type, and
 * order message that wallet libraries expect for EIP-712 signing. It is
 * deterministic for the provided order and chain id.
 *
 * @param input Unsigned order fields using the facade order DTO shape.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @returns A versioned envelope containing typed-data DTO fields.
 * @throws SdkError when order parsing or chain validation fails.
 */
export function orderTypedData(input: OrderInput, chainId: number): WasmEnvelope<TypedDataEnvelopeDto>;

/**
 * Signs a cancellation digest through an explicit `eth_sign` callback.
 *
 * The SDK computes the canonical cancellation digest for the provided UIDs and
 * passes it to the digest signer callback as a `0x`-prefixed string.
 *
 * @param orderUids One or more full order UIDs to cancel.
 * @param chainId EVM chain id used for the cancellation digest.
 * @param digestSigner Callback that signs the digest string.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing signed cancellations.
 * @throws SdkError for empty input, invalid UID, callback failure, or timeout.
 */
export function signCancellationEthSignDigest(orderUids: string[], chainId: number, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellationsInput>>;

/**
 * Signs cancellation typed data through an EIP-1193 callback.
 *
 * The callback receives an `eth_signTypedData_v4` request object. Use this
 * helper when an injected wallet or wallet client owns typed-data signing.
 *
 * @param orderUids One or more full order UIDs to cancel.
 * @param chainId EVM chain id used for the cancellation domain.
 * @param owner Owner address included in the EIP-1193 request parameters.
 * @param requestCallback Callback that executes the EIP-1193 request.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing signed cancellations.
 * @throws SdkError for invalid input, wallet failure, timeout, or cancellation.
 */
export function signCancellationWithEip1193(orderUids: string[], chainId: number, owner: string, requestCallback: Eip1193RequestCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellationsInput>>;

/**
 * Signs cancellation typed data through a typed-data callback.
 *
 * The SDK builds the batch cancellation EIP-712 payload for the provided order
 * UIDs and asks the callback to sign it. The response can be submitted through
 * `OrderBookClient.cancelOrders`.
 *
 * @param orderUids One or more full order UIDs to cancel.
 * @param chainId EVM chain id used for the cancellation domain.
 * @param typedDataSigner Callback that signs the typed-data envelope.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing signed cancellations.
 * @throws SdkError for empty input, invalid UID, callback failure, or timeout.
 */
export function signCancellationWithTypedDataSigner(orderUids: string[], chainId: number, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedCancellationsInput>>;

/**
 * Signs an order digest through an explicit `eth_sign` callback.
 *
 * The SDK computes the canonical order digest, passes the digest as a
 * `0x`-prefixed string to the callback, normalizes the signature, and returns
 * an `ethsign` signed-order DTO.
 *
 * @param input Unsigned order fields to sign.
 * @param chainId EVM chain id used for the digest.
 * @param owner Owner address used in the generated order UID.
 * @param digestSigner Callback that signs the digest string.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed order.
 * @throws SdkError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderEthSignDigest(input: OrderInput, chainId: number, owner: string, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through a custom EIP-1271 callback.
 *
 * Use this method when the JavaScript host owns the smart-account or
 * account-abstraction client and can return the final contract signature
 * directly. The SDK still builds typed data and the deterministic order UID.
 *
 * @param input Unsigned order to sign.
 * @param chainId EVM chain id for the EIP-712 domain.
 * @param owner Smart-account owner address used in the generated order UID.
 * @param customCallback Callback that returns the final EIP-1271 signature.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed-order DTO.
 * @throws SdkError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderWithCustomEip1271(input: OrderInput, chainId: number, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through an EIP-1193 request callback.
 *
 * The callback receives an `eth_signTypedData_v4` request object with owner
 * address and serialized typed data. This is the bridge for injected wallets
 * and wallet-client libraries that expose an EIP-1193-style request function.
 *
 * @param input Unsigned order fields to sign.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @param owner Owner address used in the wallet request and order UID.
 * @param requestCallback Callback that executes the EIP-1193 request.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed order.
 * @throws SdkError for invalid input, wallet failure, timeout, or cancellation.
 */
export function signOrderWithEip1193(input: OrderInput, chainId: number, owner: string, requestCallback: Eip1193RequestCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through typed-data ECDSA and wraps it as EIP-1271.
 *
 * The SDK sends the EIP-712 envelope to the provided typed-data callback,
 * then converts the returned ECDSA signature into the CoW EIP-1271 payload.
 * Per-call options may attach cancellation and wallet timeout settings.
 *
 * @param input Unsigned order to sign.
 * @param chainId EVM chain id for the EIP-712 domain.
 * @param owner Smart-account owner address used in the generated order UID.
 * @param typedDataSigner Callback that signs the typed-data envelope.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed-order DTO.
 * @throws SdkError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderWithEip1271(input: OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Signs an order through a typed-data callback.
 *
 * The SDK builds the EIP-712 typed-data envelope, passes it to the callback,
 * normalizes the returned ECDSA signature, and returns the signed-order DTO
 * with the canonical order UID and digest.
 *
 * @param input Unsigned order fields to sign.
 * @param chainId EVM chain id used for the EIP-712 domain.
 * @param owner Owner address used in the generated order UID.
 * @param typedDataSigner Callback that signs the typed-data envelope.
 * @param options Optional cancellation, timeout, and wallet timeout settings.
 * @returns A versioned envelope containing the signed order.
 * @throws SdkError for invalid input, callback failure, timeout, or cancellation.
 */
export function signOrderWithTypedDataSigner(input: OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<SignedOrderDto>>;

/**
 * Returns the EVM chain ids supported by the SDK deployment registry.
 *
 * This is a pure helper and does not perform network I/O. The returned list is
 * suitable for runtime validation, UI selection, or capability checks before a
 * client is constructed.
 *
 * @returns A typed array of supported EVM chain ids.
 */
export function supportedChainIds(): Uint32Array;

/**
 * Validates an app-data document against the embedded schema set.
 *
 * Validation is local and deterministic. The result reports whether the
 * document conforms and includes validation details without uploading data.
 *
 * @param doc App-data document input to validate.
 * @returns A versioned envelope containing the validation result.
 * @throws SdkError when the input cannot be converted into a document.
 */
export function validateAppDataDoc(doc: AppDataDocInput): WasmEnvelope<ValidationResultDto>;

/**
 * Returns the version of the wasm package runtime.
 *
 * The value comes from the Rust package metadata used to build the wasm
 * artifact and can be included in diagnostics or compatibility checks.
 *
 * @returns The semantic version string for this wasm build.
 */
export function wasmVersion(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __cow_sdk_wasm_init: () => void;
    readonly __wbg_ipfsclient_free: (a: number, b: number) => void;
    readonly __wbg_orderbookclient_free: (a: number, b: number) => void;
    readonly __wbg_subgraphclient_free: (a: number, b: number) => void;
    readonly __wbg_tradingclient_free: (a: number, b: number) => void;
    readonly appDataDoc: (a: number, b: number) => void;
    readonly appDataHexToCid: (a: number, b: number, c: number) => void;
    readonly appDataInfo: (a: number, b: number) => void;
    readonly buildCancelOrderTx: (a: number, b: number) => void;
    readonly buildPresignTx: (a: number, b: number) => void;
    readonly cidToAppDataHex: (a: number, b: number, c: number) => void;
    readonly computeOrderUid: (a: number, b: number, c: number, d: number, e: number) => void;
    readonly deploymentAddresses: (a: number, b: number, c: number, d: number) => void;
    readonly domainSeparator: (a: number, b: number) => void;
    readonly eip1271SignaturePayload: (a: number, b: number, c: number, d: number) => void;
    readonly ipfsclient_fetchAppDataFromCid: (a: number, b: number, c: number, d: number) => number;
    readonly ipfsclient_fetchAppDataFromHex: (a: number, b: number, c: number, d: number) => number;
    readonly ipfsclient_new: (a: number, b: number) => void;
    readonly orderTypedData: (a: number, b: number, c: number) => void;
    readonly orderbookclient_cancelOrders: (a: number, b: number, c: number) => number;
    readonly orderbookclient_getNativePrice: (a: number, b: number, c: number, d: number) => number;
    readonly orderbookclient_getOrder: (a: number, b: number, c: number, d: number) => number;
    readonly orderbookclient_getOrders: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly orderbookclient_getOrdersByOwner: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly orderbookclient_getQuote: (a: number, b: number, c: number) => number;
    readonly orderbookclient_getTrades: (a: number, b: number, c: number) => number;
    readonly orderbookclient_new: (a: number, b: number) => void;
    readonly orderbookclient_sendOrder: (a: number, b: number, c: number) => number;
    readonly orderbookclient_sendOrderCreation: (a: number, b: number, c: number) => number;
    readonly signCancellationEthSignDigest: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly signCancellationWithEip1193: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly signCancellationWithTypedDataSigner: (a: number, b: number, c: number, d: number, e: number) => number;
    readonly signOrderEthSignDigest: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly signOrderWithCustomEip1271: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly signOrderWithEip1193: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly signOrderWithEip1271: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly signOrderWithTypedDataSigner: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly subgraphclient_getLastDaysVolume: (a: number, b: number, c: number) => number;
    readonly subgraphclient_getLastHoursVolume: (a: number, b: number, c: number) => number;
    readonly subgraphclient_getTotals: (a: number, b: number) => number;
    readonly subgraphclient_new: (a: number, b: number) => void;
    readonly subgraphclient_runQuery: (a: number, b: number, c: number) => number;
    readonly supportedChainIds: (a: number) => void;
    readonly tradingclient_buildSellNativeCurrencyTx: (a: number, b: number, c: bigint, d: number, e: number, f: number) => number;
    readonly tradingclient_getCowProtocolAllowance: (a: number, b: number, c: number, d: number) => number;
    readonly tradingclient_getQuote: (a: number, b: number, c: number) => number;
    readonly tradingclient_new: (a: number, b: number) => void;
    readonly tradingclient_postLimitOrder: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly tradingclient_postSwapOrder: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly tradingclient_postSwapOrderFromQuote: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly tradingclient_postSwapOrderWithEip1271: (a: number, b: number, c: number, d: number, e: number, f: number) => number;
    readonly validateAppDataDoc: (a: number, b: number) => void;
    readonly wasmVersion: (a: number) => void;
    readonly __wasm_bindgen_func_elem_11698: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_11706: (a: number, b: number, c: number, d: number) => void;
    readonly __wasm_bindgen_func_elem_1318: (a: number, b: number, c: number) => number;
    readonly __wasm_bindgen_func_elem_11619: (a: number, b: number) => void;
    readonly __wbindgen_export: (a: number, b: number) => number;
    readonly __wbindgen_export2: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_export3: (a: number) => void;
    readonly __wbindgen_export4: (a: number, b: number, c: number) => void;
    readonly __wbindgen_export5: (a: number, b: number) => void;
    readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
