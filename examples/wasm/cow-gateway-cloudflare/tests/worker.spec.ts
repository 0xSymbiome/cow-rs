import { SELF } from "cloudflare:test";
import { describe, expect, test } from "vitest";

describe("Cloudflare gateway worker", () => {
  test("initializes the cloudflare wasm flavor and reports chains", async () => {
    const response = await SELF.fetch("https://example.test/health");
    const payload = await response.json<{ ok: boolean; supportedChainIds: number[] }>();

    expect(payload.ok).toBe(true);
    expect(payload.supportedChainIds).toContain(1);
  });
});
