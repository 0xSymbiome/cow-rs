use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const CALLBACK_TYPES: &str = r#"
export type CowFetchMethod = "GET" | "POST" | "PUT" | "DELETE";
export type Value = unknown;

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
  headers: Record<string, string>;
  body: string;
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
"#;
