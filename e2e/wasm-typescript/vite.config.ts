import { defineConfig } from "vite";
import topLevelAwait from "vite-plugin-top-level-await";
import wasm from "vite-plugin-wasm";

// The `cow-sdk-wasm-test-package` bundler entry uses the ESM
// integration proposal for WebAssembly (`import * as wasm from
// "./cow_sdk_wasm_bg.wasm";`). Vite 8 removed the experimental
// `builtin:vite-wasm-fallback` plugin, so the `vite-plugin-wasm`
// official plugin (paired with `vite-plugin-top-level-await` because
// the generated init flow uses top-level `await`) handles the import
// on the dev server and in the production bundle.
export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  optimizeDeps: {
    exclude: ["cow-sdk-wasm-test-package"]
  }
});
