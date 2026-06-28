import * as raw from "./raw/signing.js";
import type { WasmEnvelope } from "./envelope.js";
import type { SigningOptions } from "./options.js";
import type { CustomEip1271Callback, DigestSignerCallback, TypedDataSignerCallback } from "./callbacks.js";
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
export declare function computeOrderUid(order: raw.OrderData, chainId: number, owner: string): WasmEnvelope<raw.GeneratedOrderUid>;
export declare function decodeEthFlowLog(log: raw.EventLog): WasmEnvelope<raw.EthFlowEvent>;
export declare function decodeSettlementLog(log: raw.EventLog): WasmEnvelope<raw.SettlementEvent>;
export declare function deploymentAddresses(chainId: number, env?: string | null): WasmEnvelope<raw.DeploymentAddresses>;
export declare function domainSeparator(chainId: number): WasmEnvelope<string>;
export declare function eip1271SignaturePayload(order: raw.OrderData, ecdsaSignature: string): WasmEnvelope<string>;
export type TypedDataMessage = unknown;
export declare function orderTypedData(order: raw.OrderData, chainId: number): WasmEnvelope<raw.TypedDataEnvelope<TypedDataMessage>>;
export declare function signOrderEthSignDigest(order: raw.OrderData, chainId: number, owner: string, digestSigner: DigestSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrder>>;
export declare function signOrderWithCustomEip1271(order: raw.OrderData, chainId: number, owner: string, customCallback: CustomEip1271Callback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrder>>;
export declare function signOrderWithEip1271(order: raw.OrderData, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrder>>;
export declare function signOrderWithTypedDataSigner(order: raw.OrderData, chainId: number, owner: string, typedDataSigner: TypedDataSignerCallback, options?: SigningOptions | null): Promise<WasmEnvelope<raw.SignedOrder>>;
export declare function supportedChainIds(): Uint32Array;
export declare function wasmVersion(): string;
export declare function wrappedNativeToken(chainId: number): WasmEnvelope<raw.WrappedNativeToken>;
export type { BuyTokenDestination, CowEip1271SignRequest, DeploymentAddresses, EthFlowEvent, EventLog, GeneratedOrderUid, OrderData, OrderKind, SellTokenSource, SettlementEvent, SignedOrder, TypedDataDomain, TypedDataEnvelope, TypedDataField, WrappedNativeToken } from "./raw/signing.js";
export type { CustomEip1271Callback, DigestSignerCallback, TypedDataSignerCallback } from "./callbacks.js";
export { CowError, isCowError, isRetryable, isUserRejection, normalizeError, retryAfterMs } from "./errors.js";
export type { CowErrorData, OrderBookErrorType, OrderBookRejectionCategory } from "./errors.js";
export { withRetry } from "./retry.js";
export type { RetryOptions } from "./retry.js";
export type { SchemaVersion, WasmEnvelope } from "./envelope.js";
export type { SdkClientOptions, SigningOptions, WalletConfig } from "./options.js";
