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

export interface TypedDataDomainDto {
  name?: string;
  version?: string;
  chainId?: number;
  verifyingContract?: string;
  salt?: string;
}

export interface TypedDataFieldDto {
  name: string;
  type: string;
}

export interface TypedDataEnvelopeDto {
  domain: TypedDataDomainDto;
  types: Record<string, TypedDataFieldDto[]>;
  primaryType: string;
  message: unknown;
}

export interface CowEip1271SignRequest {
  order: unknown;
  typedData: TypedDataEnvelopeDto;
  owner: string;
  chainId: number;
}

export interface ContractCallDto {
  address: string;
  method: string;
  abiJson: string;
  argsJson: string;
}

export type TypedDataSignerCallback = (
  envelope: TypedDataEnvelopeDto
) => Promise<string> | string;

export type Eip1193RequestCallback = (request: {
  method: string;
  params?: unknown[];
}) => Promise<unknown> | unknown;

export type DigestSignerCallback = (digest: string) => Promise<string> | string;

export type CowEip1271SignCallback = (
  request: CowEip1271SignRequest
) => Promise<string> | string;

export type CustomEip1271Callback = CowEip1271SignCallback;

export type ContractReadCallback = (
  request: ContractCallDto
) => Promise<string> | string;
