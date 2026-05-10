use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const CALLBACK_TYPES: &str = r#"
export type CowFetchMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";
export type Value = unknown;
export type SdkError = WasmError;

export interface SdkClientOptions {
  timeoutMs?: number;
  signal?: AbortSignal;
}

export interface WalletConfig {
  timeoutMs?: number;
}

export interface SigningOptions extends SdkClientOptions {
  walletConfig?: WalletConfig;
}

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
"#;
