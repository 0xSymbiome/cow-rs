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
  ["cloudflare", "cloudflare"]
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
  if (flavourName === "cloudflare") {
    return content;
  }
  return content
    .replaceAll(`${flavourName}-bundler/cow_sdk_wasm.js`, `${flavourName}-nodejs/cow_sdk_wasm.cjs`)
    .replaceAll(`${flavourName}-bundler/cow_sdk_wasm.cjs`, `${flavourName}-nodejs/cow_sdk_wasm.cjs`)
    .replaceAll(`${flavourName}-web/cow_sdk_wasm.js`, `${flavourName}-nodejs/cow_sdk_wasm.cjs`)
    .replaceAll(`${flavourName}-web/cow_sdk_wasm.cjs`, `${flavourName}-nodejs/cow_sdk_wasm.cjs`);
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
}

rmSync(esmOut, { recursive: true, force: true });
rmSync(cjsOut, { recursive: true, force: true });
JS

node "${package_root}/scripts/render-package-json.mjs"
