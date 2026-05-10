import { cancelledError, invalidInput, normalizeError, type SdkError } from "./errors.js";
import type { CowFetchCallback, CowFetchRequest, CowFetchResponse } from "./callbacks.js";
import type { CommonClientConfig, HttpTransportConfig, SdkClientOptions } from "./options.js";

export interface ClientDefaults {
  signal: AbortSignal | undefined;
  timeoutMs: number | null | undefined;
}

export type DisposableRawClient = {
  free?: () => void;
};

export function translateHttpTransport(transport: HttpTransportConfig): {
  kind: "callback";
  callback: CowFetchCallback;
} {
  if (transport.kind === "callback") {
    return transport;
  }

  const fetchFn = transport.fetch ?? globalThis.fetch;
  if (typeof fetchFn !== "function") {
    throw invalidInput(
      "transport.fetch",
      "globalThis.fetch is unavailable; pass an explicit fetch function"
    );
  }

  return {
    kind: "callback",
    callback: adaptFetchToCallback(fetchFn)
  };
}

export function translateClientConfig<T extends { transport: HttpTransportConfig }>(
  config: T
): Omit<T, "signal" | "transport"> & { transport: { kind: "callback"; callback: CowFetchCallback } } {
  const { signal: _signal, transport, ...rest } = config as T & { signal?: AbortSignal };
  return {
    ...rest,
    transport: translateHttpTransport(transport)
  };
}

export function mergeOptions(
  defaults: ClientDefaults,
  options?: SdkClientOptions | null
): SdkClientOptions | undefined {
  const signal = options?.signal ?? defaults.signal;
  if (signal?.aborted) {
    throw cancelledError();
  }

  const timeoutMs = options?.timeoutMs ?? defaults.timeoutMs ?? undefined;
  if (signal || timeoutMs !== undefined) {
    const merged: SdkClientOptions = {};
    if (signal) {
      merged.signal = signal;
    }
    if (timeoutMs !== undefined && timeoutMs !== null) {
      merged.timeoutMs = timeoutMs;
    }
    return merged;
  }
  return undefined;
}

export function assertActive(disposed: boolean): void {
  if (disposed) {
    throw invalidInput("client", "client has been disposed");
  }
}

export function disposeRaw(raw: DisposableRawClient | undefined): void {
  raw?.free?.();
}

export function normalizeThrown(error: unknown): SdkError {
  return normalizeError(error);
}

export async function callAsync<T>(operation: () => Promise<T>): Promise<T> {
  try {
    return await operation();
  } catch (error) {
    throw normalizeError(error);
  }
}

export function callSync<T>(operation: () => T): T {
  try {
    return operation();
  } catch (error) {
    throw normalizeError(error);
  }
}

export function defaultsFrom(config: CommonClientConfig | { signal?: AbortSignal; timeoutMs?: number | null }): ClientDefaults {
  return {
    signal: config.signal,
    timeoutMs: config.timeoutMs
  };
}

function adaptFetchToCallback(fetchFn: typeof globalThis.fetch): CowFetchCallback {
  return async (request: CowFetchRequest): Promise<CowFetchResponse> => {
    const init: RequestInit = {
      method: request.method,
      headers: request.headers
    };
    if (request.signal) {
      init.signal = request.signal;
    }
    if (request.body !== undefined) {
      init.body = request.body;
    }

    const response = await fetchFn(request.url, init);
    const headers: Record<string, string> = {};
    response.headers.forEach((value, key) => {
      headers[key] = value;
    });

    return {
      status: response.status,
      statusText: response.statusText,
      headers,
      body: await response.text()
    };
  };
}
