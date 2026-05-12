#!/usr/bin/env bash
# scripts/check-services-drift.sh
#
# Report-only detector for upstream parity drift. It compares the services
# OpenAPI, services errorType set, selected model DTO fields, generated
# settlement deployment chain table, and cow-sdk supported-chain README list
# against the typed cow-rs surface in this checkout, then emits a Markdown
# summary for CI.
#
# Dependencies: bash, git, grep, awk, sort, comm, diff. If yq is available,
# source-lock pins are read with the canonical select(.id == "...") query;
# otherwise the script falls back to awk.
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
  --services-upstream PATH
                          Explicit path to the upstream services repository.
  --contracts-upstream PATH
                          Explicit path to the upstream contracts repository.
                          Defaults to a sibling of the services checkout.
  --cow-sdk-upstream PATH Explicit path to the upstream cow-sdk repository.
                          Defaults to a sibling of the services checkout.
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

source_lock_commit() {
  local source_lock_path="$1"
  local repo_id="$2"

  if command -v yq >/dev/null 2>&1; then
    local pin
    pin="$(yq -r ".repositories[] | select(.id == \"$repo_id\") | .commit // \"\"" "$source_lock_path")"
    if [ -n "$pin" ] && [ "$pin" != "null" ]; then
      printf '%s\n' "$pin"
      return
    fi
  fi

  awk '
    /^repositories:/ { in_repos = 1; next }
    in_repos && /^[A-Za-z][A-Za-z_]*:/ { in_repos = 0; in_target = 0 }
    in_repos && /^- id:/ {
      in_target = ($3 == repo_id)
      next
    }
    in_repos && in_target && /^  commit:/ {
      print $2
      exit
    }
  ' repo_id="$repo_id" "$source_lock_path"
}

resolve_dir() {
  local label="$1"
  local path="$2"
  [ -d "$path" ] || tool_fail "$label directory not found: $path"
  (cd "$path" && pwd -P) || tool_fail "could not resolve $label directory: $path"
}

resolve_upstream_dir() {
  local repo_id="$1"
  local label="$2"
  local requested="$3"
  local pin="$4"
  if [ -d "$requested" ]; then
    resolve_dir "$label" "$requested"
    return
  fi

  local parent="$requested"
  parent="${parent%/*}"
  if [ "$parent" = "$requested" ]; then
    parent="."
  fi
  if [ -d "$parent/cowprotocol-$repo_id@$pin" ]; then
    resolve_dir "$label" "$parent/cowprotocol-$repo_id@$pin"
    return
  fi

  tool_fail "$label directory not found: $requested"
}

infer_sibling_upstream_dir() {
  local repo_id="$1"
  local services_dir="$2"
  local parent
  parent="$(dirname "$services_dir")"
  printf '%s\n' "$parent/$repo_id"
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

extract_cow_supported_chains() {
  local config_file="$1"
  [ -f "$config_file" ] || tool_fail "cow-rs config file not found: $config_file"

  awk '
    /^pub enum SupportedChainId[[:space:]]*\{/ {
      capture = 1
      next
    }
    capture && /^}/ {
      exit
    }
    capture {
      candidate = $0
      sub(/\/\/.*$/, "", candidate)
      if (candidate ~ /^[[:space:]]*[A-Za-z][A-Za-z0-9_]*[[:space:]]*=[[:space:]]*[0-9_]+[[:space:]]*,/) {
        name = candidate
        sub(/^[[:space:]]*/, "", name)
        sub(/[[:space:]]*=.*$/, "", name)

        chain_id = candidate
        sub(/^.*=[[:space:]]*/, "", chain_id)
        sub(/[[:space:]]*,.*$/, "", chain_id)
        gsub(/_/, "", chain_id)

        print chain_id "|" name
      }
    }
  ' "$config_file" | sort -t '|' -k1,1
}

extract_services_settlement_chains() {
  local services_root="$1"
  local generated="$services_root/contracts/generated/contracts-generated/gpv2settlement/src/lib.rs"
  [ -f "$generated" ] || tool_fail "services generated gpv2settlement metadata not found: $generated"

  awk '
    /^pub const fn deployment_info\(chain_id: u64\)/ {
      capture = 1
      next
    }
    capture && /^}/ {
      exit
    }
    capture {
      candidate = $0
      sub(/u64.*$/, "", candidate)
      sub(/^[[:space:]]*/, "", candidate)
      if (candidate ~ /^[0-9]+$/) {
        print candidate
      }
    }
  ' "$generated" | sort -u
}

extract_cow_sdk_readme_chains() {
  local cow_sdk_root="$1"
  local readme="$cow_sdk_root/README.md"
  [ -f "$readme" ] || tool_fail "cow-sdk README not found: $readme"

  awk '
    /^### Supported chains[[:space:]]*$/ {
      capture = 1
      next
    }
    capture && /^###[[:space:]]+/ {
      exit
    }
    capture && /^##[[:space:]]+/ {
      exit
    }
    capture && /^\- / && match($0, /\([0-9]+\)/) {
      value = substr($0, RSTART + 1, RLENGTH - 2)
      print value
    }
  ' "$readme" | sort -u
}

chain_id_set() {
  cut -d '|' -f 1 | sort -u
}

chain_variant_for_id() {
  local chains="$1"
  local chain_id="$2"
  printf '%s\n' "$chains" | awk -F '|' -v target="$chain_id" '$1 == target { print $2; exit }'
}

append_report() {
  report="${report}$*"$'\n'
}

upstream_arg="/tmp/services"
services_upstream_arg=""
contracts_upstream_arg=""
cow_sdk_upstream_arg=""
cow_rs_root_arg="$(pwd)"
summary_output=""

while [ "$#" -gt 0 ]; do
  case "$1" in
    --upstream)
      [ "$#" -ge 2 ] || bad_args "--upstream requires a path"
      upstream_arg="$2"
      shift 2
      ;;
    --services-upstream)
      [ "$#" -ge 2 ] || bad_args "--services-upstream requires a path"
      services_upstream_arg="$2"
      shift 2
      ;;
    --contracts-upstream)
      [ "$#" -ge 2 ] || bad_args "--contracts-upstream requires a path"
      contracts_upstream_arg="$2"
      shift 2
      ;;
    --cow-sdk-upstream)
      [ "$#" -ge 2 ] || bad_args "--cow-sdk-upstream requires a path"
      cow_sdk_upstream_arg="$2"
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

services_pin="$(source_lock_commit "$source_lock" "services")"
contracts_pin="$(source_lock_commit "$source_lock" "contracts")"
cow_sdk_pin="$(source_lock_commit "$source_lock" "cow-sdk")"
[ -n "$services_pin" ] || tool_fail "could not parse services commit from $source_lock"
[ -n "$contracts_pin" ] || tool_fail "could not parse contracts commit from $source_lock"
[ -n "$cow_sdk_pin" ] || tool_fail "could not parse cow-sdk commit from $source_lock"

if [ -z "$services_upstream_arg" ]; then
  services_upstream_arg="$upstream_arg"
fi

services_upstream="$(resolve_upstream_dir "services" "upstream services" "$services_upstream_arg" "$services_pin")" || exit $?

if [ -z "$contracts_upstream_arg" ]; then
  contracts_upstream_arg="$(infer_sibling_upstream_dir "contracts" "$services_upstream")"
fi
if [ -z "$cow_sdk_upstream_arg" ]; then
  cow_sdk_upstream_arg="$(infer_sibling_upstream_dir "cow-sdk" "$services_upstream")"
fi

contracts_upstream="$(resolve_upstream_dir "contracts" "upstream contracts" "$contracts_upstream_arg" "$contracts_pin")" || exit $?
cow_sdk_upstream="$(resolve_upstream_dir "cow-sdk" "upstream cow-sdk" "$cow_sdk_upstream_arg" "$cow_sdk_pin")" || exit $?

rejection_file="$cow_rs_root/crates/orderbook/src/rejection.rs"
types_file="$cow_rs_root/crates/orderbook/src/types.rs"
config_file="$cow_rs_root/crates/core/src/config/chains.rs"
local_openapi="$cow_rs_root/parity/openapi/services-orderbook.yml"
services_openapi="$services_upstream/crates/orderbook/openapi.yml"
[ -f "$local_openapi" ] || tool_fail "cow-rs services OpenAPI snapshot not found: $local_openapi"
[ -f "$services_openapi" ] || tool_fail "upstream services OpenAPI not found: $services_openapi"

services_head="unknown"
contracts_head="unknown"
cow_sdk_head="unknown"
services_pin_status="warning: upstream path is not a git checkout"
contracts_pin_status="warning: upstream path is not a git checkout"
cow_sdk_pin_status="warning: upstream path is not a git checkout"

if services_head="$(git -C "$services_upstream" rev-parse HEAD 2>/dev/null)"; then
  if [ "$services_head" = "$services_pin" ]; then
    services_pin_status="match"
  else
    services_pin_status="warning: upstream checkout is $services_head, expected $services_pin"
  fi
else
  services_head="unknown"
fi
if contracts_head="$(git -C "$contracts_upstream" rev-parse HEAD 2>/dev/null)"; then
  if [ "$contracts_head" = "$contracts_pin" ]; then
    contracts_pin_status="match"
  else
    contracts_pin_status="warning: upstream checkout is $contracts_head, expected $contracts_pin"
  fi
else
  contracts_head="unknown"
fi
if cow_sdk_head="$(git -C "$cow_sdk_upstream" rev-parse HEAD 2>/dev/null)"; then
  if [ "$cow_sdk_head" = "$cow_sdk_pin" ]; then
    cow_sdk_pin_status="match"
  else
    cow_sdk_pin_status="warning: upstream checkout is $cow_sdk_head, expected $cow_sdk_pin"
  fi
else
  cow_sdk_head="unknown"
fi

openapi_drift_count=0
openapi_rows=""
strip_openapi_vendor_header() {
  sed \
    -e '/^# Vendored from cowprotocol\/services @ /d' \
    -e '/^# Path: crates\/orderbook\/openapi.yml$/d' \
    -e '/^# Generated: /d' \
    -e '/^# DO NOT EDIT - regenerate via `parity-maintainer vendor-openapi`\.$/d' \
    "$1"
}

if diff -q <(strip_openapi_vendor_header "$services_openapi") <(strip_openapi_vendor_header "$local_openapi") >/dev/null; then
  openapi_rows="| match | services OpenAPI snapshot | upstream \`crates/orderbook/openapi.yml\` matches \`parity/openapi/services-orderbook.yml\` |"$'\n'
else
  openapi_drift_count=1
  openapi_summary="OpenAPI documents differ after vendoring-header normalization"
  openapi_rows="| openapi-drift | services OpenAPI snapshot | $openapi_summary |"$'\n'
fi

services_errors="$(extract_services_errors "$services_upstream")"
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

  upstream_fields="$(extract_model_fields "$services_upstream" "$upstream_struct")"
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
done < <(dto_pairs "$services_upstream" "$types_file")

[ "$dto_pair_count" -gt 0 ] || tool_fail "no comparable DTO struct pairs found"

if [ "$services_head" = "unknown" ]; then
  semantic_rows="${semantic_rows}| not-checked | services checkout | upstream path is not a git checkout |"$'\n'
elif ! git -C "$services_upstream" cat-file -e "$services_pin^{commit}" 2>/dev/null; then
  semantic_drift_count=$((semantic_drift_count + 1))
  semantic_rows="${semantic_rows}| pin-missing | \`$services_pin\` | pinned services commit is not present in the upstream checkout; manual review required |"$'\n'
elif ! git -C "$services_upstream" merge-base --is-ancestor "$services_pin" "$services_head"; then
  semantic_drift_count=$((semantic_drift_count + 1))
  semantic_rows="${semantic_rows}| pin-not-ancestor | \`$services_pin\` | pinned services commit is not an ancestor of upstream HEAD \`$services_head\`; upstream history may have been rebased |"$'\n'
else
  while IFS= read -r surface; do
    [ -n "$surface" ] || continue
    diff_summary="$(git -C "$services_upstream" diff --stat "$services_pin" "$services_head" -- "$surface")"
    if [ -n "$diff_summary" ]; then
      semantic_drift_count=$((semantic_drift_count + 1))
      semantic_rows="${semantic_rows}| semantic-surface-changed | \`$surface\` | between pinned \`${services_pin:0:8}\` and upstream HEAD \`${services_head:0:8}\`: $diff_summary |"$'\n'
      semantic_rows="${semantic_rows}| review-target | \`$surface\` | review \`crates/trading/tests/validation_contract.rs\` and \`crates/trading/tests/parameters_contract.rs\` for parity |"$'\n'
    fi
  done < <(semantic_surfaces)
fi

cow_supported_chains="$(extract_cow_supported_chains "$config_file")"
cow_supported_chain_ids="$(printf '%s\n' "$cow_supported_chains" | chain_id_set)"
services_settlement_chain_ids_all="$(extract_services_settlement_chains "$services_upstream")"
cow_sdk_readme_chain_ids="$(extract_cow_sdk_readme_chains "$cow_sdk_upstream")"
services_settlement_chain_ids="$(comm -12 <(printf '%s\n' "$services_settlement_chain_ids_all") <(printf '%s\n' "$cow_sdk_readme_chain_ids"))"
services_deployment_only_chain_ids="$(comm -23 <(printf '%s\n' "$services_settlement_chain_ids_all") <(printf '%s\n' "$cow_sdk_readme_chain_ids"))"

[ -n "$cow_supported_chain_ids" ] || tool_fail "no cow-rs SupportedChainId variants parsed"
[ -n "$services_settlement_chain_ids_all" ] || tool_fail "no services gpv2settlement chain ids parsed"
[ -n "$cow_sdk_readme_chain_ids" ] || tool_fail "no cow-sdk README supported chain ids parsed"

chain_drift_count=0
chain_rows=""

services_chain_only="$(comm -23 <(printf '%s\n' "$services_settlement_chain_ids") <(printf '%s\n' "$cow_supported_chain_ids"))"
services_cow_only="$(comm -13 <(printf '%s\n' "$services_settlement_chain_ids") <(printf '%s\n' "$cow_supported_chain_ids"))"
services_chain_only_count="$(printf '%s\n' "$services_chain_only" | count_lines)"
services_cow_only_count="$(printf '%s\n' "$services_cow_only" | count_lines)"
chain_drift_count=$((chain_drift_count + services_chain_only_count + services_cow_only_count))
if [ "$services_chain_only_count" -eq 0 ] && [ "$services_cow_only_count" -eq 0 ]; then
  chain_rows="${chain_rows}| match | services gpv2settlement deployment_info | all cow-sdk-supported chain ids match \`SupportedChainId::ALL\` |"$'\n'
else
  while IFS= read -r chain_id; do
    [ -n "$chain_id" ] || continue
    chain_rows="${chain_rows}| chain-only-in-services | \`$chain_id\` | services generated metadata contains a chain id missing from \`SupportedChainId::ALL\` |"$'\n'
  done <<< "$services_chain_only"
  while IFS= read -r chain_id; do
    [ -n "$chain_id" ] || continue
    variant="$(chain_variant_for_id "$cow_supported_chains" "$chain_id")"
    chain_rows="${chain_rows}| chain-only-in-cow-rs | \`$chain_id\` | \`SupportedChainId::$variant\` missing from services generated settlement metadata |"$'\n'
  done <<< "$services_cow_only"
fi

while IFS= read -r chain_id; do
  [ -n "$chain_id" ] || continue
  chain_rows="${chain_rows}| deployment-only-in-services | \`$chain_id\` | services generated settlement metadata contains this deployed chain, but cow-sdk README does not list it as a supported CoW Swap chain |"$'\n'
done <<< "$services_deployment_only_chain_ids"

cow_sdk_readme_only="$(comm -23 <(printf '%s\n' "$cow_sdk_readme_chain_ids") <(printf '%s\n' "$cow_supported_chain_ids"))"
cow_sdk_cow_only="$(comm -13 <(printf '%s\n' "$cow_sdk_readme_chain_ids") <(printf '%s\n' "$cow_supported_chain_ids"))"
cow_sdk_readme_only_count="$(printf '%s\n' "$cow_sdk_readme_only" | count_lines)"
cow_sdk_cow_only_count="$(printf '%s\n' "$cow_sdk_cow_only" | count_lines)"
chain_drift_count=$((chain_drift_count + cow_sdk_readme_only_count + cow_sdk_cow_only_count))
if [ "$cow_sdk_readme_only_count" -eq 0 ] && [ "$cow_sdk_cow_only_count" -eq 0 ]; then
  chain_rows="${chain_rows}| match | cow-sdk README supported chains | all chain ids match \`SupportedChainId::ALL\` |"$'\n'
else
  while IFS= read -r chain_id; do
    [ -n "$chain_id" ] || continue
    chain_rows="${chain_rows}| chain-only-in-cow-sdk-readme | \`$chain_id\` | cow-sdk README lists a supported chain id missing from \`SupportedChainId::ALL\` |"$'\n'
  done <<< "$cow_sdk_readme_only"
  while IFS= read -r chain_id; do
    [ -n "$chain_id" ] || continue
    variant="$(chain_variant_for_id "$cow_supported_chains" "$chain_id")"
    chain_rows="${chain_rows}| chain-only-in-cow-rs | \`$chain_id\` | \`SupportedChainId::$variant\` missing from cow-sdk README supported chains |"$'\n'
  done <<< "$cow_sdk_cow_only"
fi

total_drift=$((openapi_drift_count + err_services_count + err_cow_count + dto_services_count + dto_cow_count + semantic_drift_count + chain_drift_count))
drift_detected=false
if [ "$total_drift" -gt 0 ]; then
  drift_detected=true
fi

report=""
append_report "# Upstream Parity Drift Report"
append_report
append_report "| Input | Value |"
append_report "| --- | --- |"
append_report "| cow-rs root | \`$cow_rs_root\` |"
append_report "| upstream services root | \`$services_upstream\` |"
append_report "| upstream contracts root | \`$contracts_upstream\` |"
append_report "| upstream cow-sdk root | \`$cow_sdk_upstream\` |"
append_report "| drift detected | \`$drift_detected\` |"
append_report
append_report "## Source-Lock Pins"
append_report
append_report "| Repository | Pinned commit | Checkout commit | Status |"
append_report "| --- | --- | --- | --- |"
append_report "| services | \`$services_pin\` | \`$services_head\` | $services_pin_status |"
append_report "| contracts | \`$contracts_pin\` | \`$contracts_head\` | $contracts_pin_status |"
append_report "| cow-sdk | \`$cow_sdk_pin\` | \`$cow_sdk_head\` | $cow_sdk_pin_status |"
append_report
append_report "## OpenAPI Drift"
append_report
append_report "| Classification | Surface | Detail |"
append_report "| --- | --- | --- |"
report="${report}${openapi_rows}"
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
append_report "## Chain Coverage Drift"
append_report
append_report "| Classification | Surface | Detail |"
append_report "| --- | --- | --- |"
report="${report}${chain_rows}"

append_report
append_report "## Summary Count"
append_report
append_report "| Metric | Count |"
append_report "| --- | ---: |"
append_report "| OpenAPI drift rows | $openapi_drift_count |"
append_report "| compared errorType tags in services | $(printf '%s\n' "$services_errors" | count_lines) |"
append_report "| compared OrderbookRejection variants in cow-rs | $(printf '%s\n' "$cow_rejections" | count_lines) |"
append_report "| errorType tags only in services | $err_services_count |"
append_report "| variants only in cow-rs | $err_cow_count |"
append_report "| compared DTO pairs | $dto_pair_count |"
append_report "| DTO fields only in services | $dto_services_count |"
append_report "| DTO fields only in cow-rs | $dto_cow_count |"
append_report "| semantic surface drift rows | $semantic_drift_count |"
append_report "| compared SupportedChainId variants | $(printf '%s\n' "$cow_supported_chain_ids" | count_lines) |"
append_report "| services deployment-only chain rows | $(printf '%s\n' "$services_deployment_only_chain_ids" | count_lines) |"
append_report "| services settlement chain drift rows | $((services_chain_only_count + services_cow_only_count)) |"
append_report "| cow-sdk README chain drift rows | $((cow_sdk_readme_only_count + cow_sdk_cow_only_count)) |"
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
