import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(scriptDir, "..");
const packagePath = join(packageRoot, "package.json");
let failed = false;

function fail(message) {
  console.error(`verify-no-raw-exports: ${message}`);
  failed = true;
}

function collectStrings(value, output = []) {
  if (typeof value === "string") {
    output.push(value);
  } else if (value && typeof value === "object") {
    for (const nested of Object.values(value)) {
      collectStrings(nested, output);
    }
  }
  return output;
}

if (!existsSync(packagePath)) {
  fail("package.json has not been rendered");
} else {
  const manifest = JSON.parse(readFileSync(packagePath, "utf8"));
  const descriptor = JSON.parse(readFileSync(join(packageRoot, "flavours.json"), "utf8"));
  // The only exports allowed to point at raw wasm-bindgen output are the
  // per-flavour raw wasm module subpaths (for example `./trading/edge/wasm`),
  // which Cloudflare Workers import as a CompiledWasm module and pass to
  // `initialize`.
  const allowedRawWasm = new Map(
    descriptor.flavours
      .filter((flavour) => flavour.rawWasmSubpath)
      .map((flavour) => [flavour.rawWasmSubpath, manifest.exports?.[flavour.rawWasmSubpath]])
  );

  for (const [subpath, target] of Object.entries(manifest.exports ?? {})) {
    for (const value of collectStrings(target)) {
      if (value.includes("/dist/raw/") || value.startsWith("./dist/raw/")) {
        if (!allowedRawWasm.has(subpath) || value !== allowedRawWasm.get(subpath)) {
          fail(`${subpath} exposes raw wasm-bindgen output through ${value}`);
        }
      }
    }
  }
}

if (failed) {
  process.exit(1);
}

console.log("verify-no-raw-exports: package exports expose only facade paths");
