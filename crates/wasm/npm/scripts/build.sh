#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../../../.." && pwd)"
npm_root="${repo_root}/crates/wasm/npm"
disposable_lib_reference='/// <reference lib="esnext.disposable" />'

add_disposable_lib_reference() {
  local declaration_file="$1"

  if ! grep -q '\[Symbol\.dispose\]' "${declaration_file}"; then
    return
  fi

  if grep -q 'reference lib="esnext.disposable"' "${declaration_file}"; then
    return
  fi

  local tmp_file="${declaration_file}.tmp"
  {
    printf '%s\n' "${disposable_lib_reference}"
    cat "${declaration_file}"
  } > "${tmp_file}"
  mv "${tmp_file}" "${declaration_file}"
}

cd "${repo_root}"

rm -rf "${npm_root}/dist" "${repo_root}/crates/wasm/pkg"
mkdir -p "${npm_root}/dist"

wasm-pack build crates/wasm --target web --out-dir npm/dist/web --release
wasm-pack build crates/wasm --target bundler --out-dir npm/dist/bundler --release
wasm-pack build crates/wasm --target nodejs --out-dir npm/dist/nodejs --release

if [ -f "${npm_root}/dist/nodejs/cow_sdk_wasm.js" ]; then
  mv "${npm_root}/dist/nodejs/cow_sdk_wasm.js" "${npm_root}/dist/nodejs/cow_sdk_wasm.cjs"
fi

if [ "${BUILD_DENO:-0}" = "1" ]; then
  wasm-pack build crates/wasm --target deno --out-dir npm/dist/deno --release
fi

while IFS= read -r declaration_file; do
  add_disposable_lib_reference "${declaration_file}"
done < <(find "${npm_root}/dist" -name '*.d.ts' -type f)

find "${npm_root}/dist" -name .gitignore -type f -delete
find "${npm_root}/dist" \( -name README.md -o -name package.json \) -type f -delete

node "${npm_root}/scripts/render-package-json.mjs"
node "${npm_root}/scripts/verify-exports.mjs"
