import * as wasm from "../../dist/raw/default-bundler/cow_sdk_js.js";

export type * from "../../dist/raw/default-bundler/cow_sdk_js.js";

// Bundler and nodejs targets instantiate on import (no wasm-bindgen `init`), so
// `InitInput` is declared here and `initializeRaw` is a no-op — the facade exposes
// one `initialize(module?)` across targets. compile-facade generates the web shim,
// swapping `initializeRaw` for the real `init` (Workers/Deno/edge/no-bundler hosts
// supply the module).
export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;
export const initializeRaw = async (_input?: { module_or_path?: unknown }): Promise<void> => {};

export const RawIpfsClient = wasm.IpfsClient;
export const RawOrderBookClient = wasm.OrderBookClient;
export const RawSubgraphClient = wasm.SubgraphClient;
export const RawTradingClient = wasm.TradingClient;

export const __cow_sdk_js_init = wasm.__cow_sdk_js_init;
export const appDataDoc = wasm.appDataDoc;
export const appDataHexToCid = wasm.appDataHexToCid;
export const appDataInfo = wasm.appDataInfo;
export const buildAppData = wasm.buildAppData;
export const buildCancelOrderTx = wasm.buildCancelOrderTx;
export const buildPresignTx = wasm.buildPresignTx;
export const buildTwapCreateTransaction = wasm.buildTwapCreateTransaction;
export const buildTwapRemoveTransaction = wasm.buildTwapRemoveTransaction;
export const cidToAppDataHex = wasm.cidToAppDataHex;
export const computeOrderUid = wasm.computeOrderUid;
export const decodeEthFlowLog = wasm.decodeEthFlowLog;
export const decodeSettlementLog = wasm.decodeSettlementLog;
export const deploymentAddresses = wasm.deploymentAddresses;
export const domainSeparator = wasm.domainSeparator;
export const eip1271SignaturePayload = wasm.eip1271SignaturePayload;
export const orderTypedData = wasm.orderTypedData;
export const signCancellationEthSignDigest = wasm.signCancellationEthSignDigest;
export const signCancellationWithTypedDataSigner = wasm.signCancellationWithTypedDataSigner;
export const signOrderEthSignDigest = wasm.signOrderEthSignDigest;
export const signOrderWithCustomEip1271 = wasm.signOrderWithCustomEip1271;
export const signOrderWithEip1271 = wasm.signOrderWithEip1271;
export const signOrderWithTypedDataSigner = wasm.signOrderWithTypedDataSigner;
export const supportedChainIds = wasm.supportedChainIds;
export const validateAppDataDoc = wasm.validateAppDataDoc;
export const wasmVersion = wasm.wasmVersion;
export const wrappedNativeToken = wasm.wrappedNativeToken;
