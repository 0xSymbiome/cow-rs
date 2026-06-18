import * as wasm from "../../dist/raw/trading-bundler/cow_sdk_wasm.js";

export type * from "../../dist/raw/trading-bundler/cow_sdk_wasm.js";

// The bundler target instantiates on import and ships no wasm-bindgen `init`, so
// it emits no `InitInput` type. Declare it here, matching the web target's
// shape, so the single `trading` facade can type a uniform `initialize(module?)`
// across every target.
export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

// The bundler target instantiates the module as part of the host bundler's wasm
// import, so there is nothing to initialize at runtime. `initializeRaw` is a
// no-op here so the single `trading` facade can expose a uniform `initialize()`
// across targets; the `web` target's shim (`trading-web.ts`) wires the real
// wasm-bindgen initializer for hosts that must supply the module themselves
// (Cloudflare Workers, Deno, Vercel Edge, no-bundler browsers).
export const initializeRaw = async (_input?: { module_or_path?: unknown }): Promise<void> => {};
export const RawOrderBookClient = wasm.OrderBookClient;
export const RawTradingClient = wasm.TradingClient;

export const __cow_sdk_wasm_init = wasm.__cow_sdk_wasm_init;
export const appDataDoc = wasm.appDataDoc;
export const appDataHexToCid = wasm.appDataHexToCid;
export const appDataInfo = wasm.appDataInfo;
export const buildCancelOrderTx = wasm.buildCancelOrderTx;
export const buildPresignTx = wasm.buildPresignTx;
export const cidToAppDataHex = wasm.cidToAppDataHex;
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
export const validateAppDataDoc = wasm.validateAppDataDoc;
export const wasmVersion = wasm.wasmVersion;
