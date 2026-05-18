#!/usr/bin/env bash
# Lint guard that keeps docs/audit/README.md Last reviewed cells in sync with
# the per-audit Last reviewed banners.

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --root)
      if [ "$#" -lt 2 ]; then
        echo "error: --root requires a directory argument" >&2
        exit 2
      fi
      repo_root="$(cd "$2" && pwd)"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

readme="$repo_root/docs/audit/README.md"
fail=0

if [ ! -f "$readme" ]; then
  echo "error: required audit index missing: $readme" >&2
  exit 1
fi

for audit in "$repo_root"/docs/audit/*.md; do
  [ "$(basename "$audit")" = "README.md" ] && continue

  rel_audit="${audit#"$repo_root"/}"
  title="$(grep -m1 -E '^# ' "$audit" | sed -E 's/^# //' || true)"
  if [ -z "$title" ]; then
    printf '::error file=%s::missing top-level audit title\n' "$rel_audit"
    fail=1
    continue
  fi

  banner_date="$(grep -m1 -E '^Last reviewed:[[:space:]]+[0-9]{4}-[0-9]{2}-[0-9]{2}' "$audit" \
    | sed -E 's/^Last reviewed:[[:space:]]+([0-9]{4}-[0-9]{2}-[0-9]{2}).*/\1/' || true)"
  if [ -z "$banner_date" ]; then
    printf '::error file=%s::missing Last reviewed banner for title=%q\n' "$rel_audit" "$title"
    fail=1
    continue
  fi

  index_date="$(awk -F '|' -v title="$title" '
    function trim(value) {
      gsub(/\r/, "", value)
      sub(/^[[:space:]]+/, "", value)
      sub(/[[:space:]]+$/, "", value)
      return value
    }

    /^\|/ {
      first_cell = trim($2)
      if (first_cell ~ /^\[[^]]+\]\([^)]+\)$/) {
        sub(/^\[/, "", first_cell)
        sub(/\]\([^)]+\)$/, "", first_cell)
      }

      if (first_cell == title) {
        print trim($7)
        found = 1
        exit
      }
    }
  ' "$readme")"

  if [ -z "$index_date" ]; then
    printf '::error file=%s::no index row for title=%q\n' "docs/audit/README.md" "$title"
    fail=1
    continue
  fi

  if ! printf '%s\n' "$index_date" | grep -Eq '^[0-9]{4}-[0-9]{2}-[0-9]{2}$'; then
    printf '::error file=%s::index row for title=%q has invalid Last reviewed cell=%q\n' \
      "docs/audit/README.md" "$title" "$index_date"
    fail=1
    continue
  fi

  if [ "$banner_date" != "$index_date" ]; then
    printf '::error file=%s::index_date=%s banner_date=%s for title=%q\n' \
      "docs/audit/README.md" "$index_date" "$banner_date" "$title"
    fail=1
  fi
done

exit "$fail"
