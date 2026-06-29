import { defineConfig } from "vite";

// `cow-sdk-js-test-package` resolves to its web-target build for the browser
// (`browser` / `import` conditions): the facade calls `await initialize()` from an
// async function, and the loader fetches the wasm through
// `new URL('..._bg.wasm', import.meta.url)` — an asset Vite emits and resolves
// natively. The bundler-target `import * as wasm` ESM integration, which needed
// `vite-plugin-wasm` plus `vite-plugin-top-level-await`, is no longer used; the
// top-level-await plugin would even downlevel the bundle and fail. The package is
// excluded from dependency pre-bundling so Vite serves its ESM facade and wasm
// asset directly.
export default defineConfig({
  optimizeDeps: {
    exclude: ["cow-sdk-js-test-package"]
  }
});
