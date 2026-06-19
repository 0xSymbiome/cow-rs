import * as raw from "./raw/signing.js";
import { callAsync, callSync } from "./internal.js";
import type { WasmEnvelope } from "./envelope.js";
import type { SigningOptions } from "./options.js";
import type {
  CustomEip1271Callback,
  DigestSignerCallback,
  Eip1193RequestCallback,
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
  input: raw.OrderInput,
  chainId: number,
  owner: string
): WasmEnvelope<raw.GeneratedOrderUidDto> {
  return callSync(() => raw.computeOrderUid(input, chainId, owner));
}

export function decodeEthFlowLog(log: raw.EventLogInput): WasmEnvelope<raw.EthFlowEventDto> {
  return callSync(() => raw.decodeEthFlowLog(log));
}

export function decodeSettlementLog(log: raw.EventLogInput): WasmEnvelope<raw.SettlementEventDto> {
  return callSync(() => raw.decodeSettlementLog(log));
}

export function deploymentAddresses(
  chainId: number,
  env?: string | null
): WasmEnvelope<raw.DeploymentAddressesDto> {
  return callSync(() => raw.deploymentAddresses(chainId, env ?? null));
}

export function domainSeparator(chainId: number): string {
  return callSync(() => raw.domainSeparator(chainId));
}

export function eip1271SignaturePayload(
  input: raw.OrderInput,
  ecdsaSignature: string
): WasmEnvelope<string> {
  return callSync(() => raw.eip1271SignaturePayload(input, ecdsaSignature));
}

export function orderTypedData(
  input: raw.OrderInput,
  chainId: number
): WasmEnvelope<raw.TypedDataEnvelopeDto> {
  return callSync(() => raw.orderTypedData(input, chainId));
}

export function signOrderEthSignDigest(
  input: raw.OrderInput,
  chainId: number,
  owner: string,
  digestSigner: DigestSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrderDto>> {
  return callAsync(() =>
    raw.signOrderEthSignDigest(input, chainId, owner, digestSigner, options ?? null)
  );
}

export function signOrderWithCustomEip1271(
  input: raw.OrderInput,
  chainId: number,
  owner: string,
  customCallback: CustomEip1271Callback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrderDto>> {
  return callAsync(() =>
    raw.signOrderWithCustomEip1271(input, chainId, owner, customCallback, options ?? null)
  );
}

export function signOrderWithEip1193(
  input: raw.OrderInput,
  chainId: number,
  owner: string,
  requestCallback: Eip1193RequestCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrderDto>> {
  return callAsync(() =>
    raw.signOrderWithEip1193(input, chainId, owner, requestCallback, options ?? null)
  );
}

export function signOrderWithEip1271(
  input: raw.OrderInput,
  chainId: number,
  owner: string,
  typedDataSigner: TypedDataSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrderDto>> {
  return callAsync(() =>
    raw.signOrderWithEip1271(input, chainId, owner, typedDataSigner, options ?? null)
  );
}

export function signOrderWithTypedDataSigner(
  input: raw.OrderInput,
  chainId: number,
  owner: string,
  typedDataSigner: TypedDataSignerCallback,
  options?: SigningOptions | null
): Promise<WasmEnvelope<raw.SignedOrderDto>> {
  return callAsync(() =>
    raw.signOrderWithTypedDataSigner(input, chainId, owner, typedDataSigner, options ?? null)
  );
}

export function supportedChainIds(): Uint32Array {
  return callSync(() => raw.supportedChainIds());
}

export function wasmVersion(): string {
  return callSync(() => raw.wasmVersion());
}

export type {
  CowEip1271SignRequest,
  DeploymentAddressesDto,
  Eip1193Request,
  EthFlowEventDto,
  EventLogInput,
  GeneratedOrderUidDto,
  OrderInput,
  OrderKindDto,
  OrderTraderParametersInput,
  PaginationOptions,
  SettlementEventDto,
  SignedOrderDto,
  TokenBalanceDto,
  TradesQueryInput,
  TransactionRequestDto,
  TypedDataDomainDto,
  TypedDataEnvelopeDto,
  TypedDataFieldDto
} from "./raw/signing.js";
export type {
  CowEip1271SignCallback,
  CustomEip1271Callback,
  DigestSignerCallback,
  Eip1193RequestCallback,
  TypedDataSignerCallback
} from "./callbacks.js";
export type { OrderBookRejectionCategory, CowError } from "./errors.js";
export type { SchemaVersion, WasmEnvelope } from "./envelope.js";
export type { SdkClientOptions, SigningOptions, WalletConfig } from "./options.js";
