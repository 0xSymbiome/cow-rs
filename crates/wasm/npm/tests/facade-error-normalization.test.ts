import { describe, expect, test } from "vitest";
import { normalizeError } from "../src/errors.js";

describe("facade error normalization", () => {
  test("keeps typed errors and camel-cases known raw fields", () => {
    expect(
      normalizeError({
        schemaVersion: "v1",
        kind: "unsupportedChain",
        chain_id: 5
      })
    ).toMatchObject({
      schemaVersion: "v1",
      kind: "unsupportedChain",
      chainId: 5,
      message: expect.stringContaining("supportedChainIds")
    });
  });

  test("wraps future typed errors with the unknown sentinel", () => {
    const raw = { schemaVersion: "v1", kind: "futureError", detail: "new" };
    expect(normalizeError(raw)).toMatchObject({
      schemaVersion: "v1",
      kind: "__unknown",
      message: expect.stringContaining("unrecognized error variant"),
      raw
    });
  });

  test("turns JavaScript exceptions into internal SDK errors", () => {
    expect(normalizeError(new Error("boom"))).toMatchObject({
      schemaVersion: "v1",
      kind: "internal",
      message: expect.stringContaining("boom")
    });
  });

  test("adds actionable guidance to timeout and cancellation errors", () => {
    expect(normalizeError({ schemaVersion: "v1", kind: "walletTimeout", timeout_ms: 250 }))
      .toMatchObject({
        kind: "walletTimeout",
        timeoutMs: 250,
        message: expect.stringContaining("walletConfig.timeoutMs")
      });

    expect(normalizeError({ schemaVersion: "v1", kind: "cancelled" })).toMatchObject({
      kind: "cancelled",
      message: expect.stringContaining("AbortController")
    });
  });
});
