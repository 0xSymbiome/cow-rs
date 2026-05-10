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

function checkBudget({ label, actual, budget, flavour, target }) {
  const ratio = actual / budget;
  if (actual > budget) {
    const message = `${flavour}/${target} ${label} ${formatBytes(actual)} exceeds budget ${formatBytes(budget)}`;
    if (softWarn) {
      console.warn(`warning: ${message}`);
    } else {
      console.error(`::error::${message}`);
      failed = true;
    }
    return;
  }

  if (ratio >= warningRatio) {
    console.warn(
      `warning: ${flavour}/${target} ${label} ${formatBytes(actual)} is ${(ratio * 100).toFixed(1)}% of budget ${formatBytes(budget)}`
    );
  }
}

for (const flavour of descriptor.flavours) {
  const rawBudget = bytesFromMiB(flavour.rawBudgetMiB);
  const brotliBudget = bytesFromKiB(flavour.brotliBudgetKiB);
  const gzipBudget = flavour.gzipBudgetMiB ? bytesFromMiB(flavour.gzipBudgetMiB) : null;

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
      )} KiB brotli / ${Math.ceil(gzipBytes / 1024)} KiB gzip`
    );

    checkBudget({
      label: "raw size",
      actual: rawBytes,
      budget: rawBudget,
      flavour: flavour.name,
      target
    });
    checkBudget({
      label: "brotli size",
      actual: brotliBytes,
      budget: brotliBudget,
      flavour: flavour.name,
      target
    });
    if (gzipBudget !== null) {
      checkBudget({
        label: "gzip size",
        actual: gzipBytes,
        budget: gzipBudget,
        flavour: flavour.name,
        target
      });
    }
  }
}

if (failed) {
  process.exit(1);
}
