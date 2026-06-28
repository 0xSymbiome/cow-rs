import { existsSync, readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { brotliCompressSync, gzipSync } from "node:zlib";

const scriptDir = dirname(fileURLToPath(import.meta.url));
const packageRoot = join(scriptDir, "..");
const descriptor = JSON.parse(readFileSync(join(packageRoot, "flavours.json"), "utf8"));
const softWarn = process.argv.includes("--soft-warn");
const verifyPerFlavor = process.argv.includes("--verify-per-flavor");
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
  // crates/js/npm/flavours.json — the trading flavour uses gzipFailBytes
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
  if (verifyPerFlavor && (!Array.isArray(flavour.targets) || flavour.targets.length === 0)) {
    console.error(`::error::${flavour.name} does not declare any wasm targets`);
    failed = true;
    continue;
  }

  const rawBudget = bytesFromMiB(flavour.rawBudgetMiB);
  const brotliBudget = bytesFromKiB(flavour.brotliBudgetKiB);
  const gzipBudget = resolveGzipBudget(flavour);

  if (verifyPerFlavor) {
    // Per-flavor three-metric posture: raw + brotli + gzip caps must all be
    // declared so the gate enforces simultaneous limits across the metrics
    // that downstream platforms actually charge for. Missing a gzip cap means
    // the flavor would silently pass at gzip-compressed sizes that exceed
    // platform limits.
    if (typeof flavour.rawBudgetMiB !== "number") {
      console.error(`::error::${flavour.name} is missing rawBudgetMiB`);
      failed = true;
    }
    if (typeof flavour.brotliBudgetKiB !== "number") {
      console.error(`::error::${flavour.name} is missing brotliBudgetKiB`);
      failed = true;
    }
    if (gzipBudget === null) {
      console.error(
        `::error::${flavour.name} is missing a gzip budget (gzipFailBytes or gzipBudgetMiB) — per-flavor three-metric verification requires raw + brotli + gzip caps`
      );
      failed = true;
    }
  }

  for (const target of flavour.targets) {
    let wasmPath = join(
      packageRoot,
      "dist",
      "raw",
      `${flavour.name}-${target}`,
      "cow_sdk_js_bg.wasm"
    );
    if (!existsSync(wasmPath)) {
      // The build deduplicates byte-identical binaries across a flavour's targets
      // (see dedupe-target-wasm.mjs), so a target may reuse a sibling's binary.
      // Measure whichever copy remains — the size is identical by construction.
      const fallback = flavour.targets
        .map((sibling) =>
          join(packageRoot, "dist", "raw", `${flavour.name}-${sibling}`, "cow_sdk_js_bg.wasm")
        )
        .find((candidate) => existsSync(candidate));
      if (!fallback) {
        console.error(`::error::${flavour.name}/${target} wasm binary is missing`);
        failed = true;
        continue;
      }
      wasmPath = fallback;
    }
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
    } else if (verifyPerFlavor) {
      // The per-flavor verifier already reported the missing gzip budget at the
      // top of the loop, but we surface the per-target context here too so the
      // CI summary lists every (flavor, target) pair that lacks the cap.
      console.error(
        `::error::${flavour.name}/${target} gzip size ${formatBytes(gzipBytes)} cannot be checked because no gzip budget is declared`
      );
      failed = true;
    }
  }
}

if (failed) {
  process.exit(1);
}
