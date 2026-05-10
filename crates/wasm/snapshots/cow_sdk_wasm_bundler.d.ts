/// <reference lib="esnext.disposable" />
/* tslint:disable */
/* eslint-disable */

export type CowFetchMethod = "GET" | "POST" | "PUT" | "DELETE";
export type Value = unknown;

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
    headers: Record<string, string>;
    body: string;
}

export type CowFetchCallback = (
request: CowFetchRequest,
) => Promise<CowFetchResponse> | CowFetchResponse;

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

export type HttpTransportConfig =
| { kind: "fetch"; fetch?: typeof globalThis.fetch }
| { kind: "callback"; callback: CowFetchCallback };

export interface OrderBookClientConfig {
    chainId: number;
    env?: string | null;
    transport: HttpTransportConfig;
    timeoutMs?: number | null;
}

export interface SubgraphClientConfig {
    chainId: number;
    apiKey: string;
    transport: HttpTransportConfig;
    timeoutMs?: number | null;
}

export interface TradingClientConfig {
    chainId: number;
    env?: string | null;
    appCode: string;
    transport: HttpTransportConfig;
    timeoutMs?: number | null;
}

export interface IpfsClientConfig {
    ipfsUri?: string | null;
    transport: HttpTransportConfig;
    timeoutMs?: number | null;
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
     * Schema version.
     */
    schemaVersion: SchemaVersion;
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
     * Schema version.
     */
    schemaVersion: SchemaVersion;
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
     * Schema version.
     */
    schemaVersion: SchemaVersion;
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
     * Schema version.
     */
    schemaVersion: SchemaVersion;
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
 * Generated order UID output.
 */
export interface GeneratedOrderUidDto {
    /**
     * Schema version.
     */
    schemaVersion: SchemaVersion;
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
export type WasmError = { kind: "invalidInput"; message: string; field?: string } | { kind: "unknownEnumValue"; field: string; value: string } | { kind: "unsupportedChain"; chain_id: number } | { kind: "walletRequest"; method: string; code?: number; message: string; data?: Value } | { kind: "transport"; class: string; message: string; status?: number; headers?: [string, string][]; body?: string } | { kind: "orderbook"; code?: string; message: string } | { kind: "subgraph"; message: string } | { kind: "signing"; message: string } | { kind: "appData"; class?: string; message: string } | { kind: "cancelled" } | { kind: "internal"; message: string };

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
 * Signed order DTO returned by wallet callback exports.
 */
export interface SignedOrderDto {
    /**
     * Schema version.
     */
    schemaVersion: SchemaVersion;
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
 * Transparent JSON input for orderbook order creations.
 */
export type OrderCreationInput = Value;

/**
 * Transparent JSON input for orderbook quote requests.
 */
export type OrderQuoteRequestInput = Value;

/**
 * Transparent JSON input for subgraph raw queries.
 */
export type SubgraphQueryInput = Value;

/**
 * Transparent JSON input for trading swap parameters.
 */
export type SwapParametersInput = Value;

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
     * Schema version.
     */
    schemaVersion: SchemaVersion;
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
 * Version tag carried by wasm output envelopes.
 */
export type SchemaVersion = "v1";

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
 * IPFS client backed by an explicitly configured HTTP transport.
 */
export class IpfsClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Fetches and parses an app-data document by CID.
     */
    fetchAppDataFromCid(cid: string): Promise<any>;
    /**
     * Fetches and parses an app-data document by app-data hash.
     */
    fetchAppDataFromHex(app_data_hex: string): Promise<any>;
    /**
     * Creates an IPFS client from a single config object.
     */
    constructor(config: IpfsClientConfig);
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
    cancelOrders(signed: SignedCancellationsInput): Promise<any>;
    /**
     * Fetches a token's native price.
     */
    getNativePrice(token: string): Promise<any>;
    /**
     * Fetches an order by UID.
     */
    getOrder(order_uid: string): Promise<any>;
    /**
     * Fetches orders owned by an address.
     */
    getOrdersByOwner(owner: string): Promise<any>;
    /**
     * Fetches a quote.
     */
    getQuote(request: OrderQuoteRequestInput): Promise<any>;
    /**
     * Fetches trades for an order UID.
     */
    getTrades(order_uid: string): Promise<any>;
    /**
     * Creates an orderbook client from a single config object.
     */
    constructor(config: OrderBookClientConfig);
    /**
     * Submits a signed order.
     */
    sendOrder(signed: SignedOrderDto): Promise<string>;
    /**
     * Submits a raw order-creation payload.
     */
    sendOrderCreation(input: OrderCreationInput): Promise<string>;
}

/**
 * Subgraph client backed by an explicitly configured HTTP transport.
 */
export class SubgraphClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Fetches daily volume rows.
     */
    getLastDaysVolume(days: number): Promise<any>;
    /**
     * Fetches hourly volume rows.
     */
    getLastHoursVolume(hours: number): Promise<any>;
    /**
     * Fetches aggregate totals.
     */
    getTotals(): Promise<any>;
    /**
     * Creates a subgraph client from a single config object.
     */
    constructor(config: SubgraphClientConfig);
    /**
     * Runs a raw GraphQL query.
     */
    runQuery(request: SubgraphQueryInput): Promise<any>;
}

/**
 * Trading facade backed by an explicitly configured HTTP transport.
 */
export class TradingClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Fetches a quote without submitting an order.
     */
    getQuote(params: SwapParametersInput): Promise<any>;
    /**
     * Creates a trading client from a single config object.
     */
    constructor(config: TradingClientConfig);
    /**
     * Quotes, signs, and posts a swap order through a typed-data callback.
     */
    postSwapOrder(params: SwapParametersInput, owner: string, signer_callback: Function): Promise<any>;
    /**
     * Quotes and posts a swap order with a custom EIP-1271 signature callback.
     */
    postSwapOrderWithEip1271(params: SwapParametersInput, owner: string, custom_callback: Function): Promise<any>;
}

/**
 * Initializes the wasm crate's panic hook once.
 */
export function __cow_sdk_wasm_init(): void;

/**
 * Builds an app-data document without hashing it.
 */
export function appDataDoc(doc: AppDataDocInput): any;

/**
 * Converts an app-data hash to an IPFS CID.
 */
export function appDataHexToCid(app_data_hex: string): string;

/**
 * Returns deterministic app-data content, hash, and CID.
 */
export function appDataInfo(doc: AppDataDocInput): any;

/**
 * Converts an IPFS CID to an app-data hash.
 */
export function cidToAppDataHex(cid: string): string;

/**
 * Computes the compact order UID and digest.
 */
export function computeOrderUid(input: OrderInput, chain_id: number, owner: string): any;

/**
 * Returns canonical deployment addresses for a chain and environment.
 */
export function deploymentAddresses(chain_id: number, env?: string | null): any;

/**
 * Computes the EIP-712 domain separator for a supported chain.
 */
export function domainSeparator(chain_id: number): string;

/**
 * Encodes a CoW EIP-1271 payload from an ECDSA signature.
 */
export function eip1271SignaturePayload(input: OrderInput, ecdsa_signature: string): string;

/**
 * Builds signer-facing order typed data.
 */
export function orderTypedData(input: OrderInput, chain_id: number): any;

/**
 * Signs a cancellation digest through an explicit `eth_sign` callback.
 */
export function signCancellationEthSignDigest(order_uids: string[], chain_id: number, digest_signer: Function): Promise<any>;

/**
 * Signs cancellation typed data through an EIP-1193 callback.
 */
export function signCancellationWithEip1193(order_uids: string[], chain_id: number, owner: string, request_callback: Function): Promise<any>;

/**
 * Signs cancellation typed data through a typed-data callback.
 */
export function signCancellationWithTypedDataSigner(order_uids: string[], chain_id: number, typed_data_signer: Function): Promise<any>;

/**
 * Signs an order digest through an explicit `eth_sign` callback.
 */
export function signOrderEthSignDigest(input: OrderInput, chain_id: number, owner: string, digest_signer: Function): Promise<any>;

/**
 * Signs an order through a custom EIP-1271 callback.
 */
export function signOrderWithCustomEip1271(input: OrderInput, chain_id: number, owner: string, custom_callback: Function): Promise<any>;

/**
 * Signs an order through an EIP-1193 request callback.
 */
export function signOrderWithEip1193(input: OrderInput, chain_id: number, owner: string, request_callback: Function): Promise<any>;

/**
 * Signs an order through typed-data ECDSA and wraps it as EIP-1271.
 */
export function signOrderWithEip1271(input: OrderInput, chain_id: number, owner: string, typed_data_signer: Function): Promise<any>;

/**
 * Signs an order through a typed-data callback.
 */
export function signOrderWithTypedDataSigner(input: OrderInput, chain_id: number, owner: string, typed_data_signer: Function): Promise<any>;

/**
 * Returns supported EVM chain ids.
 */
export function supportedChainIds(): Uint32Array;

/**
 * Validates an app-data document against the embedded schemas.
 */
export function validateAppDataDoc(doc: AppDataDocInput): any;

/**
 * Returns the wasm crate version.
 */
export function wasmVersion(): string;
