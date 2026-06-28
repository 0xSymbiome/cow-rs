import * as raw from "./raw/signing.js";
import { callAsync, callSync } from "./internal.js";
import type { WasmEnvelope } from "./envelope.js";
import type { SigningOptions } from "./options.js";
import type {
  CustomEip1271Callback,
  DigestSignerCallback,
  TypedDataSignerCallback
} from "./callbacks.js";

let initialized: Promise<void> | undefined;

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
export async function initialize(module?: WebAssembly.Module | raw.InitInput): Promise<void> {
  if (!initialized) {
    initialized = raw.initializeRaw({ module_or_path: module }).then(() => undefined);
  }
  await initialized;
}

export default initialize;

export type { InitInput } from "./raw/signing.js";

export function computeOrderUid(
  order: raw.OrderData,
  chainId: number,
  owner: string
): WasmEnvelope<raw.GeneratedOrderUid> {
  return callSync(() => raw.computeOrderUid(order, chainId, owner));
}

export function decodeEthFlowLog(log: raw.EventLog): WasmEnvelope<raw.EthFlowEvent> {
  return callSync(() => raw.decodeEthFlowLog(log));
}

export function decodeSettlementLog(log: raw.EventLog): WasmEnvelope<raw.SettlementEvent> {
  return callSync(() => raw.decodeSettlementLog(log));
}

export function deploymentAddresses(
  chainId: number,
  env?: string | null
): WasmEnvelope<raw.DeploymentAddresses> {
  return callSync(() => raw.deploymentAddresses(chainId, env ?? null));
}

export function domainSeparator(chainId: number): WasmEnvelope<string> {
  return callSync(() => raw.domainSeparator(chainId));
}

export function eip1271SignaturePayload(
  order: raw.OrderData,
  ecdsaSignature: string
): WasmEnvelope<string> {
  return callSync(() => raw.eip1271SignaturePayload(order, ecdsaSignature));
}

export type TypedDataMessage = unknown;

export function orderTypedData(
  order: raw.OrderData,
  chainId: number
): WasmEnvelope<raw.TypedDataEnvelope<TypedDataMessage>> {
  return callSync(() => raw.orderTypedData(order, chainId));
}

export function signOrderEthSignDigest(
  order: raw.OrderData,
  chainId: number,
  owner: string,
  digestSigner: DigestSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrder>> {
  return callAsync(() =>
    raw.signOrderEthSignDigest(order, chainId, owner, digestSigner, options ?? null)
  );
}

export function signOrderWithCustomEip1271(
  order: raw.OrderData,
  chainId: number,
  owner: string,
  customCallback: CustomEip1271Callback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrder>> {
  return callAsync(() =>
    raw.signOrderWithCustomEip1271(order, chainId, owner, customCallback, options ?? null)
  );
}

export function signOrderWithEip1271(
  order: raw.OrderData,
  chainId: number,
  owner: string,
  typedDataSigner: TypedDataSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrder>> {
  return callAsync(() =>
    raw.signOrderWithEip1271(order, chainId, owner, typedDataSigner, options ?? null)
  );
}

export function signOrderWithTypedDataSigner(
  order: raw.OrderData,
  chainId: number,
  owner: string,
  typedDataSigner: TypedDataSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrder>> {
  return callAsync(() =>
    raw.signOrderWithTypedDataSigner(order, chainId, owner, typedDataSigner, options ?? null)
  );
}

export function supportedChainIds(): Uint32Array {
  return callSync(() => raw.supportedChainIds());
}

export function wasmVersion(): string {
  return callSync(() => raw.wasmVersion());
}

export function wrappedNativeToken(chainId: number): WasmEnvelope<raw.WrappedNativeToken> {
  return callSync(() => raw.wrappedNativeToken(chainId));
}

export type {
  BuyTokenDestination,
  CowEip1271SignRequest,
  DeploymentAddresses,
  EthFlowEvent,
  EventLog,
  GeneratedOrderUid,
  OrderData,
  OrderKind,
  SellTokenSource,
  SettlementEvent,
  SignedOrder,
  TypedDataDomain,
  TypedDataEnvelope,
  TypedDataField,
  WrappedNativeToken
} from "./raw/signing.js";
export type {
  CustomEip1271Callback,
  DigestSignerCallback,
  TypedDataSignerCallback
} from "./callbacks.js";
export { CowError, isCowError, isRetryable, isUserRejection, normalizeError, retryAfterMs } from "./errors.js";
export type { CowErrorData, OrderBookErrorType, OrderBookRejectionCategory } from "./errors.js";
export { withRetry } from "./retry.js";
export type { RetryOptions } from "./retry.js";
export type { SchemaVersion, WasmEnvelope } from "./envelope.js";
export type { SdkClientOptions, SigningOptions, WalletConfig } from "./options.js";
