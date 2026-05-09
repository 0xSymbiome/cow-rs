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
 * Disposable callback registry handle.
 */
export class FetchCallbackHandle {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Disposes this callback registration. Calling this more than once is harmless.
     */
    dispose(): void;
    /**
     * Numeric callback id.
     */
    readonly id: number;
}

/**
 * Adapter that lets app-data IPFS reads flow through an HTTP transport.
 */
export class HttpToIpfsAdapter {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Fetches and parses an app-data document by CID.
     */
    fetchAppDataFromCid(cid: string, ipfs_uri?: string | null): Promise<any>;
    /**
     * Fetches and parses an app-data document by app-data hash.
     */
    fetchAppDataFromHex(app_data_hex: string, ipfs_uri?: string | null): Promise<any>;
    /**
     * Creates an adapter from an existing fetch-callback handle id.
     */
    static fromHandle(fetch_callback_id: number, timeout_ms?: number | null): HttpToIpfsAdapter;
    /**
     * Creates an adapter that owns a registered fetch callback.
     */
    constructor(fetch_callback: Function, timeout_ms?: number | null);
}

/**
 * IPFS client backed by the browser fetch transport.
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
     * Creates an IPFS client with the default browser fetch transport.
     */
    constructor(ipfs_uri?: string | null, timeout_ms?: number | null);
}

/**
 * IPFS client backed by a JavaScript fetch callback.
 */
export class IpfsClientWithFetch {
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
     * Creates an IPFS client from an existing fetch-callback handle id.
     */
    static fromHandle(ipfs_uri: string | null | undefined, timeout_ms: number | null | undefined, fetch_callback_id: number): IpfsClientWithFetch;
    /**
     * Creates an IPFS client that owns a registered fetch callback.
     */
    constructor(ipfs_uri: string | null | undefined, timeout_ms: number | null | undefined, fetch_callback: Function);
}

/**
 * Orderbook client backed by the browser fetch transport.
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
     * Creates an orderbook client for a chain and environment.
     */
    constructor(chain_id: number, env?: string | null);
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
 * Orderbook client backed by a JavaScript fetch callback.
 */
export class OrderBookClientWithFetch {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Cancels orders through a signed cancellation payload.
     */
    cancelOrders(signed: SignedCancellationsInput): Promise<any>;
    /**
     * Creates an orderbook client from an existing fetch-callback handle id.
     */
    static fromHandle(chain_id: number, env: string | null | undefined, fetch_callback_id: number): OrderBookClientWithFetch;
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
     * Creates an orderbook client that owns a registered fetch callback.
     */
    constructor(chain_id: number, env: string | null | undefined, fetch_callback: Function);
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
 * Subgraph client backed by the browser fetch transport.
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
     * Creates a subgraph client for a chain and Graph API key.
     */
    constructor(chain_id: number, api_key: string);
    /**
     * Runs a raw GraphQL query.
     */
    runQuery(request: SubgraphQueryInput): Promise<any>;
}

/**
 * Subgraph client backed by a JavaScript fetch callback.
 */
export class SubgraphClientWithFetch {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Creates a subgraph client from an existing fetch-callback handle id.
     */
    static fromHandle(chain_id: number, api_key: string, fetch_callback_id: number): SubgraphClientWithFetch;
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
     * Creates a subgraph client that owns a registered fetch callback.
     */
    constructor(chain_id: number, api_key: string, fetch_callback: Function);
    /**
     * Runs a raw GraphQL query.
     */
    runQuery(request: SubgraphQueryInput): Promise<any>;
}

/**
 * Trading facade backed by the browser fetch transport.
 */
export class TradingClient {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Fetches a quote without submitting an order.
     */
    getQuote(params: SwapParametersInput): Promise<any>;
    /**
     * Creates a trading client for a chain, environment, and app code.
     */
    constructor(chain_id: number, env: string | null | undefined, app_code: string);
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
 * Trading facade backed by a JavaScript fetch callback.
 */
export class TradingClientWithFetch {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Creates a trading client from an existing fetch-callback handle id.
     */
    static fromHandle(chain_id: number, env: string | null | undefined, app_code: string, fetch_callback_id: number): TradingClientWithFetch;
    /**
     * Fetches a quote without submitting an order.
     */
    getQuote(params: SwapParametersInput): Promise<any>;
    /**
     * Creates a trading client that owns a registered fetch callback.
     */
    constructor(chain_id: number, env: string | null | undefined, app_code: string, fetch_callback: Function);
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
 * Fetches and parses an app-data document by CID.
 */
export function fetchAppDataFromCid(cid: string, ipfs_uri?: string | null, timeout_ms?: number | null): Promise<any>;

/**
 * Fetches and parses an app-data document by app-data hash.
 */
export function fetchAppDataFromHex(app_data_hex: string, ipfs_uri?: string | null, timeout_ms?: number | null): Promise<any>;

/**
 * Builds signer-facing order typed data.
 */
export function orderTypedData(input: OrderInput, chain_id: number): any;

/**
 * Registers a JS fetch callback and returns a disposable handle.
 */
export function registerFetchCallback(callback: Function): FetchCallbackHandle;

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

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly appDataDoc: (a: any) => [number, number, number];
    readonly appDataHexToCid: (a: number, b: number) => [number, number, number, number];
    readonly appDataInfo: (a: any) => [number, number, number];
    readonly cidToAppDataHex: (a: number, b: number) => [number, number, number, number];
    readonly computeOrderUid: (a: any, b: number, c: number, d: number) => [number, number, number];
    readonly deploymentAddresses: (a: number, b: number, c: number) => [number, number, number];
    readonly domainSeparator: (a: number) => [number, number, number, number];
    readonly orderTypedData: (a: any, b: number) => [number, number, number];
    readonly supportedChainIds: () => [number, number];
    readonly validateAppDataDoc: (a: any) => [number, number, number];
    readonly wasmVersion: () => [number, number];
    readonly __wbg_fetchcallbackhandle_free: (a: number, b: number) => void;
    readonly __wbg_orderbookclient_free: (a: number, b: number) => void;
    readonly __wbg_orderbookclientwithfetch_free: (a: number, b: number) => void;
    readonly __wbg_subgraphclient_free: (a: number, b: number) => void;
    readonly __wbg_subgraphclientwithfetch_free: (a: number, b: number) => void;
    readonly fetchcallbackhandle_dispose: (a: number) => void;
    readonly fetchcallbackhandle_id: (a: number) => number;
    readonly orderbookclient_cancelOrders: (a: number, b: any) => any;
    readonly orderbookclient_getNativePrice: (a: number, b: number, c: number) => any;
    readonly orderbookclient_getOrder: (a: number, b: number, c: number) => any;
    readonly orderbookclient_getOrdersByOwner: (a: number, b: number, c: number) => any;
    readonly orderbookclient_getQuote: (a: number, b: any) => any;
    readonly orderbookclient_getTrades: (a: number, b: number, c: number) => any;
    readonly orderbookclient_new: (a: number, b: number, c: number) => [number, number, number];
    readonly orderbookclient_sendOrder: (a: number, b: any) => any;
    readonly orderbookclient_sendOrderCreation: (a: number, b: any) => any;
    readonly orderbookclientwithfetch_cancelOrders: (a: number, b: any) => any;
    readonly orderbookclientwithfetch_fromHandle: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly orderbookclientwithfetch_getNativePrice: (a: number, b: number, c: number) => any;
    readonly orderbookclientwithfetch_getOrder: (a: number, b: number, c: number) => any;
    readonly orderbookclientwithfetch_getOrdersByOwner: (a: number, b: number, c: number) => any;
    readonly orderbookclientwithfetch_getQuote: (a: number, b: any) => any;
    readonly orderbookclientwithfetch_getTrades: (a: number, b: number, c: number) => any;
    readonly orderbookclientwithfetch_new: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly orderbookclientwithfetch_sendOrder: (a: number, b: any) => any;
    readonly orderbookclientwithfetch_sendOrderCreation: (a: number, b: any) => any;
    readonly registerFetchCallback: (a: any) => [number, number, number];
    readonly subgraphclient_getLastDaysVolume: (a: number, b: number) => any;
    readonly subgraphclient_getLastHoursVolume: (a: number, b: number) => any;
    readonly subgraphclient_getTotals: (a: number) => any;
    readonly subgraphclient_new: (a: number, b: number, c: number) => [number, number, number];
    readonly subgraphclient_runQuery: (a: number, b: any) => any;
    readonly subgraphclientwithfetch_fromHandle: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly subgraphclientwithfetch_getLastDaysVolume: (a: number, b: number) => any;
    readonly subgraphclientwithfetch_getLastHoursVolume: (a: number, b: number) => any;
    readonly subgraphclientwithfetch_getTotals: (a: number) => any;
    readonly subgraphclientwithfetch_new: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly subgraphclientwithfetch_runQuery: (a: number, b: any) => any;
    readonly __wbg_httptoipfsadapter_free: (a: number, b: number) => void;
    readonly __wbg_ipfsclient_free: (a: number, b: number) => void;
    readonly __wbg_ipfsclientwithfetch_free: (a: number, b: number) => void;
    readonly fetchAppDataFromCid: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly fetchAppDataFromHex: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly httptoipfsadapter_fetchAppDataFromCid: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly httptoipfsadapter_fetchAppDataFromHex: (a: number, b: number, c: number, d: number, e: number) => any;
    readonly httptoipfsadapter_fromHandle: (a: number, b: number) => [number, number, number];
    readonly httptoipfsadapter_new: (a: any, b: number) => [number, number, number];
    readonly ipfsclient_fetchAppDataFromCid: (a: number, b: number, c: number) => any;
    readonly ipfsclient_fetchAppDataFromHex: (a: number, b: number, c: number) => any;
    readonly ipfsclient_new: (a: number, b: number, c: number) => [number, number, number];
    readonly ipfsclientwithfetch_fetchAppDataFromCid: (a: number, b: number, c: number) => any;
    readonly ipfsclientwithfetch_fetchAppDataFromHex: (a: number, b: number, c: number) => any;
    readonly ipfsclientwithfetch_fromHandle: (a: number, b: number, c: number, d: number) => [number, number, number];
    readonly ipfsclientwithfetch_new: (a: number, b: number, c: number, d: any) => [number, number, number];
    readonly __cow_sdk_wasm_init: () => void;
    readonly __wbg_tradingclient_free: (a: number, b: number) => void;
    readonly __wbg_tradingclientwithfetch_free: (a: number, b: number) => void;
    readonly eip1271SignaturePayload: (a: any, b: number, c: number) => [number, number, number, number];
    readonly signCancellationEthSignDigest: (a: number, b: number, c: number, d: any) => any;
    readonly signCancellationWithEip1193: (a: number, b: number, c: number, d: number, e: number, f: any) => any;
    readonly signCancellationWithTypedDataSigner: (a: number, b: number, c: number, d: any) => any;
    readonly signOrderEthSignDigest: (a: any, b: number, c: number, d: number, e: any) => any;
    readonly signOrderWithCustomEip1271: (a: any, b: number, c: number, d: number, e: any) => any;
    readonly signOrderWithEip1193: (a: any, b: number, c: number, d: number, e: any) => any;
    readonly signOrderWithEip1271: (a: any, b: number, c: number, d: number, e: any) => any;
    readonly signOrderWithTypedDataSigner: (a: any, b: number, c: number, d: number, e: any) => any;
    readonly tradingclient_getQuote: (a: number, b: any) => any;
    readonly tradingclient_new: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly tradingclient_postSwapOrder: (a: number, b: any, c: number, d: number, e: any) => any;
    readonly tradingclient_postSwapOrderWithEip1271: (a: number, b: any, c: number, d: number, e: any) => any;
    readonly tradingclientwithfetch_fromHandle: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number, number];
    readonly tradingclientwithfetch_getQuote: (a: number, b: any) => any;
    readonly tradingclientwithfetch_new: (a: number, b: number, c: number, d: number, e: number, f: any) => [number, number, number];
    readonly tradingclientwithfetch_postSwapOrder: (a: number, b: any, c: number, d: number, e: any) => any;
    readonly tradingclientwithfetch_postSwapOrderWithEip1271: (a: number, b: any, c: number, d: number, e: any) => any;
    readonly wasm_bindgen__convert__closures_____invoke__hd9f032c1a2f8138e: (a: number, b: number, c: any) => [number, number];
    readonly wasm_bindgen__convert__closures_____invoke__h16b7440c88f0269d: (a: number, b: number, c: any, d: any) => void;
    readonly wasm_bindgen__convert__closures_____invoke__hf3184c2633042a72: (a: number, b: number) => void;
    readonly wasm_bindgen__convert__closures_____invoke__h7af8680472e534d6: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_destroy_closure: (a: number, b: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
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
