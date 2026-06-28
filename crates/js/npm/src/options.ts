import type { CowEnv, CowFetchCallback, CowFetchRequest, CowFetchResponse } from "./callbacks.js";

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

export type HttpTransportConfig =
  | { kind: "fetch"; fetch?: typeof globalThis.fetch }
  | { kind: "callback"; callback: CowFetchCallback };

export type JitterStrategyConfig = "none" | "full" | "equal" | "decorrelated";
export type LimiterScopeConfig = "global" | "perHost";

export interface RetryPolicyConfig {
  /** Maximum attempts, including the initial request. */
  maxAttempts?: number;
  /** Base exponential-backoff delay in milliseconds. */
  baseDelayMs?: number;
  /** Maximum exponential-backoff delay in milliseconds. */
  maxDelayMs?: number;
}

export interface RequestRateLimiterConfig {
  /** Request tokens granted per interval. Zero disables limiting. */
  tokensPerInterval?: number;
  /** Limiter interval in milliseconds. */
  intervalMs?: number;
  /** Bucket scope. */
  scope?: LimiterScopeConfig;
}

export interface TransportPolicyConfig {
  retryPolicy?: RetryPolicyConfig;
  requestRateLimiter?: RequestRateLimiterConfig;
  jitterStrategy?: JitterStrategyConfig;
  userAgent?: string;
}

export interface CommonClientConfig {
  chainId: number;
  env?: CowEnv | string | null;
  apiKey?: string | null;
  transport?: HttpTransportConfig;
  transportPolicy?: TransportPolicyConfig | null;
  timeoutMs?: number | null;
  signal?: AbortSignal;
}

export interface IpfsClientConfig {
  ipfsUri?: string | null;
  transport?: HttpTransportConfig;
  transportPolicy?: TransportPolicyConfig | null;
  timeoutMs?: number | null;
  signal?: AbortSignal;
}

export interface SubgraphClientConfig {
  chainId: number;
  apiKey: string;
  transport?: HttpTransportConfig;
  transportPolicy?: TransportPolicyConfig | null;
  timeoutMs?: number | null;
  signal?: AbortSignal;
}

export interface TradingClientConfig extends CommonClientConfig {
  appCode: string;
}

export type { CowFetchCallback, CowFetchRequest, CowFetchResponse };
