import { defineConfig, type PluginOption } from "vite";
import * as wasmPlugin from "vite-plugin-wasm";

const wasm = (wasmPlugin as unknown as { default: () => PluginOption }).default;

export default defineConfig({
  plugins: [wasm()],
  optimizeDeps: {
    exclude: ["cow-sdk-wasm-local"]
  }
});
