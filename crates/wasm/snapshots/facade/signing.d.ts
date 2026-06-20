import * as raw from "./raw/signing.js";
import type { WasmEnvelope } from "./envelope.js";
import type { SigningOptions } from "./options.js";
import type { CustomEip1271Callback, DigestSignerCallback, Eip1193RequestCallback, TypedDataSignerCallback } from "./callbacks.js";
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
export type { InitInput } from "./raw/signing.js";
export declare function computeOrderUid(input: raw.OrderInput, chainId: number, owner: string): WasmEnvelope<raw.GeneratedOrderUidDto>;
export declare function decodeEthFlowLog(log: raw.EventLogInput): WasmEnvelope<raw.EthFlowEventDto>;
export declare function decodeSettlementLog(log: raw.EventLogInput): WasmEnvelope<raw.SettlementEventDto>;
export declare function deploymentAddresses(chainId: number, env?: string | null): WasmEnvelope<raw.DeploymentAddressesDto>;
export declare function domainSeparator(chainId: number): string;
export declare function eip1271SignaturePayload(input: raw.OrderInput, ecdsaSignature: string): WasmEnvelope<string>;
export declare function orderTypedData(input: raw.OrderInput, chainId: number): WasmEnvelope<raw.TypedDataEnvelopeDto>;
export declare function signOrderEthSignDigest(input: raw.OrderInput, chainId: number, owner: string, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrderDto>>;
export declare function signOrderWithCustomEip1271(input: raw.OrderInput, chainId: number, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrderDto>>;
export declare function signOrderWithEip1193(input: raw.OrderInput, chainId: number, owner: string, requestCallback: Eip1193RequestCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrderDto>>;
export declare function signOrderWithEip1271(input: raw.OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrderDto>>;
export declare function signOrderWithTypedDataSigner(input: raw.OrderInput, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrderDto>>;
export declare function supportedChainIds(): Uint32Array;
export declare function wasmVersion(): string;
export declare function wrappedNativeToken(chainId: number): WasmEnvelope<raw.WrappedNativeTokenDto>;
export type { CowEip1271SignRequest, DeploymentAddressesDto, Eip1193Request, EthFlowEventDto, EventLogInput, GeneratedOrderUidDto, OrderInput, OrderKindDto, SettlementEventDto, SignedOrderDto, TokenBalanceDto, TypedDataDomainDto, TypedDataEnvelopeDto, TypedDataFieldDto, WrappedNativeTokenDto } from "./raw/signing.js";
export type { CowEip1271SignCallback, CustomEip1271Callback, DigestSignerCallback, Eip1193RequestCallback, TypedDataSignerCallback } from "./callbacks.js";
export { CowError, isCowError, isRetryable, isUserRejection, normalizeError, retryAfterMs } from "./errors.js";
export type { CowErrorData, OrderBookErrorType, OrderBookRejectionCategory } from "./errors.js";
export { withRetry } from "./retry.js";
export type { RetryOptions } from "./retry.js";
export type { SchemaVersion, WasmEnvelope } from "./envelope.js";
export type { SdkClientOptions, SigningOptions, WalletConfig } from "./options.js";
