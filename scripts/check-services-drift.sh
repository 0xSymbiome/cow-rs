#!/usr/bin/env bash
# scripts/check-services-drift.sh
#
# Report-only detector for upstream services API drift. It compares the
# services errorType set and selected model DTO fields against the typed
# orderbook surface in this checkout, then emits a Markdown summary for CI.
#
# Dependencies: bash, git, grep, awk, sort, comm, diff. If yq is
# available, the source-lock services pin is read with the canonical
# select(.id == "services") query; otherwise the script falls back to awk.
#
# Exit code contract:
#   0  completed successfully, with or without detected drift
#   2  tool failure, missing file, or parse failure
#   64 bad CLI arguments

set -uo pipefail

usage() {
  cat <<EOF
Usage: scripts/check-services-drift.sh [options]

Options:
  --upstream PATH         Path to a checkout of the upstream services
                          repository (default: /tmp/services).
  --cow-rs-root PATH      Path to the cow-rs repo root (default: cwd).
  --summary-output PATH   Write the Markdown summary report to PATH in
                          addition to stdout.
  -h, --help              Show this message and exit.
EOF
}

bad_args() {
  echo "error: $*" >&2
  usage >&2
  exit 64
}

tool_fail() {
  echo "error: $*" >&2
  exit 2
}

count_lines() {
  awk 'NF { count++ } END { print count + 0 }'
}

services_pin() {
  if command -v yq >/dev/null 2>&1; then
    local pin
    pin="$(yq -r '.repositories[] | select(.id == "services") | .commit // ""' "$1")"
    if [ -n "$pin" ] && [ "$pin" != "null" ]; then
      printf '%s\n' "$pin"
      return
    fi
  fi

  awk '
    /^repositories:/ { in_repos = 1; next }
    in_repos && /^[A-Za-z][A-Za-z_]*:/ { in_repos = 0; in_services = 0 }
    in_repos && /^- id:/ {
      in_services = ($3 == "services")
      next
    }
    in_repos && in_services && /^  commit:/ {
      print $2
      exit
    }
  ' "$1"
}

resolve_dir() {
  local label="$1"
  local path="$2"
  [ -d "$path" ] || tool_fail "$label directory not found: $path"
  (cd "$path" && pwd -P) || tool_fail "could not resolve $label directory: $path"
}

resolve_upstream_dir() {
  local requested="$1"
  local pin="$2"
  if [ -d "$requested" ]; then
    resolve_dir "upstream services" "$requested"
    return
  fi

  local parent="$requested"
  parent="${parent%/*}"
  if [ "$parent" = "$requested" ]; then
    parent="."
  fi
  if [ -d "$parent/cowprotocol-services@$pin" ]; then
    resolve_dir "upstream services" "$parent/cowprotocol-services@$pin"
    return
  fi

  tool_fail "upstream services directory not found: $requested"
}

extract_services_errors() {
  local upstream="$1"
  local api_file="$upstream/crates/orderbook/src/api.rs"
  local api_dir="$upstream/crates/orderbook/src/api"

  [ -f "$api_file" ] || tool_fail "upstream api.rs not found: $api_file"
  [ -d "$api_dir" ] || tool_fail "upstream api directory not found: $api_dir"

  awk '
    FNR == 1 {
      skip_test_module = 0
      awaiting_error_type = 0
    }
    /^#\[cfg\(test\)\]/ {
      skip_test_module = 1
    }
    skip_test_module {
      next
    }
    function emit_from_call(line) {
      if (line !~ /(^|[^A-Za-z0-9_])((super::|crate::api::)?(rich_)?error)[[:space:]]*\([[:space:]]*"/) {
        return 0
      }
      sub(/^.*((super::|crate::api::)?(rich_)?error)[[:space:]]*\([[:space:]]*"/, "", line)
      sub(/".*$/, "", line)
      if (line ~ /^[A-Za-z][A-Za-z0-9_]*$/) print line
      return 1
    }
    emit_from_call($0) {
      awaiting_error_type = 0
      next
    }
    $0 !~ /fn[[:space:]]+(rich_)?error/ && $0 ~ /(^|[^A-Za-z0-9_])((super::|crate::api::)?(rich_)?error)[[:space:]]*\([[:space:]]*$/ {
      awaiting_error_type = 1
    }
    awaiting_error_type && match($0, /"[A-Za-z][A-Za-z0-9_]*"/) {
      line = substr($0, RSTART + 1, RLENGTH - 2)
      print line
      awaiting_error_type = 0
    }
  ' "$api_file" "$api_dir"/*.rs | sort -u
}

extract_cow_rejections() {
  local rejection_file="$1"

  [ -f "$rejection_file" ] || tool_fail "cow-rs rejection file not found: $rejection_file"

  awk '
    /^pub enum OrderbookRejection[[:space:]]*\{/ {
      capture = 1
      depth = 1
      next
    }
    capture {
      line = $0
      if (depth == 1) {
        candidate = line
        sub(/^[[:space:]]*/, "", candidate)
        if (candidate ~ /^[A-Z][A-Za-z0-9_]*[[:space:]]*(,|\{)/) {
          sub(/[[:space:]].*$/, "", candidate)
          sub(/[,{\(].*$/, "", candidate)
          if (candidate != "Unknown") print candidate
        }
      }
      opens = gsub(/\{/, "{", line)
      closes = gsub(/\}/, "}", line)
      depth += opens - closes
      if (depth == 0) exit
    }
  ' "$rejection_file" | sort -u
}

list_model_structs() {
  local model_dir="$1"
  [ -d "$model_dir" ] || tool_fail "model directory not found: $model_dir"
  grep -RhoE '^pub struct [A-Za-z][A-Za-z0-9_]*[[:space:]]*\{' "$model_dir"/*.rs \
    | awk '{ print $3 }' \
    | sort -u
}

list_cow_structs() {
  local types_file="$1"
  [ -f "$types_file" ] || tool_fail "cow-rs types file not found: $types_file"
  grep -hoE '^pub struct [A-Za-z][A-Za-z0-9_]*[[:space:]]*\{' "$types_file" \
    | awk '{ print $3 }' \
    | sort -u
}

extract_struct_fields_from_files() {
  local struct_name="$1"
  shift

  awk -v target="$struct_name" '
    $0 ~ "^pub struct " target "[[:space:]]*\\{" {
      capture = 1
      depth = 1
      next
    }
    capture {
      line = $0
      if (depth == 1) {
        candidate = line
        sub(/^[[:space:]]*/, "", candidate)
        if (candidate ~ /^(pub[[:space:]]+)?[A-Za-z_][A-Za-z0-9_]*[[:space:]]*:/) {
          sub(/^pub[[:space:]]+/, "", candidate)
          name = candidate
          sub(/[[:space:]]*:.*/, "", name)
          type = candidate
          sub(/^[^:]+:[[:space:]]*/, "", type)
          sub(/[[:space:]]*,[[:space:]]*$/, "", type)
          gsub(/[[:space:]]+/, " ", type)
          print name ":" type
        }
      }
      opens = gsub(/\{/, "{", line)
      closes = gsub(/\}/, "}", line)
      depth += opens - closes
      if (depth == 0) capture = 0
    }
  ' "$@" | sort -u
}

extract_model_fields() {
  local upstream="$1"
  local struct_name="$2"
  extract_struct_fields_from_files "$struct_name" "$upstream"/crates/model/src/*.rs
}

extract_cow_fields() {
  local types_file="$1"
  local struct_name="$2"
  extract_struct_fields_from_files "$struct_name" "$types_file"
}

dto_pairs() {
  local upstream="$1"
  local cow_types="$2"
  {
    comm -12 \
      <(list_model_structs "$upstream/crates/model/src") \
      <(list_cow_structs "$cow_types")
    printf '%s\n' \
      'NativeTokenPrice|NativePriceResponse' \
      'OrderQuote|QuoteData' \
      'SolverCompetitionAPI|SolverCompetitionResponse'
  } | awk '
    /\|/ { print; next }
    NF { print $1 "|" $1 }
  ' | sort -u
}

semantic_surfaces() {
  printf '%s\n' \
    "crates/shared/src/order_validation.rs"
}

append_report() {
  report="${report}$*"$'\n'
}

upstream_arg="/tmp/services"
cow_rs_root_arg="$(pwd)"
summary_output=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --upstream)
      [ "$#" -ge 2 ] || bad_args "--upstream requires a path"
      upstream_arg="$2"
      shift 2
      ;;
    --cow-rs-root)
      [ "$#" -ge 2 ] || bad_args "--cow-rs-root requires a path"
      cow_rs_root_arg="$2"
      shift 2
      ;;
    --summary-output)
      [ "$#" -ge 2 ] || bad_args "--summary-output requires a path"
      summary_output="$2"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      bad_args "unknown argument: $1"
      ;;
  esac
done

cow_rs_root="$(resolve_dir "cow-rs root" "$cow_rs_root_arg")"
source_lock="$cow_rs_root/parity/source-lock.yaml"
[ -f "$source_lock" ] || tool_fail "source lock not found: $source_lock"

pin="$(services_pin "$source_lock")"
[ -n "$pin" ] || tool_fail "could not parse services commit from $source_lock"

upstream="$(resolve_upstream_dir "$upstream_arg" "$pin")"
rejection_file="$cow_rs_root/crates/orderbook/src/rejection.rs"
types_file="$cow_rs_root/crates/orderbook/src/types.rs"

upstream_head="unknown"
pin_status="warning: upstream path is not a git checkout"
if upstream_head="$(git -C "$upstream" rev-parse HEAD 2>/dev/null)"; then
  if [ "$upstream_head" = "$pin" ]; then
    pin_status="match"
  else
    pin_status="warning: upstream checkout is $upstream_head, expected $pin"
  fi
else
  upstream_head="unknown"
fi

services_errors="$(extract_services_errors "$upstream")"
cow_rejections="$(extract_cow_rejections "$rejection_file")"

[ -n "$services_errors" ] || tool_fail "no upstream services errorType tags parsed"
[ -n "$cow_rejections" ] || tool_fail "no cow-rs OrderbookRejection variants parsed"

err_services_only="$(comm -23 <(printf '%s\n' "$services_errors") <(printf '%s\n' "$cow_rejections"))"
err_cow_only="$(comm -13 <(printf '%s\n' "$services_errors") <(printf '%s\n' "$cow_rejections"))"
err_services_count="$(printf '%s\n' "$err_services_only" | count_lines)"
err_cow_count="$(printf '%s\n' "$err_cow_only" | count_lines)"

dto_services_count=0
dto_cow_count=0
dto_pair_count=0
dto_rows=""
semantic_drift_count=0
semantic_rows=""

while IFS='|' read -r upstream_struct cow_struct; do
  [ -n "$upstream_struct" ] || continue
  dto_pair_count=$((dto_pair_count + 1))

  upstream_fields="$(extract_model_fields "$upstream" "$upstream_struct")"
  cow_fields="$(extract_cow_fields "$types_file" "$cow_struct")"

  if [ -z "$upstream_fields" ] || [ -z "$cow_fields" ]; then
    continue
  fi

  services_only="$(comm -23 <(printf '%s\n' "$upstream_fields") <(printf '%s\n' "$cow_fields"))"
  cow_only="$(comm -13 <(printf '%s\n' "$upstream_fields") <(printf '%s\n' "$cow_fields"))"

  dto_services_count=$((dto_services_count + $(printf '%s\n' "$services_only" | count_lines)))
  dto_cow_count=$((dto_cow_count + $(printf '%s\n' "$cow_only" | count_lines)))

  while IFS= read -r field; do
    [ -n "$field" ] || continue
    field_name="${field%%:*}"
    field_type="${field#*:}"
    dto_rows="${dto_rows}| ${upstream_struct} -> ${cow_struct} | field-only-in-services | \`${field_name}\` | \`${field_type}\` |"$'\n'
  done <<< "$services_only"

  while IFS= read -r field; do
    [ -n "$field" ] || continue
    field_name="${field%%:*}"
    field_type="${field#*:}"
    dto_rows="${dto_rows}| ${upstream_struct} -> ${cow_struct} | field-only-in-cow-rs | \`${field_name}\` | \`${field_type}\` |"$'\n'
  done <<< "$cow_only"
done < <(dto_pairs "$upstream" "$types_file")

[ "$dto_pair_count" -gt 0 ] || tool_fail "no comparable DTO struct pairs found"

if [ "$upstream_head" = "unknown" ]; then
  semantic_rows="${semantic_rows}| not-checked | services checkout | upstream path is not a git checkout |"$'\n'
elif ! git -C "$upstream" cat-file -e "$pin^{commit}" 2>/dev/null; then
  semantic_drift_count=$((semantic_drift_count + 1))
  semantic_rows="${semantic_rows}| pin-missing | \`$pin\` | pinned services commit is not present in the upstream checkout; manual review required |"$'\n'
elif ! git -C "$upstream" merge-base --is-ancestor "$pin" "$upstream_head"; then
  semantic_drift_count=$((semantic_drift_count + 1))
  semantic_rows="${semantic_rows}| pin-not-ancestor | \`$pin\` | pinned services commit is not an ancestor of upstream HEAD \`$upstream_head\`; upstream history may have been rebased |"$'\n'
else
  while IFS= read -r surface; do
    [ -n "$surface" ] || continue
    diff_summary="$(git -C "$upstream" diff --stat "$pin" "$upstream_head" -- "$surface")"
    if [ -n "$diff_summary" ]; then
      semantic_drift_count=$((semantic_drift_count + 1))
      semantic_rows="${semantic_rows}| semantic-surface-changed | \`$surface\` | between pinned \`${pin:0:8}\` and upstream HEAD \`${upstream_head:0:8}\`: $diff_summary |"$'\n'
      semantic_rows="${semantic_rows}| review-target | \`$surface\` | review \`crates/trading/tests/validation_contract.rs\` and \`crates/trading/tests/parameters_contract.rs\` for parity |"$'\n'
    fi
  done < <(semantic_surfaces)
fi

total_drift=$((err_services_count + err_cow_count + dto_services_count + dto_cow_count + semantic_drift_count))
drift_detected=false
if [ "$total_drift" -gt 0 ]; then
  drift_detected=true
fi

report=""
append_report "# Upstream Services Drift Report"
append_report
append_report "| Input | Value |"
append_report "| --- | --- |"
append_report "| cow-rs root | \`$cow_rs_root\` |"
append_report "| upstream services root | \`$upstream\` |"
append_report "| pinned services commit | \`$pin\` |"
append_report "| upstream checkout commit | \`$upstream_head\` |"
append_report "| pin status | $pin_status |"
append_report "| drift detected | \`$drift_detected\` |"
append_report
append_report "## errorType Drift"
append_report
append_report "| Classification | Value | Detail |"
append_report "| --- | --- | --- |"

if [ "$err_services_count" -eq 0 ] && [ "$err_cow_count" -eq 0 ]; then
  append_report "| match | all compared errorType tags | both sides agree |"
else
  while IFS= read -r tag; do
    [ -n "$tag" ] || continue
    append_report "| errortype-only-in-services | \`$tag\` | services has it, cow-rs does not. Promotion-to-typed-variant candidate. |"
  done <<< "$err_services_only"

  while IFS= read -r tag; do
    [ -n "$tag" ] || continue
    append_report "| variant-only-in-cow-rs | \`$tag\` | cow-rs has it, services does not. Promotion-to-deletion candidate, or services dropped the tag. |"
  done <<< "$err_cow_only"
fi

append_report
append_report "## DTO Field Drift"
append_report
append_report "| DTO | Classification | Field | Type |"
append_report "| --- | --- | --- | --- |"
if [ "$dto_services_count" -eq 0 ] && [ "$dto_cow_count" -eq 0 ]; then
  append_report "| all compared DTOs | match | all compared fields | both sides agree |"
else
  report="${report}${dto_rows}"
fi

append_report
append_report "## Semantic Surfaces"
append_report
append_report "| Classification | Surface | Detail |"
append_report "| --- | --- | --- |"
if [ -z "$semantic_rows" ]; then
  append_report "| match | all compared semantic surfaces | pinned services commit and upstream HEAD agree |"
else
  report="${report}${semantic_rows}"
fi

append_report
append_report "## Summary Count"
append_report
append_report "| Metric | Count |"
append_report "| --- | ---: |"
append_report "| compared errorType tags in services | $(printf '%s\n' "$services_errors" | count_lines) |"
append_report "| compared OrderbookRejection variants in cow-rs | $(printf '%s\n' "$cow_rejections" | count_lines) |"
append_report "| errorType tags only in services | $err_services_count |"
append_report "| variants only in cow-rs | $err_cow_count |"
append_report "| compared DTO pairs | $dto_pair_count |"
append_report "| DTO fields only in services | $dto_services_count |"
append_report "| DTO fields only in cow-rs | $dto_cow_count |"
append_report "| semantic surface drift rows | $semantic_drift_count |"
append_report "| total drift rows | $total_drift |"

printf '%s' "$report"

if [ -n "$summary_output" ]; then
  printf '%s' "$report" > "$summary_output" || tool_fail "could not write summary output: $summary_output"
fi

if [ -n "${GITHUB_OUTPUT:-}" ]; then
  {
    echo "drift_detected=$drift_detected"
  } >> "$GITHUB_OUTPUT" || tool_fail "could not write GitHub output: $GITHUB_OUTPUT"
fi

exit 0
