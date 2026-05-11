import { access, copyFile, mkdir, rm } from "node:fs/promises";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { build } from "esbuild";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outDir = join(root, "dist-worker");
const wasmFile = "cow_sdk_wasm_bg.wasm";
const wasmSource = join(
  root,
  "node_modules",
  "cow-sdk-wasm-local",
  "dist",
  "raw",
  "cloudflare-web",
  wasmFile
);

await access(wasmSource).catch(() => {
  throw new Error(
    "Missing Cloudflare WASM artifact. Run `pnpm install --frozen-lockfile` and build the local package before building this example."
  );
});

await rm(outDir, { recursive: true, force: true });
await mkdir(outDir, { recursive: true });
await copyFile(wasmSource, join(outDir, wasmFile));

await build({
  absWorkingDir: root,
  bundle: true,
  conditions: ["workerd", "browser", "import", "module"],
  entryPoints: ["src/worker.ts"],
  format: "esm",
  logLevel: "info",
  mainFields: ["browser", "module", "main"],
  outfile: "dist-worker/worker.js",
  platform: "browser",
  target: "es2022",
  plugins: [
    {
      name: "cow-sdk-wasm-cloudflare-module",
      setup(build) {
        build.onResolve(
          { filter: /^cow-sdk-wasm-local\/cloudflare\/wasm$/ },
          () => ({
            namespace: "cow-sdk-wasm-module",
            path: "cloudflare-wasm"
          })
        );

        build.onResolve({ filter: /^\.\/cow_sdk_wasm_bg\.wasm$/ }, (args) => ({
          external: true,
          path: args.path
        }));

        build.onLoad({ filter: /.*/, namespace: "cow-sdk-wasm-module" }, () => ({
          contents: `import wasmModule from "./${wasmFile}";\nexport default wasmModule;\n`,
          loader: "js",
          resolveDir: outDir
        }));
      }
    }
  ]
});
