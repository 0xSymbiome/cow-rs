#!/usr/bin/env bash
set -euo pipefail

script_dir="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
repo_root="$(cd "$script_dir/.." && pwd)"

exec cargo run --quiet --manifest-path "$repo_root/scripts/policy-maintainer/Cargo.toml" -- check-stub "$@"
