import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { brotliCompressSync, gzipSync } from "node:zlib";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(scriptDir, "..");
const descriptor = JSON.parse(readFileSync(join(packageRoot, "flavours.json"), "utf8"));
const softWarn = process.argv.includes("--soft-warn");
const warningRatio = 0.95;
let failed = false;

function bytesFromMiB(value) {
  return Math.floor(value * 1024 * 1024);
}

function bytesFromKiB(value) {
  return Math.floor(value * 1024);
}

function formatBytes(bytes) {
  return `${bytes} B`;
}

function checkBudget({ label, actual, failBudget, warnBudget, flavour, target }) {
  if (actual > failBudget) {
    const message = `${flavour}/${target} ${label} ${formatBytes(actual)} exceeds fail budget ${formatBytes(failBudget)}`;
    if (softWarn) {
      console.warn(`warning: ${message}`);
    } else {
      console.error(`::error::${message}`);
      failed = true;
    }
    return;
  }

  const warnThreshold = warnBudget ?? Math.floor(failBudget * warningRatio);
  if (actual >= warnThreshold) {
    const ratio = (actual / failBudget) * 100;
    console.warn(
      `warning: ${flavour}/${target} ${label} ${formatBytes(actual)} is ${ratio.toFixed(1)}% of fail budget ${formatBytes(failBudget)}`
    );
  }
}

function resolveGzipBudget(flavour) {
  // Prefer explicit byte budgets when present (the canonical shape per
  // crates/wasm/npm/flavours.json — cloudflare flavor uses gzipFailBytes
  // + gzipWarnBytes to track Cloudflare Workers' published 3 MB
  // compressed-size limit without MiB / MB ambiguity).
  if (typeof flavour.gzipFailBytes === "number") {
    return {
      failBudget: flavour.gzipFailBytes,
      warnBudget: typeof flavour.gzipWarnBytes === "number"
        ? flavour.gzipWarnBytes
        : null
    };
  }
  // Backward-compatible MiB-based fallback for flavors that have not
  // migrated to the byte budget yet.
  if (typeof flavour.gzipBudgetMiB === "number") {
    return { failBudget: bytesFromMiB(flavour.gzipBudgetMiB), warnBudget: null };
  }
  return null;
}

for (const flavour of descriptor.flavours) {
  const rawBudget = bytesFromMiB(flavour.rawBudgetMiB);
  const brotliBudget = bytesFromKiB(flavour.brotliBudgetKiB);
  const gzipBudget = resolveGzipBudget(flavour);

  for (const target of flavour.targets) {
    const wasmPath = join(
      packageRoot,
      "dist",
      "raw",
      `${flavour.name}-${target}`,
      "cow_sdk_wasm_bg.wasm"
    );
    const bytes = readFileSync(wasmPath);
    const rawBytes = bytes.length;
    const brotliBytes = brotliCompressSync(bytes).length;
    const gzipBytes = gzipSync(bytes).length;

    console.log(
      `${flavour.name}/${target}: ${(rawBytes / 1024 / 1024).toFixed(2)} MiB raw / ${Math.ceil(
        brotliBytes / 1024
      )} KiB brotli / ${gzipBytes} B gzip`
    );

    checkBudget({
      label: "raw size",
      actual: rawBytes,
      failBudget: rawBudget,
      warnBudget: null,
      flavour: flavour.name,
      target
    });
    checkBudget({
      label: "brotli size",
      actual: brotliBytes,
      failBudget: brotliBudget,
      warnBudget: null,
      flavour: flavour.name,
      target
    });
    if (gzipBudget !== null) {
      checkBudget({
        label: "gzip size",
        actual: gzipBytes,
        failBudget: gzipBudget.failBudget,
        warnBudget: gzipBudget.warnBudget,
        flavour: flavour.name,
        target
      });
    }
  }
}

if (failed) {
  process.exit(1);
}
