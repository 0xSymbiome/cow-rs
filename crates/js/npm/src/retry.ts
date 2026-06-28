import { type CowError, isCowError, isRetryable, normalizeError, retryAfterMs } from "./errors.js";

/** Options for {@link withRetry}. */
export interface RetryOptions {
  /** Maximum retry attempts after the first call. Default `3`. */
  retries?: number;
  /** Base backoff in milliseconds when the server gave no `Retry-After`. Default `500`. */
  baseDelayMs?: number;
  /** Upper bound on any single backoff wait, in milliseconds. Default `30_000`. */
  maxDelayMs?: number;
  /** Abort signal that cancels a pending backoff and rejects with its reason. */
  signal?: AbortSignal;
  /**
   * Fire-and-forget hook invoked before each backoff wait, for retry telemetry.
   * It receives the 1-based number of the attempt that just failed, the
   * normalized {@link CowError}, and the chosen delay in milliseconds. It is not
   * awaited and a throw from it is swallowed, so a buggy hook can never abort the
   * retry or mask the underlying error.
   */
  onRetry?(attempt: number, error: CowError, delayMs: number): void;
}

/**
 * Runs `fn`, retrying only a failure the SDK itself classifies as retryable
 * ({@link isRetryable}) — a transient rate-limit or server fault on which the SDK
 * already exhausted its own internal budget. It waits the server-suggested
 * `Retry-After` ({@link retryAfterMs}) when present, otherwise an exponential
 * backoff, for up to `retries` attempts.
 *
 * Any non-retryable failure — invalid input, an unsupported chain, a wallet
 * rejection, or an orderbook rejection decided on the request's merits — is
 * rethrown immediately as a {@link CowError}; it will never be retried. Order
 * creation is idempotent (a resubmission returns the same `orderUid`), so
 * retrying a transient post failure does not risk a duplicate order.
 *
 * Aborting `options.signal` during a backoff wait rejects with the signal's
 * reason (an `AbortError`), not a `CowError` — abort is caller control flow, not
 * an SDK fault.
 */
export async function withRetry<T>(fn: () => Promise<T>, options: RetryOptions = {}): Promise<T> {
  const retries = options.retries ?? 3;
  const baseDelayMs = options.baseDelayMs ?? 500;
  const maxDelayMs = options.maxDelayMs ?? 30_000;

  for (let attempt = 0; ; attempt++) {
    try {
      return await fn();
    } catch (thrown) {
      const error: CowError = isCowError(thrown) ? thrown : normalizeError(thrown);
      if (attempt >= retries || !isRetryable(error)) {
        throw error;
      }
      const backoff = baseDelayMs * 2 ** attempt;
      const delay = Math.min(retryAfterMs(error) ?? backoff, maxDelayMs);
      if (options.onRetry) {
        try {
          options.onRetry(attempt + 1, error, delay);
        } catch {
          // A telemetry hook must never break the retry loop or mask the error.
        }
      }
      await sleep(delay, options.signal);
    }
  }
}

function sleep(ms: number, signal?: AbortSignal): Promise<void> {
  return new Promise((resolve, reject) => {
    if (signal?.aborted) {
      reject(signal.reason ?? new DOMException("The retry backoff was aborted.", "AbortError"));
      return;
    }
    const onAbort = () => {
      clearTimeout(timer);
      reject(signal?.reason ?? new DOMException("The retry backoff was aborted.", "AbortError"));
    };
    const timer = setTimeout(() => {
      signal?.removeEventListener("abort", onAbort);
      resolve();
    }, ms);
    signal?.addEventListener("abort", onAbort, { once: true });
  });
}
