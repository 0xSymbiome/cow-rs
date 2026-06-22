use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const COMMON_TYPES: &str = r#"
export type Value = unknown;
export type CowError = WasmError;

export interface SdkClientOptions {
  timeoutMs?: number;
  signal?: AbortSignal;
}
"#;

#[cfg(feature = "signing")]
#[wasm_bindgen(typescript_custom_section)]
const SIGNING_TYPES: &str = r#"
export interface WalletConfig {
  timeoutMs?: number;
}

export interface SigningOptions extends SdkClientOptions {
  walletConfig?: WalletConfig;
}

export type TypedDataSignerCallback = (
  envelope: TypedDataEnvelopeDto,
) => Promise<string> | string;

export type DigestSignerCallback = (
  digest: string,
) => Promise<string> | string;

export type CustomEip1271Callback = (
  request: CowEip1271SignRequest,
) => Promise<string> | string;
"#;

#[cfg(any(
    feature = "orderbook",
    feature = "subgraph",
    feature = "ipfs",
    feature = "trading"
))]
#[wasm_bindgen(typescript_custom_section)]
const HTTP_TYPES: &str = r#"
export type CowFetchMethod = "GET" | "POST" | "PUT" | "DELETE" | "PATCH";

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

export type HttpTransportConfig =
  | { kind: "callback"; callback: CowFetchCallback };
"#;

#[cfg(feature = "trading")]
#[wasm_bindgen(typescript_custom_section)]
const TRADING_CALLBACK_TYPES: &str = r#"
export type ContractReadCallback = (
  request: ContractCallDto,
) => Promise<string> | string;
"#;

#[cfg(feature = "orderbook")]
#[wasm_bindgen(typescript_custom_section)]
const ORDERBOOK_CONFIG_TYPES: &str = r#"
export interface OrderBookClientConfig {
  chainId: number;
  env?: string | null;
  apiKey?: string | null;
  transport?: HttpTransportConfig;
  transportPolicy?: TransportPolicyConfig | null;
  timeoutMs?: number | null;
}
"#;

#[cfg(feature = "subgraph")]
#[wasm_bindgen(typescript_custom_section)]
const SUBGRAPH_CONFIG_TYPES: &str = r#"
export interface SubgraphClientConfig {
  chainId: number;
  apiKey: string;
  transport?: HttpTransportConfig;
  transportPolicy?: TransportPolicyConfig | null;
  timeoutMs?: number | null;
}
"#;

#[cfg(feature = "trading")]
#[wasm_bindgen(typescript_custom_section)]
const TRADING_CONFIG_TYPES: &str = r#"
export interface TradingClientConfig {
  chainId: number;
  env?: string | null;
  appCode: string;
  apiKey?: string | null;
  transport?: HttpTransportConfig;
  transportPolicy?: TransportPolicyConfig | null;
  timeoutMs?: number | null;
}
"#;

#[cfg(feature = "ipfs")]
#[wasm_bindgen(typescript_custom_section)]
const IPFS_CONFIG_TYPES: &str = r#"
export interface IpfsClientConfig {
  ipfsUri?: string | null;
  transport?: HttpTransportConfig;
  transportPolicy?: TransportPolicyConfig | null;
  timeoutMs?: number | null;
}
"#;
