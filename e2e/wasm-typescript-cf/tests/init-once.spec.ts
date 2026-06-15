import { exports } from "cloudflare:workers";
import { describe, expect, test } from "vitest";

describe("Worker initialization", () => {
  test("initializes the wasm module once per isolate", async () => {
    await exports.default.fetch("https://example.test/version");
    await exports.default.fetch("https://example.test/chains");
    const response = await exports.default.fetch("https://example.test/init-count");
    const payload = await response.json<{ initCount: number }>();

    expect(payload.initCount).toBe(1);
  });
});
