// Deduplicate the wasm binary across a flavour's bundler/nodejs targets.
//
// wasm-pack emits a byte-identical `cow_sdk_wasm_bg.wasm` for the `bundler` and
// `nodejs` targets of a flavour — only the JS glue differs in how it loads the
// module. Rather than ship the same binary twice, the nodejs glue is repointed
// at the bundler copy and the redundant nodejs binary is dropped, so each
// flavour ships exactly one wasm binary.
//
// The nodejs glue loads the binary synchronously via `readFileSync` from a
// `${__dirname}/cow_sdk_wasm_bg.wasm` path; the bundler glue uses a bundler
// `import` and keeps its own copy. The rewrite is guarded by strict assertions
// (binaries must be identical; the glue must match the expected load path) so a
// future wasm-pack output change fails the build loudly instead of silently
// shipping a broken nodejs package.

import { existsSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(scriptDir, "..");
const rawRoot = join(packageRoot, "dist", "raw");
const descriptor = JSON.parse(readFileSync(join(packageRoot, "flavours.json"), "utf8"));

const WASM = "cow_sdk_wasm_bg.wasm";
const NODE_GLUE = "cow_sdk_wasm.cjs";
const LOAD_MARKER = "`${__dirname}/cow_sdk_wasm_bg.wasm`";

for (const flavour of descriptor.flavours) {
  const targets = flavour.targets ?? [];
  if (!targets.includes("bundler") || !targets.includes("nodejs")) {
    continue;
  }

  const bundlerWasm = join(rawRoot, `${flavour.name}-bundler`, WASM);
  const nodeWasm = join(rawRoot, `${flavour.name}-nodejs`, WASM);
  const nodeGlue = join(rawRoot, `${flavour.name}-nodejs`, NODE_GLUE);

  for (const required of [bundlerWasm, nodeGlue]) {
    if (!existsSync(required)) {
      throw new Error(`dedupe-target-wasm: ${flavour.name} is missing ${required}`);
    }
  }

  if (!existsSync(nodeWasm)) {
    // Already deduplicated (idempotent re-run) — nothing to do.
    continue;
  }

  if (!readFileSync(bundlerWasm).equals(readFileSync(nodeWasm))) {
    throw new Error(
      `dedupe-target-wasm: ${flavour.name} bundler and nodejs wasm differ; refusing to deduplicate`
    );
  }

  const glue = readFileSync(nodeGlue, "utf8");
  if (!glue.includes(LOAD_MARKER)) {
    throw new Error(
      `dedupe-target-wasm: ${flavour.name} nodejs glue does not match the expected wasm load path; wasm-pack output changed`
    );
  }

  const repointed = `\`\${__dirname}/../${flavour.name}-bundler/${WASM}\``;
  writeFileSync(nodeGlue, glue.replace(LOAD_MARKER, () => repointed));
  rmSync(nodeWasm);
  console.log(`dedupe-target-wasm: ${flavour.name} nodejs now reuses the bundler wasm binary`);
}
