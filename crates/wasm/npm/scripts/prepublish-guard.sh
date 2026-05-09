#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
package_root="$(cd "${script_dir}/.." && pwd)"

cd "${package_root}"

if [ ! -f package.json ]; then
  echo "prepublish-guard: package.json has not been rendered" >&2
  exit 1
fi

if command -v jq >/dev/null 2>&1; then
  package_name="$(jq -r '.name' package.json)"
else
  package_name="$(node -e "process.stdout.write(JSON.parse(require('node:fs').readFileSync('package.json', 'utf8')).name)")"
fi

if [ "${package_name}" = "cow-sdk-wasm-placeholder" ] && [ "${ALLOW_PLACEHOLDER_NPM_PUBLISH:-0}" != "1" ]; then
  echo "prepublish-guard: refusing to publish placeholder package name" >&2
  exit 1
fi

echo "prepublish-guard: package name is publishable"
