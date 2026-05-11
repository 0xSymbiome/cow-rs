import type { SchemaVersion } from "./envelope.js";

export type SdkError =
  | { schemaVersion: "v1"; kind: "invalidInput"; message: string; field?: string }
  | { schemaVersion: "v1"; kind: "unknownEnumValue"; message: string; field: string; value: string }
  | { schemaVersion: "v1"; kind: "unsupportedChain"; message: string; chainId: number }
  | {
      schemaVersion: "v1";
      kind: "walletRequest";
      method: string;
      code?: number;
      message: string;
      data?: unknown;
    }
  | { schemaVersion: "v1"; kind: "walletTimeout"; message: string; timeoutMs: number }
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
  | { schemaVersion: "v1"; kind: "forbiddenInteraction"; message: string; target: string; reason: string }
  | { schemaVersion: "v1"; kind: "cancelled"; message: string }
  | { schemaVersion: "v1"; kind: "internal"; message: string }
  | { schemaVersion: SchemaVersion; kind: "__unknown"; message: string; raw: unknown };

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
          message: unknownMessage(),
          raw: normalized.raw ?? raw
        };
      }

      return withActionableMessage({
        ...normalized,
        schemaVersion,
        kind
      } as SdkError);
    }

    if (kind) {
      return {
        schemaVersion: normalized.schemaVersion === "__unknown" ? "__unknown" : "v1",
        kind: "__unknown",
        message: unknownMessage(),
        raw
      };
    }
  }

  if (raw instanceof Error) {
    return { schemaVersion: "v1", kind: "internal", message: internalMessage(raw.message) };
  }

  return { schemaVersion: "v1", kind: "internal", message: internalMessage(String(raw)) };
}

export function cancelledError(): SdkError {
  return {
    schemaVersion: "v1",
    kind: "cancelled",
    message: "Operation was cancelled. Create a fresh AbortController or retry without an already-aborted signal."
  };
}

export function invalidInput(field: string, reason: string): SdkError {
  return {
    schemaVersion: "v1",
    kind: "invalidInput",
    field,
    message: `Invalid \`${field}\`: ${reason}. Check the value supplied for \`${field}\` and retry with a valid SDK input.`
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

function withActionableMessage(error: SdkError): SdkError {
  if ("message" in error && typeof error.message === "string" && error.message.length > 0) {
    return error;
  }

  switch (error.kind) {
    case "unknownEnumValue":
      return {
        ...error,
        message: `Unsupported value \`${error.value}\` for \`${error.field}\`. Use one of the documented values for this field.`
      };
    case "unsupportedChain":
      return {
        ...error,
        message: `Unsupported chain ID ${error.chainId}. Call supportedChainIds() before constructing requests and route unsupported networks to another integration.`
      };
    case "walletTimeout":
      return {
        ...error,
        message: `Wallet request timed out after ${error.timeoutMs} ms. Increase walletConfig.timeoutMs or ask the user to approve the wallet request before the timeout.`
      };
    case "forbiddenInteraction":
      return {
        ...error,
        message: `Forbidden settlement interaction target \`${error.target}\`. Remove this target from settlement interactions before signing or submitting the order.`
      };
    case "cancelled":
      return cancelledError();
    case "__unknown":
      return {
        ...error,
        message: unknownMessage()
      };
    default:
      return error;
  }
}

function internalMessage(detail: string): string {
  return `SDK internal error: ${detail}. This indicates serialization or invariant failure; retry with the same inputs only after checking the reported input shape.`;
}

function unknownMessage(): string {
  return "SDK received an unrecognized error variant. Inspect raw, preserve it in logs without credentials, and update the SDK if the variant is now documented.";
}
