import { SELF } from "cloudflare:test";
import { describe, expect, test } from "vitest";

describe("Worker initialization", () => {
  test("initializes the wasm module once per isolate", async () => {
    await SELF.fetch("https://example.test/version");
    await SELF.fetch("https://example.test/chains");
    const response = await SELF.fetch("https://example.test/init-count");
    const payload = await response.json<{ initCount: number }>();

    expect(payload.initCount).toBe(1);
  });
});
