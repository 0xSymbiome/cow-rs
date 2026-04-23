#!/usr/bin/env bash
set -euo pipefail

SCRIPT="$(cd "$(dirname "$0")/.." && pwd)/scripts/check-release-docs-agree.sh"
FIXTURES="$(cd "$(dirname "$0")/.." && pwd)/scripts/fixtures/release-gates-self-test"

if ! "$SCRIPT" --root "$FIXTURES/matching" > /dev/null; then
  echo "SELF-TEST FAIL: matching fixture should exit 0"
  exit 1
fi

for drift_dir in cargo-tree-disagreement \
                 cargo-audit-disagreement \
                 playwright-disagreement; do
  if "$SCRIPT" --root "$FIXTURES/drifted-$drift_dir" > /dev/null 2>&1; then
    echo "SELF-TEST FAIL: drifted-$drift_dir should exit non-zero"
    exit 1
  fi
done

if "$SCRIPT" --root "$FIXTURES/drifted-footer-acknowledgement" > /dev/null 2>&1; then
  echo "SELF-TEST FAIL: drifted-footer-acknowledgement should exit non-zero"
  exit 1
fi

echo "SELF-TEST PASS"
