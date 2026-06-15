import { exports } from "cloudflare:workers";
import { describe, expect, test } from "vitest";

describe("Worker wasm helpers", () => {
  test("returns the wasm version", async () => {
    const response = await exports.default.fetch("https://example.test/version");
    const payload = await response.json<{ version: string }>();

    expect(payload.version).toMatch(/^\d+\.\d+\.\d+/);
  });

  test("returns supported chains", async () => {
    const response = await exports.default.fetch("https://example.test/chains");
    const payload = await response.json<{ chains: number[] }>();

    expect(payload.chains).toContain(1);
    expect(payload.chains).toContain(100);
  });

  test("returns a CID from app-data hash", async () => {
    const response = await exports.default.fetch("https://example.test/cid");
    const payload = await response.json<{ cid: string }>();

    expect(payload.cid).toBe("f01551b20337aa6e6c2a7a0d1eb79a35ebd88b08fc963d5f7a3fc953b7ffb2b7f5898a1df");
  });

  test("returns an order UID", async () => {
    const response = await exports.default.fetch("https://example.test/uid");
    const payload = await response.json<{ orderUid: string; orderDigest: string }>();

    expect(payload.orderUid).toMatch(/^0x[0-9a-f]{112}$/);
    expect(payload.orderDigest).toMatch(/^0x[0-9a-f]{64}$/);
  });
});
