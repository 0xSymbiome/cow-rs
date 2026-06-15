import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    environment: "node",
    globals: true,
    include: ["tests/*.spec.ts"],
    pool: "threads",
    testTimeout: 30_000
  }
});
