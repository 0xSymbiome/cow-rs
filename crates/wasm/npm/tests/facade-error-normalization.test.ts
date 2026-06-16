import { describe, expect, test } from "vitest";
import { normalizeError } from "../src/errors.js";

describe("facade error normalization", () => {
  test("keeps typed error fields and adds an actionable message when absent", () => {
    expect(
      normalizeError({
        schemaVersion: "v1",
        kind: "unsupportedChain",
        chainId: 5
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

  // Input-DTO deserialization failures cross the wasm boundary as a plain
  // `Error` (the generated `from_wasm_abi` glue throws the serde message),
  // so they carry no structured `kind`. They are caller input errors and
  // must normalize to `invalidInput`, not `internal`.
  test("classifies serde deserialization failures as invalidInput", () => {
    expect(normalizeError(new Error("unknown variant `teleport`, expected `sell` or `buy`")))
      .toMatchObject({
        schemaVersion: "v1",
        kind: "invalidInput",
        message: expect.stringContaining("unknown variant `teleport`, expected `sell` or `buy`")
      });

    expect(normalizeError(new Error("invalid type: integer `1`, expected a string")))
      .toMatchObject({ schemaVersion: "v1", kind: "invalidInput" });
  });

  test("extracts the field name from missing/unknown field failures", () => {
    expect(normalizeError(new Error("missing field `appCode`"))).toMatchObject({
      schemaVersion: "v1",
      kind: "invalidInput",
      field: "appCode"
    });

    expect(normalizeError(new Error("unknown field `frobnicate`, expected one of `appCode`")))
      .toMatchObject({ kind: "invalidInput", field: "frobnicate" });
  });

  test("does not misclassify unrelated exceptions as invalidInput", () => {
    // No serde-failure phrasing → stays internal.
    expect(normalizeError(new Error("connection reset"))).toMatchObject({
      kind: "internal"
    });
  });

  test("adds actionable guidance to timeout and cancellation errors", () => {
    expect(normalizeError({ schemaVersion: "v1", kind: "walletTimeout", timeoutMs: 250 }))
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
