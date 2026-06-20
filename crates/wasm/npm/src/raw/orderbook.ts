import * as wasm from "../../dist/raw/orderbook-bundler/cow_sdk_wasm.js";

export type * from "../../dist/raw/orderbook-bundler/cow_sdk_wasm.js";

// Bundler and nodejs targets instantiate on import (no wasm-bindgen `init`), so
// `InitInput` is declared here and `initializeRaw` is a no-op — the facade exposes
// one `initialize(module?)` across targets. compile-facade generates the web shim,
// swapping `initializeRaw` for the real `init` (Workers/Deno/edge/no-bundler hosts
// supply the module).
export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;
export const initializeRaw = async (_input?: { module_or_path?: unknown }): Promise<void> => {};

export const RawOrderBookClient = wasm.OrderBookClient;

export const __cow_sdk_wasm_init = wasm.__cow_sdk_wasm_init;
export const buildCancelOrderTx = wasm.buildCancelOrderTx;
export const buildPresignTx = wasm.buildPresignTx;
export const computeOrderUid = wasm.computeOrderUid;
export const decodeEthFlowLog = wasm.decodeEthFlowLog;
export const decodeSettlementLog = wasm.decodeSettlementLog;
export const deploymentAddresses = wasm.deploymentAddresses;
export const domainSeparator = wasm.domainSeparator;
export const eip1271SignaturePayload = wasm.eip1271SignaturePayload;
export const orderTypedData = wasm.orderTypedData;
export const signCancellationEthSignDigest = wasm.signCancellationEthSignDigest;
export const signCancellationWithEip1193 = wasm.signCancellationWithEip1193;
export const signCancellationWithTypedDataSigner = wasm.signCancellationWithTypedDataSigner;
export const signOrderEthSignDigest = wasm.signOrderEthSignDigest;
export const signOrderWithCustomEip1271 = wasm.signOrderWithCustomEip1271;
export const signOrderWithEip1193 = wasm.signOrderWithEip1193;
export const signOrderWithEip1271 = wasm.signOrderWithEip1271;
export const signOrderWithTypedDataSigner = wasm.signOrderWithTypedDataSigner;
export const supportedChainIds = wasm.supportedChainIds;
export const wasmVersion = wasm.wasmVersion;
export const wrappedNativeToken = wasm.wrappedNativeToken;
