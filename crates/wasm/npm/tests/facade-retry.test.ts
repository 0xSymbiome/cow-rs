import { describe, expect, test, vi } from "vitest";
import { CowError, isRetryable, isUserRejection, normalizeError, retryAfterMs } from "../src/errors.js";
import { withRetry } from "../src/retry.js";

// A retryable orderbook failure: the SDK exhausted its own budget on a transient
// (rate-limit / server) fault, with an optional server-suggested backoff.
function retryableError(retryAfterMs?: number): CowError {
  return normalizeError({
    kind: "orderbook",
    code: "503",
    message: "service unavailable",
    retryable: true,
    retryAfterMs,
  });
}

describe("withRetry", () => {
  test("returns the first success without retrying", async () => {
    const fn = vi.fn().mockResolvedValue("ok");
    await expect(withRetry(fn)).resolves.toBe("ok");
    expect(fn).toHaveBeenCalledTimes(1);
  });

  test("retries a retryable failure, then succeeds", async () => {
    const fn = vi.fn().mockRejectedValueOnce(retryableError(1)).mockResolvedValue("ok");
    await expect(withRetry(fn, { retries: 2, baseDelayMs: 1 })).resolves.toBe("ok");
    expect(fn).toHaveBeenCalledTimes(2);
  });

  test("never retries a non-retryable failure", async () => {
    const fn = vi.fn().mockRejectedValue(normalizeError({ kind: "invalidInput", message: "bad" }));
    await expect(withRetry(fn, { retries: 5, baseDelayMs: 1 })).rejects.toBeInstanceOf(CowError);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  test("gives up after the retry budget and rethrows the last error as a CowError", async () => {
    const fn = vi.fn().mockRejectedValue(retryableError(1));
    await expect(withRetry(fn, { retries: 2, baseDelayMs: 1 })).rejects.toMatchObject({
      kind: "orderbook",
      retryable: true,
    });
    // Initial attempt plus two retries.
    expect(fn).toHaveBeenCalledTimes(3);
  });

  test("normalizes a non-CowError throw and does not retry an internal fault", async () => {
    const fn = vi.fn().mockRejectedValue("a bare string");
    await expect(withRetry(fn, { retries: 3, baseDelayMs: 1 })).rejects.toBeInstanceOf(CowError);
    expect(fn).toHaveBeenCalledTimes(1);
  });

  test("onRetry fires once per retry with the attempt, error, and delay", async () => {
    const seen: Array<{ attempt: number; kind: string; delayMs: number }> = [];
    let calls = 0;
    const fn = vi.fn().mockImplementation(async () => {
      calls += 1;
      if (calls < 3) throw retryableError();
      return "ok";
    });
    await withRetry(fn, {
      retries: 3,
      baseDelayMs: 1,
      onRetry: (attempt, error, delayMs) => seen.push({ attempt, kind: error.kind, delayMs }),
    });
    expect(seen.map((s) => s.attempt)).toEqual([1, 2]);
    expect(seen.every((s) => s.kind === "orderbook" && s.delayMs >= 1)).toBe(true);
  });

  test("a throwing onRetry never aborts the retry or masks the error", async () => {
    let calls = 0;
    const fn = vi.fn().mockImplementation(async () => {
      calls += 1;
      if (calls < 2) throw retryableError();
      return "ok";
    });
    await expect(
      withRetry(fn, {
        retries: 3,
        baseDelayMs: 1,
        onRetry: () => {
          throw new Error("telemetry boom");
        },
      }),
    ).resolves.toBe("ok");
  });
});

describe("isRetryable / retryAfterMs predicates", () => {
  test("isRetryable is true only for an orderbook error flagged retryable", () => {
    expect(isRetryable(normalizeError({ kind: "orderbook", message: "x", retryable: true }))).toBe(true);
    expect(isRetryable(normalizeError({ kind: "orderbook", message: "x" }))).toBe(false);
    expect(isRetryable(normalizeError({ kind: "signing", message: "x" }))).toBe(false);
    // A plain object is not a CowError, so the instance gate rejects it.
    expect(isRetryable({ kind: "orderbook", retryable: true })).toBe(false);
  });

  test("retryAfterMs returns the hint only for an orderbook error", () => {
    expect(retryAfterMs(normalizeError({ kind: "orderbook", message: "x", retryAfterMs: 1500 }))).toBe(1500);
    expect(retryAfterMs(normalizeError({ kind: "orderbook", message: "x" }))).toBeUndefined();
    expect(retryAfterMs(normalizeError({ kind: "signing", message: "x" }))).toBeUndefined();
  });
});

describe("isUserRejection", () => {
  test("true for a declined wallet request (4001) and a cancelled operation", () => {
    expect(
      isUserRejection(
        normalizeError({ kind: "walletRequest", method: "eth_signTypedData_v4", code: 4001, message: "rejected" }),
      ),
    ).toBe(true);
    expect(isUserRejection(normalizeError({ kind: "cancelled", message: "cancelled" }))).toBe(true);
  });

  test("false for a non-4001 wallet error, any other kind, and a plain object", () => {
    expect(
      isUserRejection(
        normalizeError({ kind: "walletRequest", method: "eth_sendTransaction", code: 4100, message: "unauthorized" }),
      ),
    ).toBe(false);
    expect(isUserRejection(normalizeError({ kind: "orderbook", message: "x", retryable: true }))).toBe(false);
    expect(isUserRejection({ kind: "cancelled" })).toBe(false);
  });
});
