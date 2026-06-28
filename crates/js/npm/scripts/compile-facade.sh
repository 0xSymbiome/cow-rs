#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
package_root="$(cd "${script_dir}/.." && pwd)"
dist_root="${package_root}/dist"
esm_out="${dist_root}/.facade-esm"
cjs_out="${dist_root}/.facade-cjs"

run_tsc() {
  if [ -x "${package_root}/node_modules/.bin/tsc" ]; then
    "${package_root}/node_modules/.bin/tsc" "$@"
  else
    npm exec --yes --package=typescript -- tsc "$@"
  fi
}

if [ ! -f "${package_root}/flavours.json" ]; then
  printf 'missing flavours.json\n' >&2
  exit 1
fi

if [ ! -d "${dist_root}/raw" ]; then
  printf 'missing dist/raw; run scripts/build.sh before compiling the facade\n' >&2
  exit 1
fi

rm -rf "${esm_out}" "${cjs_out}"

cd "${package_root}"
run_tsc --project tsconfig.facade.json --outDir "${esm_out}" --module ES2022 --moduleResolution Bundler
run_tsc \
  --project tsconfig.facade.json \
  --outDir "${cjs_out}" \
  --module CommonJS \
  --moduleResolution Node \
  --ignoreDeprecations 6.0 \
  --declaration false \
  --verbatimModuleSyntax false

node --input-type=module - "${package_root}" <<'JS'
import { cpSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from "node:fs";
import { dirname, extname, join, relative } from "node:path";

const [, , packageRoot] = process.argv;
const distRoot = join(packageRoot, "dist");
const esmOut = join(distRoot, ".facade-esm");
const cjsOut = join(distRoot, ".facade-cjs");
const descriptor = JSON.parse(readFileSync(join(packageRoot, "flavours.json"), "utf8"));
const entries = new Map([
  ["default", "default"],
  ["orderbook", "orderbook"],
  ["signing", "signing"],
  ["trading", "trading"]
]);

function walk(directory, output = []) {
  for (const entry of readdirSync(directory, { withFileTypes: true })) {
    const path = join(directory, entry.name);
    if (entry.isDirectory()) {
      walk(path, output);
    } else {
      output.push(path);
    }
  }
  return output;
}

function copyTree(source, destination, transformName = (name) => name) {
  for (const file of walk(source)) {
    const relativePath = relative(source, file);
    const parts = relativePath.split(/[\\/]/);
    parts[parts.length - 1] = transformName(parts[parts.length - 1]);
    const target = join(destination, ...parts);
    mkdirSync(dirname(target), { recursive: true });
    cpSync(file, target);
  }
}

function replaceInFiles(directory, replacer) {
  for (const file of walk(directory)) {
    if (!file.endsWith(".d.ts") && ![".js", ".mjs", ".cjs"].includes(extname(file))) {
      continue;
    }
    const before = readFileSync(file, "utf8");
    const after = replacer(before, file);
    if (after !== before) {
      writeFileSync(file, after);
    }
  }
}

function rewriteEsm(content) {
  return content
    .replaceAll("../../dist/raw/", "../../raw/")
    .replaceAll('"./index.js"', '"./index.mjs"');
}

function rewriteCjs(content) {
  return content
    .replaceAll("../../dist/raw/", "../../raw/")
    .replace(/require\("(\.{1,2}\/[^"]+)\.js"\)/g, 'require("$1.cjs")');
}

function rewriteNodeTarget(content, flavourName) {
  // The CommonJS entry loads the nodejs target's glue instead of the bundler
  // glue. Flavours without a nodejs target never reach this (they ship no CJS
  // entry), so the rewrite is unconditional here.
  return content
    .replaceAll(`${flavourName}-bundler/cow_sdk_js.js`, `${flavourName}-nodejs/cow_sdk_js.cjs`)
    .replaceAll(`${flavourName}-bundler/cow_sdk_js.cjs`, `${flavourName}-nodejs/cow_sdk_js.cjs`)
    .replaceAll(`${flavourName}-web/cow_sdk_js.js`, `${flavourName}-nodejs/cow_sdk_js.cjs`)
    .replaceAll(`${flavourName}-web/cow_sdk_js.cjs`, `${flavourName}-nodejs/cow_sdk_js.cjs`);
}

for (const flavour of descriptor.flavours) {
  const entry = entries.get(flavour.name);
  if (!entry) {
    throw new Error(`no facade entry configured for ${flavour.name}`);
  }

  const targetDir = join(distRoot, flavour.name);
  rmSync(targetDir, { recursive: true, force: true });
  mkdirSync(targetDir, { recursive: true });

  copyTree(esmOut, targetDir);
  replaceInFiles(targetDir, (content) => rewriteEsm(content));

  const entryJs = join(targetDir, `${entry}.js`);
  const entryDts = join(targetDir, `${entry}.d.ts`);
  if (!existsSync(entryJs) || !existsSync(entryDts)) {
    throw new Error(`compiled facade entry missing for ${flavour.name}`);
  }
  cpSync(entryJs, join(targetDir, "index.mjs"));
  cpSync(entryDts, join(targetDir, "index.d.ts"));

  copyTree(cjsOut, targetDir, (name) => name.endsWith(".js") ? name.replace(/\.js$/, ".cjs") : name);
  replaceInFiles(targetDir, (content, file) => {
    let next = extname(file) === ".cjs" ? rewriteCjs(content) : content;
    if (extname(file) === ".cjs") {
      next = rewriteNodeTarget(next, flavour.name);
    }
    return next;
  });

  const entryCjs = join(targetDir, `${entry}.cjs`);
  if (!existsSync(entryCjs)) {
    throw new Error(`compiled CommonJS facade entry missing for ${flavour.name}`);
  }
  cpSync(entryCjs, join(targetDir, "index.cjs"));

  // A flavour that also ships the standalone `web` target gets a second ESM entry
  // (`edge.mjs`) bound to a generated web raw shim (`raw/<flavour>-web`), exposed at
  // its `webSubpath` (for example `./trading/edge`). The web raw shim differs from
  // the bundler shim only in its loader, so it is generated from the compiled
  // bundler shim — importing the wasm-bindgen `init` and wiring `initializeRaw` to
  // it (replacing the bundler shim's no-op) against the `<flavour>-web` glue —
  // keeping the per-flavour export surface single-sourced in the bundler shim. The
  // facade entry is identical across targets, so `edge.mjs` reuses the bundler
  // entry's compiled output with the raw import swapped; the redundant web shim CJS
  // is dropped (the web build is ESM-only).
  if (Array.isArray(flavour.targets) && flavour.targets.includes("web") && flavour.targets.includes("bundler")) {
    const toWeb = (text) =>
      text
        .replaceAll(`${flavour.name}-bundler`, `${flavour.name}-web`)
        .replace("import * as wasm from", "import init, * as wasm from");
    const webRawJs = toWeb(readFileSync(join(targetDir, "raw", `${flavour.name}.js`), "utf8")).replace(
      /export const initializeRaw = async \(_input\) => \{\s*\};/,
      "export const initializeRaw = init;"
    );
    writeFileSync(join(targetDir, "raw", `${flavour.name}-web.js`), webRawJs);
    const webRawDts = toWeb(readFileSync(join(targetDir, "raw", `${flavour.name}.d.ts`), "utf8"))
      .replace("export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;\n", "")
      .replace(
        /export declare const initializeRaw: \(_input\?: \{\s*module_or_path\?: unknown;\s*\}\) => Promise<void>;/,
        "export declare const initializeRaw: typeof init;"
      );
    writeFileSync(join(targetDir, "raw", `${flavour.name}-web.d.ts`), webRawDts);

    const webEntry = readFileSync(join(targetDir, "index.mjs"), "utf8").replaceAll(
      `./raw/${flavour.name}.js`,
      `./raw/${flavour.name}-web.js`
    );
    writeFileSync(join(targetDir, "edge.mjs"), webEntry);
    cpSync(join(targetDir, "index.d.ts"), join(targetDir, "edge.d.ts"));
    rmSync(join(targetDir, "raw", `${flavour.name}-web.cjs`), { force: true });
  }

  // A flavour with a `moduleSubpath` gets a source-phase `module` entry
  // (`module.mjs`). The module target auto-instantiates synchronously like the
  // bundler target, so its raw shim is identical to the bundler shim but for the
  // glue directory — generate it from the compiled bundler shim rather than
  // committing a duplicate source file. The entry reuses the bundler entry's
  // compiled output with the raw import swapped, and its declaration is the same
  // public surface.
  if (flavour.moduleSubpath) {
    const moduleRawJs = readFileSync(join(targetDir, "raw", `${flavour.name}.js`), "utf8")
      .replaceAll(`${flavour.name}-bundler`, `${flavour.name}-module`);
    writeFileSync(join(targetDir, "raw", `${flavour.name}-module.js`), moduleRawJs);
    const moduleRawDts = readFileSync(join(targetDir, "raw", `${flavour.name}.d.ts`), "utf8")
      .replaceAll(`${flavour.name}-bundler`, `${flavour.name}-module`);
    writeFileSync(join(targetDir, "raw", `${flavour.name}-module.d.ts`), moduleRawDts);
    const moduleEntry = readFileSync(join(targetDir, "index.mjs"), "utf8").replaceAll(
      `./raw/${flavour.name}.js`,
      `./raw/${flavour.name}-module.js`
    );
    writeFileSync(join(targetDir, "module.mjs"), moduleEntry);
    cpSync(join(targetDir, "index.d.ts"), join(targetDir, "module.d.ts"));
  }

  // For a flavour whose browser/import/default conditions resolve to the web build
  // (`edge.mjs`), the bundler ESM facade entry (`index.mjs`) is unreferenced by any
  // package export — `index.cjs` backs the node condition and `index.d.ts` the
  // types. Drop it so the package ships no dead entry; the `edge.mjs` and
  // `module.mjs` entries above were already generated from it.
  if (flavour.targets.includes("web")) {
    rmSync(join(targetDir, "index.mjs"), { force: true });
  }

  // Prune to the reachable closure. Each flavor is published as index.* and only
  // pulls in the shared modules (internal/errors/options/callbacks/envelope) and
  // its own raw/<flavor>; the copied sibling-flavor modules and the orphaned
  // <flavor>.* duplicate never load, so drop them rather than ship dead weight.
  for (const name of descriptor.flavours.map((f) => f.name)) {
    for (const ext of [".js", ".cjs", ".d.ts"]) {
      rmSync(join(targetDir, `${name}${ext}`), { force: true });
      if (name !== flavour.name) {
        rmSync(join(targetDir, "raw", `${name}${ext}`), { force: true });
      }
    }
  }
}

rmSync(esmOut, { recursive: true, force: true });
rmSync(cjsOut, { recursive: true, force: true });
JS

# The facade re-declares [Symbol.dispose] on its client classes. wasm-bindgen
# adds the matching lib reference to its own raw declarations; mirror it on the
# compiled facade declarations so the member resolves under consumer tsconfig
# libs that predate the Disposable types. The web `edge.d.ts` and source-phase
# `module.d.ts` entries need the same reference.
for facade_dts in "${dist_root}"/*/index.d.ts "${dist_root}"/*/edge.d.ts "${dist_root}"/*/module.d.ts; do
  [ -f "${facade_dts}" ] || continue
  if grep -q '\[Symbol\.dispose\]' "${facade_dts}" \
    && ! grep -q 'reference lib="esnext.disposable"' "${facade_dts}"; then
    { printf '/// <reference lib="esnext.disposable" />\n'; cat "${facade_dts}"; } > "${facade_dts}.tmp"
    mv "${facade_dts}.tmp" "${facade_dts}"
  fi
done

node "${package_root}/scripts/render-package-json.mjs"
