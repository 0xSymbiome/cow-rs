import { exports } from "cloudflare:workers";
import { describe, expect, test } from "vitest";

describe("Cloudflare gateway worker", () => {
  test("initializes the cloudflare wasm flavor and reports chains", async () => {
    const response = await exports.default.fetch("https://example.test/health");
    const payload = await response.json<{ ok: boolean; supportedChainIds: number[] }>();

    expect(payload.ok).toBe(true);
    expect(payload.supportedChainIds).toContain(1);
  });
});
