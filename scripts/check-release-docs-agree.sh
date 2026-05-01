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
#        - CONTRIBUTING.md
#        - PROPERTIES.md
#
#   2. The cargo audit RustSec `--ignore RUSTSEC-...` token list in docs
#      must match the canonical advisory tolerance register in:
#        - .github/config/deny.toml
#
#      Documentation sites checked:
#        - docs/release-checklist.md
#        - docs/verification-matrix.md
#
#   3. The Playwright install browser arguments for the browser-wallet
#      lane must be identical across:
#        - docs/release-checklist.md
#        - .github/workflows/browser-wallet-e2e.yml
#
# Exit 0 on agreement; exit 1 with a unified diff or inline error
# message on disagreement.

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --root)
      if [ "$#" -lt 2 ]; then
        echo "error: --root requires a directory argument" >&2
        exit 2
      fi
      repo_root="$2"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

release_checklist="$repo_root/docs/release-checklist.md"
verification_matrix="$repo_root/docs/verification-matrix.md"
quality_gate="$repo_root/.github/workflows/_quality-gate.yml"
deny_config="$repo_root/.github/config/deny.toml"
browser_wallet_workflow="$repo_root/.github/workflows/browser-wallet-e2e.yml"
contributing_md="$repo_root/CONTRIBUTING.md"
properties_md="$repo_root/PROPERTIES.md"
dependency_gate_audit="$repo_root/docs/audit/dependency-gate-audit.md"

for file in "$release_checklist" "$verification_matrix" \
            "$quality_gate" "$deny_config" "$browser_wallet_workflow" \
            "$contributing_md" "$properties_md" "$dependency_gate_audit"; do
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
    | sort -u \
    || true
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
    | sort -u \
    || true
}

release_checklist_pkgs="$(extract_pkgs_single_line "$release_checklist")"
verification_matrix_pkgs="$(extract_pkgs_single_line "$verification_matrix")"
quality_gate_pkgs="$(extract_pkgs_multi_line "$quality_gate")"
contributing_pkgs="$(extract_pkgs_single_line "$contributing_md")"
properties_pkgs="$(extract_pkgs_single_line "$properties_md")"

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
if [ -z "$contributing_pkgs" ]; then
  echo "error: CONTRIBUTING.md does not declare the cargo tree alloy-provider package list" >&2
  exit 1
fi
if [ -z "$properties_pkgs" ]; then
  echo "error: PROPERTIES.md does not declare the cargo tree alloy-provider package list" >&2
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

diff_or_fail "docs/release-checklist.md" "CONTRIBUTING.md" \
  "$release_checklist_pkgs" "$contributing_pkgs"

diff_or_fail "docs/release-checklist.md" "PROPERTIES.md" \
  "$release_checklist_pkgs" "$properties_pkgs"

extract_cargo_audit_cmd() {
  # Extract the RustSec ignore-token list from the first cargo-audit command.
  # The range capture supports both single-line markdown commands and
  # multi-line shell blocks that continue with trailing backslashes.
  awk '
    /cargo audit --deny unsound/ { capture = 1 }
    capture { print }
    capture && /2>&1|exit 1/ { capture = 0; next }
    capture && /cargo audit --deny unsound/ && $0 !~ /\\$/ { capture = 0; next }
    capture && /--ignore RUSTSEC-[0-9]{4}-[0-9]{4}/ && $0 !~ /\\$/ { capture = 0 }
  ' "$1" \
    | grep -oE -- '--ignore RUSTSEC-[0-9]{4}-[0-9]{4}' \
    | awk '{ print $2 }' \
    | tr -d '\r' \
    | sort -u \
    || true
}

extract_canonical_audit_ignores() {
  python - "$1" <<'PY' | tr -d '\r'
import sys
import tomllib
from pathlib import Path

config = tomllib.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
ids = []
for entry in config.get("advisories", {}).get("ignore", []):
    ids.append(entry["id"] if isinstance(entry, dict) else entry)
for advisory_id in sorted(set(ids)):
    print(advisory_id)
PY
}

audit_checklist_tokens="$(extract_cargo_audit_cmd "$release_checklist")"
audit_matrix_tokens="$(extract_cargo_audit_cmd "$verification_matrix")"
audit_canonical_tokens="$(extract_canonical_audit_ignores "$deny_config")"

if [ -z "$audit_checklist_tokens" ]; then
  echo "error: docs/release-checklist.md does not declare the cargo audit ignore-token list" >&2
  exit 1
fi
if [ -z "$audit_matrix_tokens" ]; then
  echo "error: docs/verification-matrix.md does not declare the cargo audit ignore-token list" >&2
  exit 1
fi
if [ -z "$audit_canonical_tokens" ]; then
  echo "error: .github/config/deny.toml does not declare the canonical cargo audit ignore-token list" >&2
  exit 1
fi

diff_audit_or_fail() {
  label_a="$1"
  label_b="$2"
  content_a="$3"
  content_b="$4"
  if [ "$content_a" = "$content_b" ]; then
    return 0
  fi
  echo "error: $label_a and $label_b disagree on the cargo audit RUSTSEC ignore-token list" >&2
  diff -u <(printf '%s\n' "$content_a") <(printf '%s\n' "$content_b") >&2 || true
  exit 1
}

diff_audit_or_fail "docs/release-checklist.md" "docs/verification-matrix.md" \
  "$audit_checklist_tokens" "$audit_matrix_tokens"

diff_audit_or_fail "docs/release-checklist.md" ".github/config/deny.toml" \
  "$audit_checklist_tokens" "$audit_canonical_tokens"

diff_audit_or_fail "docs/verification-matrix.md" ".github/config/deny.toml" \
  "$audit_matrix_tokens" "$audit_canonical_tokens"

while IFS= read -r advisory_id; do
  if [ -n "$advisory_id" ] && ! grep -qF "$advisory_id" "$dependency_gate_audit"; then
    echo "error: docs/audit/dependency-gate-audit.md does not document cargo audit ignore token $advisory_id" >&2
    exit 1
  fi
done < <(printf '%s\n' "$audit_canonical_tokens")

extract_browser_wallet_playwright_browsers() {
  # Capture the trailing browser-set arguments from the playwright install
  # line that targets the browser-wallet lane (matched on the e2e/browser-wallet
  # working directory). Strips through `playwright install` and any
  # leading/trailing whitespace.
  grep -E 'playwright install' "$1" \
    | grep -E 'e2e/browser-wallet' \
    | sed -E 's/^.*playwright install[[:space:]]+//' \
    | sed -E 's/[[:space:]]+$//' \
    | head -n 1 \
    || true
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

# ---- WASM console dual-authority acknowledgement ----
for html in "$repo_root/examples/wasm"/*/index.html; do
  if [ -f "$html" ]; then
    if ! head -n 40 "$html" | grep -qF "dual-authority acknowledgement"; then
      echo "error: $html is missing the dual-authority acknowledgement sentinel in its first 40 lines" >&2
      exit 1
    fi
  fi
done

if [ -f "$repo_root/scripts/policy-maintainer/Cargo.toml" ]; then
  cargo run --quiet --manifest-path "$repo_root/scripts/policy-maintainer/Cargo.toml" -- \
    check-property-citations --repo-root "$repo_root"
fi

echo "Release-gate commands agree across docs and CI."
exit 0
