#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "${script_dir}/../../../.." && pwd)"
npm_root="${repo_root}/crates/wasm/npm"
flavours_json="${npm_root}/flavours.json"
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
wasm_opt_cmd=()

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

  "${wasm_opt_cmd[@]}" "${wasm_opt_flags[@]}" "${wasm_file}" -o "${optimized_file}"
  mv "${optimized_file}" "${wasm_file}"
}

run_wasm_pack() {
  local flavour="$1"
  local target="$2"
  local features="$3"
  local out_dir="npm/dist/raw/${flavour}-${target}"

  rm -rf "${repo_root}/crates/wasm/${out_dir}"

  local args=(
    crates/wasm
    --target "${target}"
    --out-dir "${out_dir}"
    --release
    --no-default-features
    --features "${features}"
  )
  wasm-pack build "${args[@]}"
  optimize_wasm_output "${repo_root}/crates/wasm/${out_dir}"

  if [ "${target}" = "nodejs" ] && [ -f "${repo_root}/crates/wasm/${out_dir}/cow_sdk_wasm.js" ]; then
    mv \
      "${repo_root}/crates/wasm/${out_dir}/cow_sdk_wasm.js" \
      "${repo_root}/crates/wasm/${out_dir}/cow_sdk_wasm.cjs"
  fi
}

if ! command -v wasm-pack >/dev/null 2>&1; then
  printf 'wasm-pack is required to build wasm flavours\n' >&2
  exit 1
fi

if command -v wasm-opt >/dev/null 2>&1; then
  wasm_opt_cmd=(wasm-opt)
elif command -v npm >/dev/null 2>&1; then
  wasm_opt_cmd=(npm exec --yes --package=binaryen wasm-opt --)
else
  printf 'wasm-opt is required to optimize wasm flavours\n' >&2
  exit 1
fi

cd "${repo_root}"

filter_flavour="${WASM_FLAVOUR:-${WASM_FLAVOR:-}}"
filter_target="${WASM_TARGET:-}"

if [ -z "${filter_flavour}" ] && [ -z "${filter_target}" ]; then
  rm -rf "${npm_root}/dist" "${repo_root}/crates/wasm/pkg"
else
  rm -rf "${repo_root}/crates/wasm/pkg"
  mkdir -p "${npm_root}/dist/raw"
fi
mkdir -p "${npm_root}/dist/raw"

mapfile -t matrix < <(
  node --input-type=module - "${flavours_json}" "${filter_flavour}" "${filter_target}" <<'JS'
import { readFileSync } from "node:fs";

const [, , flavoursPath, flavourFilter, targetFilter] = process.argv;
const descriptor = JSON.parse(readFileSync(flavoursPath, "utf8"));
const rows = [];

for (const flavour of descriptor.flavours) {
  if (flavourFilter && flavour.name !== flavourFilter) {
    continue;
  }
  for (const target of flavour.targets) {
    if (targetFilter && target !== targetFilter) {
      continue;
    }
    rows.push([flavour.name, target, flavour.features.join(",")].join("\t"));
  }
}

if (rows.length === 0) {
  console.error("no wasm flavour targets matched the requested filters");
  process.exit(2);
}

console.log(rows.join("\n"));
JS
)

for row in "${matrix[@]}"; do
  IFS=$'\t' read -r flavour target features <<< "${row}"
  printf 'building wasm flavour %s for %s\n' "${flavour}" "${target}"
  run_wasm_pack "${flavour}" "${target}" "${features}"
done

while IFS= read -r declaration_file; do
  add_disposable_lib_reference "${declaration_file}"
done < <(find "${npm_root}/dist/raw" -name '*.d.ts' -type f)

find "${npm_root}/dist/raw" -name .gitignore -type f -delete
find "${npm_root}/dist/raw" \( -name README.md -o -name package.json \) -type f -delete

if [ -z "${filter_flavour}" ] && [ -z "${filter_target}" ]; then
  bash "${npm_root}/scripts/compile-facade.sh"
  node "${npm_root}/scripts/verify-exports.mjs"
  node "${npm_root}/scripts/verify-no-raw-exports.mjs"
  node "${npm_root}/scripts/verify-facade-denylist.mjs"
  node "${npm_root}/scripts/measure-wasm-size.mjs"
fi
