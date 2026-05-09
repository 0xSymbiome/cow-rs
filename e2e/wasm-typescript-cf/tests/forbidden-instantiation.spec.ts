import { describe, expect, test } from "vitest";
import source from "../src/worker.ts?raw";

describe("Worker source", () => {
  test("does not hand-code dynamic WebAssembly compilation", () => {
    const forbidden = [
      "WebAssembly.compile",
      "WebAssembly.compileStreaming",
      "WebAssembly.instantiateStreaming",
      "WebAssembly.instantiate("
    ];

    for (const pattern of forbidden) {
      expect(source.includes(pattern), pattern).toBe(false);
    }
  });
});
