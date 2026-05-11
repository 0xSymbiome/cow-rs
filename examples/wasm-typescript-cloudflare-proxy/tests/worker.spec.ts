import { SELF } from "cloudflare:test";
import { describe, expect, test } from "vitest";

describe("Cloudflare worker example", () => {
  test("initializes the cloudflare wasm flavor", async () => {
    const response = await SELF.fetch("https://example.test/health");
    const payload = await response.json<{ ok: boolean; supportedChainIds: number[] }>();

    expect(payload.ok).toBe(true);
    expect(payload.supportedChainIds).toContain(1);
  });
});
