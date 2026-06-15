#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
package_root="$(cd "${script_dir}/.." && pwd)"
tmp_dir="$(mktemp -d)"
tarball=""

cleanup() {
  rm -rf "${tmp_dir}"
  if [ -n "${tarball}" ] && [ -f "${package_root}/${tarball}" ]; then
    rm -f "${package_root}/${tarball}"
  fi
}
trap cleanup EXIT

cd "${package_root}"

if [ ! -f package.json ]; then
  node scripts/render-package-json.mjs
fi

node scripts/verify-exports.mjs

packed_json="$(npm pack --json)"
if command -v jq >/dev/null 2>&1; then
  tarball="$(printf '%s\n' "${packed_json}" | jq -r '.[0].filename')"
  package_name="$(jq -r '.name' package.json)"
else
  tarball="$(PACKED_JSON="${packed_json}" node -e "process.stdout.write(JSON.parse(process.env.PACKED_JSON)[0].filename)")"
  package_name="$(node -e "process.stdout.write(JSON.parse(require('node:fs').readFileSync('package.json', 'utf8')).name)")"
fi

cd "${tmp_dir}"
npm init -y >/dev/null
npm install "${package_root}/${tarball}" >/dev/null

PACKAGE_NAME="${package_name}" FLAVOURS_JSON="${package_root}/flavours.json" node - <<'NODE'
const { createRequire } = require("node:module");
const { readFileSync } = require("node:fs");
const { join } = require("node:path");
const name = process.env.PACKAGE_NAME;
const descriptor = JSON.parse(readFileSync(process.env.FLAVOURS_JSON, "utf8"));
const moduleRequire = createRequire(join(process.cwd(), "resolve.cjs"));
for (const flavour of descriptor.flavours) {
  if (!flavour.targets.includes("nodejs")) {
    continue;
  }
  const subpath = flavour.subpath === "." ? "" : flavour.subpath.slice(1);
  moduleRequire.resolve(`${name}${subpath}`);
}
NODE

PACKAGE_NAME="${package_name}" FLAVOURS_JSON="${package_root}/flavours.json" node --input-type=module - <<'NODE'
import { readFileSync } from "node:fs";

const name = process.env.PACKAGE_NAME;
const descriptor = JSON.parse(readFileSync(process.env.FLAVOURS_JSON, "utf8"));
const subpaths = descriptor.flavours.flatMap((flavour) => {
  const subpath = flavour.subpath === "." ? "" : flavour.subpath.slice(1);
  const rawWasmSubpath = flavour.rawWasmSubpath ? flavour.rawWasmSubpath.slice(1) : null;
  return rawWasmSubpath ? [subpath, rawWasmSubpath] : [subpath];
});
for (const subpath of subpaths) {
  await import.meta.resolve(`${name}${subpath}`);
}
NODE

echo "verify-package-resolution: resolved public package subpaths for ${package_name}"
