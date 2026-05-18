import type { PluginOption } from "vite";
import { defineConfig } from "vite";
import topLevelAwaitImport from "vite-plugin-top-level-await";
import wasmImport from "vite-plugin-wasm";

// The `cow-sdk-wasm-test-package` bundler entry uses the ESM
// integration proposal for WebAssembly (`import * as wasm from
// "./cow_sdk_wasm_bg.wasm";`). Vite 8 removed the experimental
// `builtin:vite-wasm-fallback` plugin, so the `vite-plugin-wasm`
// official plugin (paired with `vite-plugin-top-level-await` because
// the generated init flow uses top-level `await`) handles the import
// on the dev server and in the production bundle.
//
// Both plugins ship as CommonJS packages (their `package.json`
// `type` field is unset) whose `.d.ts` files declare
// `export default function ...`. Under TypeScript 6 + NodeNext the
// synthetic default for that shape resolves the default binding to
// the namespace itself rather than to the underlying function, so
// `wasmImport()` fails type-check with TS2349 even though the
// runtime semantics are correct. Re-typing each plugin entry as the
// concrete factory signature keeps the call sites under static
// type-check; the runtime continues to call the same factory
// resolved through `exports.import.default` in each package.
const wasm = wasmImport as unknown as () => PluginOption;
const topLevelAwait = topLevelAwaitImport as unknown as () => PluginOption;

export default defineConfig({
  plugins: [wasm(), topLevelAwait()],
  optimizeDeps: {
    exclude: ["cow-sdk-wasm-test-package"]
  }
});
