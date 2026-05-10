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
    ).toEqual({
      schemaVersion: "v1",
      kind: "unsupportedChain",
      chainId: 5
    });
  });

  test("wraps future typed errors with the unknown sentinel", () => {
    const raw = { schemaVersion: "v1", kind: "futureError", detail: "new" };
    expect(normalizeError(raw)).toEqual({
      schemaVersion: "v1",
      kind: "__unknown",
      raw
    });
  });

  test("turns JavaScript exceptions into internal SDK errors", () => {
    expect(normalizeError(new Error("boom"))).toEqual({
      schemaVersion: "v1",
      kind: "internal",
      message: "boom"
    });
  });
});
