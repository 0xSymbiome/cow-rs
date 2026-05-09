import { defineConfig } from "vite";

export default defineConfig({
  optimizeDeps: {
    exclude: ["cow-sdk-wasm-test-package"]
  }
});
