import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(scriptDir, "..");
const descriptor = JSON.parse(readFileSync(join(packageRoot, "flavours.json"), "utf8"));
let failed = false;

const forbidden = [
  /\bapp_data_hex\b/,
  /\bchain_id\b/,
  /\bcustom_callback\b/,
  /\bdigest_signer\b/,
  /\becdsa_signature\b/,
  /\bfetch_callback\b/,
  /\border_uid\b/,
  /\border_uids\b/,
  /\brequest_callback\b/,
  /\bsigner_callback\b/,
  /\btimeout_ms\b/,
  /\btyped_data_signer\b/,
  /\bOrderBookClientWithFetch\b/,
  /\bSubgraphClientWithFetch\b/,
  /\bTradingClientWithFetch\b/,
  /\bIpfsClientWithFetch\b/,
  /\bregisterFetchCallback\b/,
  /\bFetchCallbackHandle\b/,
  /\bFunction\b/,
  // The raw wasm-bindgen `free()` stays hidden behind the facade's `dispose()`.
  // `[Symbol.dispose]` (and its `esnext.disposable` lib reference) are now part
  // of the facade contract — clients implement it so `using` works — so they are
  // intentionally allowed.
  /\bfree\(\): void\b/
];

function fail(message) {
  console.error(`verify-facade-denylist: ${message}`);
  failed = true;
}

function declarationFiles(directory, output = []) {
  if (!existsSync(directory)) {
    return output;
  }
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      declarationFiles(path, output);
    } else if (entry.name.endsWith(".d.ts")) {
      output.push(path);
    }
  }
  return output;
}

for (const flavour of descriptor.flavours) {
  const declaration = join(packageRoot, "dist", flavour.name, "index.d.ts");
  if (!existsSync(declaration)) {
    fail(`${flavour.name} facade declaration is missing`);
    continue;
  }
}

for (const flavour of descriptor.flavours) {
  const directory = join(packageRoot, "dist", flavour.name);
  for (const file of declarationFiles(directory)) {
    const content = readFileSync(file, "utf8");
    for (const pattern of forbidden) {
      if (pattern.test(content)) {
        fail(`${relative(packageRoot, file)} contains forbidden pattern ${pattern}`);
      }
    }
  }
}

if (failed) {
  process.exit(1);
}

console.log("verify-facade-denylist: public facade declarations passed");
