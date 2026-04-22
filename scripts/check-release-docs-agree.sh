#!/usr/bin/env bash
# scripts/check-release-docs-agree.sh
#
# Lint guard that keeps the published release-gate commands in agreement
# across the docs and CI sites so any future drift is caught at the
# workspace level instead of silently lingering.
#
# Comparisons performed:
#
#   1. The cargo tree alloy-provider invariant `-p` package list must be
#      identical across:
#        - docs/release-checklist.md
#        - docs/verification-matrix.md
#        - .github/workflows/_quality-gate.yml
#
#   2. The Playwright install browser arguments for the browser-wallet
#      lane must be identical across:
#        - docs/release-checklist.md
#        - .github/workflows/browser-wallet-e2e.yml
#
# Exit 0 on agreement; exit 1 with a unified diff or inline error
# message on disagreement.

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

release_checklist="$repo_root/docs/release-checklist.md"
verification_matrix="$repo_root/docs/verification-matrix.md"
quality_gate="$repo_root/.github/workflows/_quality-gate.yml"
browser_wallet_workflow="$repo_root/.github/workflows/browser-wallet-e2e.yml"

for file in "$release_checklist" "$verification_matrix" \
            "$quality_gate" "$browser_wallet_workflow"; do
  if [ ! -f "$file" ]; then
    echo "error: required source file missing: $file" >&2
    exit 1
  fi
done

extract_pkgs_single_line() {
  # Extract `-p NAME` package tokens from the single line that carries the
  # cargo tree alloy-provider marker. Used for the markdown sites.
  grep -E 'cargo tree --invert alloy-provider' "$1" \
    | grep -oE -- '-p [A-Za-z][A-Za-z0-9_-]*' \
    | awk '{ print $2 }' \
    | sort -u
}

extract_pkgs_multi_line() {
  # Extract `-p NAME` package tokens from the multi-line shell block
  # rendered by the workflow file. Captures starting at the cargo tree
  # marker line and stops once the redirect (`2>&1`) is reached.
  awk '
    /cargo tree --invert alloy-provider/ { capture = 1 }
    capture { print }
    capture && /2>&1/ { capture = 0 }
  ' "$1" \
    | grep -oE -- '-p [A-Za-z][A-Za-z0-9_-]*' \
    | awk '{ print $2 }' \
    | sort -u
}

release_checklist_pkgs="$(extract_pkgs_single_line "$release_checklist")"
verification_matrix_pkgs="$(extract_pkgs_single_line "$verification_matrix")"
quality_gate_pkgs="$(extract_pkgs_multi_line "$quality_gate")"

if [ -z "$release_checklist_pkgs" ]; then
  echo "error: docs/release-checklist.md does not declare the cargo tree alloy-provider package list" >&2
  exit 1
fi
if [ -z "$verification_matrix_pkgs" ]; then
  echo "error: docs/verification-matrix.md does not declare the cargo tree alloy-provider package list" >&2
  exit 1
fi
if [ -z "$quality_gate_pkgs" ]; then
  echo "error: .github/workflows/_quality-gate.yml does not declare the cargo tree alloy-provider package list" >&2
  exit 1
fi

diff_or_fail() {
  label_a="$1"
  label_b="$2"
  content_a="$3"
  content_b="$4"
  if [ "$content_a" = "$content_b" ]; then
    return 0
  fi
  echo "error: $label_a and $label_b disagree on the cargo tree alloy-provider package list" >&2
  diff -u <(printf '%s\n' "$content_a") <(printf '%s\n' "$content_b") >&2 || true
  exit 1
}

diff_or_fail "docs/release-checklist.md" "docs/verification-matrix.md" \
  "$release_checklist_pkgs" "$verification_matrix_pkgs"

diff_or_fail "docs/release-checklist.md" ".github/workflows/_quality-gate.yml" \
  "$release_checklist_pkgs" "$quality_gate_pkgs"

diff_or_fail "docs/verification-matrix.md" ".github/workflows/_quality-gate.yml" \
  "$verification_matrix_pkgs" "$quality_gate_pkgs"

extract_browser_wallet_playwright_browsers() {
  # Capture the trailing browser-set arguments from the playwright install
  # line that targets the browser-wallet lane (matched on the e2e/browser-wallet
  # working directory). Strips through `playwright install` and any
  # leading/trailing whitespace.
  grep -E 'playwright install' "$1" \
    | grep -E 'e2e/browser-wallet' \
    | sed -E 's/^.*playwright install[[:space:]]+//' \
    | sed -E 's/[[:space:]]+$//' \
    | head -n 1
}

release_checklist_playwright="$(extract_browser_wallet_playwright_browsers "$release_checklist")"
workflow_playwright="$(extract_browser_wallet_playwright_browsers "$browser_wallet_workflow")"

if [ -z "$release_checklist_playwright" ]; then
  echo "error: docs/release-checklist.md does not declare a playwright install line for the browser-wallet lane" >&2
  exit 1
fi

if [ -z "$workflow_playwright" ]; then
  echo "error: .github/workflows/browser-wallet-e2e.yml does not declare a playwright install line for the browser-wallet lane" >&2
  exit 1
fi

if [ "$release_checklist_playwright" != "$workflow_playwright" ]; then
  echo "error: docs/release-checklist.md and .github/workflows/browser-wallet-e2e.yml disagree on the browser-wallet playwright install browser set" >&2
  printf '  release-checklist.md: %s\n' "$release_checklist_playwright" >&2
  printf '  browser-wallet-e2e.yml: %s\n' "$workflow_playwright" >&2
  exit 1
fi

echo "Release-gate commands agree across docs and CI."
exit 0
