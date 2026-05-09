#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../../../.." && pwd)"
npm_root="${repo_root}/crates/wasm/npm"
disposable_lib_reference='/// <reference lib="esnext.disposable" />'
wasm_opt_flags=(
  -Oz
  --enable-bulk-memory
  --enable-sign-ext
  --strip-debug
  --strip-producers
  --vacuum
  --merge-blocks
  --simplify-locals
  --enable-nontrapping-float-to-int
  --enable-simd
)

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

optimize_wasm_output() {
  local out_dir="$1"
  local wasm_file="${out_dir}/cow_sdk_wasm_bg.wasm"
  local optimized_file="${wasm_file}.opt"

  if [ ! -f "${wasm_file}" ]; then
    printf 'expected wasm-pack output missing: %s\n' "${wasm_file}" >&2
    exit 1
  fi

  wasm-opt "${wasm_opt_flags[@]}" "${wasm_file}" -o "${optimized_file}"
  mv "${optimized_file}" "${wasm_file}"
}

run_wasm_pack() {
  local target="$1"
  local out_dir="$2"

  wasm-pack build crates/wasm --target "${target}" --out-dir "${out_dir}" --release
  optimize_wasm_output "${repo_root}/crates/wasm/${out_dir}"
}

cd "${repo_root}"

rm -rf "${npm_root}/dist" "${repo_root}/crates/wasm/pkg"
mkdir -p "${npm_root}/dist"

run_wasm_pack web npm/dist/web
run_wasm_pack bundler npm/dist/bundler
run_wasm_pack nodejs npm/dist/nodejs

if [ -f "${npm_root}/dist/nodejs/cow_sdk_wasm.js" ]; then
  mv "${npm_root}/dist/nodejs/cow_sdk_wasm.js" "${npm_root}/dist/nodejs/cow_sdk_wasm.cjs"
fi

if [ "${BUILD_DENO:-0}" = "1" ]; then
  run_wasm_pack deno npm/dist/deno
fi

while IFS= read -r declaration_file; do
  add_disposable_lib_reference "${declaration_file}"
done < <(find "${npm_root}/dist" -name '*.d.ts' -type f)

find "${npm_root}/dist" -name .gitignore -type f -delete
find "${npm_root}/dist" \( -name README.md -o -name package.json \) -type f -delete

node "${npm_root}/scripts/render-package-json.mjs"
node "${npm_root}/scripts/verify-exports.mjs"
