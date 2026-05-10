import type { SchemaVersion } from "./envelope.js";

export type SdkError =
  | { schemaVersion: "v1"; kind: "invalidInput"; message: string; field?: string }
  | { schemaVersion: "v1"; kind: "unknownEnumValue"; field: string; value: string }
  | { schemaVersion: "v1"; kind: "unsupportedChain"; chainId: number }
  | {
      schemaVersion: "v1";
      kind: "walletRequest";
      method: string;
      code?: number;
      message: string;
      data?: unknown;
    }
  | { schemaVersion: "v1"; kind: "walletTimeout"; timeoutMs: number }
  | {
      schemaVersion: "v1";
      kind: "transport";
      class: string;
      message: string;
      status?: number;
      headers?: [string, string][];
      body?: string;
    }
  | { schemaVersion: "v1"; kind: "orderbook"; code?: string; message: string }
  | { schemaVersion: "v1"; kind: "subgraph"; message: string }
  | { schemaVersion: "v1"; kind: "signing"; message: string }
  | { schemaVersion: "v1"; kind: "appData"; class?: string; message: string }
  | { schemaVersion: "v1"; kind: "forbiddenInteraction"; target: string; reason: string }
  | { schemaVersion: "v1"; kind: "cancelled" }
  | { schemaVersion: "v1"; kind: "internal"; message: string }
  | { schemaVersion: SchemaVersion; kind: "__unknown"; raw: unknown };

const knownKinds = new Set([
  "invalidInput",
  "unknownEnumValue",
  "unsupportedChain",
  "walletRequest",
  "walletTimeout",
  "transport",
  "orderbook",
  "subgraph",
  "signing",
  "appData",
  "forbiddenInteraction",
  "cancelled",
  "internal",
  "__unknown"
]);

export function normalizeError(raw: unknown): SdkError {
  if (isRecord(raw)) {
    const normalized = camelizeKnownFields(raw);
    const kind = typeof normalized.kind === "string" ? normalized.kind : undefined;

    if (kind && knownKinds.has(kind)) {
      const schemaVersion = normalized.schemaVersion === "__unknown" ? "__unknown" : "v1";
      if (kind === "__unknown") {
        return {
          schemaVersion,
          kind: "__unknown",
          raw: normalized.raw ?? raw
        };
      }

      return {
        ...normalized,
        schemaVersion,
        kind
      } as SdkError;
    }

    if (kind) {
      return {
        schemaVersion: normalized.schemaVersion === "__unknown" ? "__unknown" : "v1",
        kind: "__unknown",
        raw
      };
    }
  }

  if (raw instanceof Error) {
    return { schemaVersion: "v1", kind: "internal", message: raw.message };
  }

  return { schemaVersion: "v1", kind: "internal", message: String(raw) };
}

export function cancelledError(): SdkError {
  return { schemaVersion: "v1", kind: "cancelled" };
}

export function invalidInput(field: string, reason: string): SdkError {
  return {
    schemaVersion: "v1",
    kind: "invalidInput",
    field,
    message: reason
  };
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function camelizeKnownFields(raw: Record<string, unknown>): Record<string, unknown> {
  const normalized: Record<string, unknown> = { ...raw };
  copyField(normalized, raw, "schema_version", "schemaVersion");
  copyField(normalized, raw, "chain_id", "chainId");
  copyField(normalized, raw, "timeout_ms", "timeoutMs");
  copyField(normalized, raw, "order_uid", "orderUid");
  copyField(normalized, raw, "order_uids", "orderUids");
  copyField(normalized, raw, "status_code", "status");
  return normalized;
}

function copyField(
  target: Record<string, unknown>,
  source: Record<string, unknown>,
  from: string,
  to: string
): void {
  if (Object.hasOwn(source, from) && !Object.hasOwn(target, to)) {
    target[to] = source[from];
  }
  delete target[from];
}
