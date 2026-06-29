import { describe, expect, test } from "vitest";
import { CowError, isCowError, normalizeError } from "../src/errors.js";

describe("facade error normalization", () => {
  test("keeps typed error fields and adds an actionable message when absent", () => {
    const error = normalizeError({ kind: "unsupportedChain", chainId: 5 });
    expect(error).toBeInstanceOf(CowError);
    expect(error).toMatchObject({
      kind: "unsupportedChain",
      chainId: 5,
      message: expect.stringContaining("supportedChainIds"),
    });
  });

  test("wraps future typed errors with the unknown sentinel and preserves raw", () => {
    const raw = { kind: "futureError", detail: "new" };
    const error = normalizeError(raw);
    expect(error).toBeInstanceOf(CowError);
    expect(error).toMatchObject({
      kind: "__unknown",
      message: expect.stringContaining("unrecognized error variant"),
      raw,
    });
  });

  test("turns JavaScript exceptions into internal SDK errors", () => {
    const error = normalizeError(new Error("boom"));
    expect(error).toBeInstanceOf(CowError);
    expect(error).toMatchObject({ kind: "internal", message: expect.stringContaining("boom") });
  });

  // Input-DTO deserialization failures cross the wasm boundary as a plain
  // `Error` (the generated `from_wasm_abi` glue throws the serde message), so
  // they carry no structured `kind`. They are caller input errors and must
  // normalize to `invalidInput`, not `internal`.
  test("classifies serde deserialization failures as invalidInput", () => {
    expect(
      normalizeError(new Error("unknown variant `teleport`, expected `sell` or `buy`")),
    ).toMatchObject({
      kind: "invalidInput",
      message: expect.stringContaining("unknown variant `teleport`, expected `sell` or `buy`"),
    });

    expect(normalizeError(new Error("invalid type: integer `1`, expected a string"))).toMatchObject({
      kind: "invalidInput",
    });
  });

  test("extracts the field name from missing/unknown field failures", () => {
    expect(normalizeError(new Error("missing field `appCode`"))).toMatchObject({
      kind: "invalidInput",
      field: "appCode",
    });

    expect(
      normalizeError(new Error("unknown field `frobnicate`, expected one of `appCode`")),
    ).toMatchObject({ kind: "invalidInput", field: "frobnicate" });
  });

  test("does not misclassify unrelated exceptions as invalidInput", () => {
    // No serde-failure phrasing → stays internal.
    expect(normalizeError(new Error("connection reset"))).toMatchObject({ kind: "internal" });
  });

  test("adds actionable guidance to timeout and cancellation errors", () => {
    expect(normalizeError({ kind: "walletTimeout", timeoutMs: 250 })).toMatchObject({
      kind: "walletTimeout",
      timeoutMs: 250,
      message: expect.stringContaining("walletConfig.timeoutMs"),
    });

    expect(normalizeError({ kind: "cancelled" })).toMatchObject({
      kind: "cancelled",
      message: expect.stringContaining("AbortController"),
    });
  });

  test("preserves the orderbook errorType tag alongside the coarse category", () => {
    const error = normalizeError({
      kind: "orderbook",
      code: "400",
      category: "insufficientFunds",
      errorType: "InsufficientAllowance",
      message: "rejected",
      retryable: false,
    });
    // The fine-grained services tag survives the boundary, letting a consumer
    // tell InsufficientAllowance (approve) from InsufficientBalance (fund),
    // where the coarse category cannot.
    if (isCowError(error) && error.kind === "orderbook") {
      expect(error.errorType).toBe("InsufficientAllowance");
      expect(error.category).toBe("insufficientFunds");
    } else {
      throw new Error("expected a narrowable orderbook CowError");
    }
    expect(JSON.stringify(error)).toContain("InsufficientAllowance");
  });
});

describe("CowError is an Error subclass", () => {
  test("a normalized error is both a CowError and an Error, and isCowError agrees", () => {
    const error = normalizeError({ kind: "internal", message: "x" });
    expect(error).toBeInstanceOf(CowError);
    expect(error).toBeInstanceOf(Error);
    expect(isCowError(error)).toBe(true);
    // A plain object with the same fields is NOT a CowError — the guard is a
    // real instance check, not the old structural fingerprint.
    expect(isCowError({ kind: "internal", message: "x" })).toBe(false);
  });

  test("message is enumerable so JSON.stringify keeps it alongside the fields", () => {
    const error = normalizeError({ kind: "unsupportedChain", chainId: 5 });
    const json = JSON.parse(JSON.stringify(error));
    expect(json.kind).toBe("unsupportedChain");
    expect(json.chainId).toBe(5);
    expect(typeof json.message).toBe("string");
    expect(json.message.length).toBeGreaterThan(0);
  });

  test("toJSON and JSON.stringify omit name and stack", () => {
    const error = normalizeError({ kind: "internal", message: "boom" });
    const json = error.toJSON() as Record<string, unknown>;
    expect(json).not.toHaveProperty("name");
    expect(json).not.toHaveProperty("stack");
    expect(JSON.stringify(error)).not.toContain("stack");
    // The stack stays available for local debugging; it just never ships in the
    // JSON form, and it only embeds the already-redacted message.
    expect(typeof error.stack).toBe("string");
  });

  test("fromJSON round-trips a serialized error back into an instance", () => {
    const original = normalizeError({
      kind: "unknownEnumValue",
      field: "kind",
      value: "teleport",
      message: "bad",
    });
    const revived = CowError.fromJSON(JSON.parse(JSON.stringify(original)));
    expect(revived).toBeInstanceOf(CowError);
    expect(revived.kind).toBe("unknownEnumValue");
    expect(revived).toMatchObject({ field: "kind", value: "teleport" });
  });

  test("fromJSON survives a structuredClone (cross-realm) round-trip, preserving raw", () => {
    const original = normalizeError({ kind: "weirdFutureKind", oops: 1 });
    expect(original.kind).toBe("__unknown");
    // A bare structuredClone of an Error drops its own fields and prototype;
    // routing toJSON() through fromJSON() restores both, including `raw`.
    const revived = CowError.fromJSON(structuredClone(original.toJSON()));
    expect(revived).toBeInstanceOf(CowError);
    expect(revived.kind).toBe("__unknown");
    expect(revived).toMatchObject({ raw: { kind: "weirdFutureKind", oops: 1 } });
  });
});
