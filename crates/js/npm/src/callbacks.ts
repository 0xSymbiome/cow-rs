export type SupportedChainId =
  | 1
  | 100
  | 137
  | 8453
  | 42161
  | 43114
  | 56
  | 11155111
  | 9745
  | 57073
  | 59144;

export type CowEnv = "prod" | "staging";

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
  request: CowFetchRequest
) => Promise<CowFetchResponse> | CowFetchResponse;

// Fields are required to match the Rust `TypedDataDomain` in `cow-sdk-core`:
// the SDK always emits a fully-populated EIP-712 domain across the ABI, so the
// envelope a host callback receives carries every field. The per-flavour raw
// module exports the same `TypedDataEnvelope` generated from the native type;
// this flavour-agnostic mirror is the callback-parameter shape only.
export interface TypedDataDomain {
  name: string;
  version: string;
  chainId: number;
  verifyingContract: string;
}

export interface TypedDataField {
  name: string;
  type: string;
}

export interface TypedDataEnvelope<M = unknown> {
  domain: TypedDataDomain;
  types: Record<string, TypedDataField[]>;
  primaryType: string;
  message: M;
}

export interface CowEip1271SignRequest {
  order: unknown;
  typedData: TypedDataEnvelope;
  owner: string;
  chainId: number;
}

export interface ContractCall {
  address: string;
  method: string;
  abiJson: string;
  argsJson: string;
}

export type TypedDataSignerCallback = (
  envelope: TypedDataEnvelope
) => Promise<string> | string;

export type DigestSignerCallback = (digest: string) => Promise<string> | string;

export type CustomEip1271Callback = (
  request: CowEip1271SignRequest
) => Promise<string> | string;

/**
 * Performs a read-only contract call on behalf of the SDK and returns the
 * ABI-decoded result.
 *
 * The callback must return the decoded value as a decimal string or number, not
 * the raw `0x`-hex `eth_call` payload. With viem, pass the `readContract`
 * result through `String(value)`:
 *
 * ```ts
 * const readContract: ContractReadCallback = async ({ address, method, abiJson, argsJson }) =>
 *   String(await publicClient.readContract({
 *     address: address as `0x${string}`,
 *     abi: JSON.parse(abiJson),
 *     functionName: method,
 *     args: JSON.parse(argsJson)
 *   }));
 * ```
 */
export type ContractReadCallback = (
  request: ContractCall
) => Promise<string> | string;
