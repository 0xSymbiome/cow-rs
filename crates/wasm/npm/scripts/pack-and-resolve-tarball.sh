#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
package_root="$(cd "${script_dir}/.." && pwd)"

cd "${package_root}"

if [ ! -f package.json ]; then
  node scripts/render-package-json.mjs
fi

node scripts/verify-exports.mjs

packed_json="$(npm pack --json)"
if command -v jq >/dev/null 2>&1; then
  tarball="$(printf '%s\n' "${packed_json}" | jq -r '.[0].filename')"
else
  tarball="$(PACKED_JSON="${packed_json}" node -e "process.stdout.write(JSON.parse(process.env.PACKED_JSON)[0].filename)")"
fi

printf '%s\n' "${package_root}/${tarball}"
