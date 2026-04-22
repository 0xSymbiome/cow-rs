#!/usr/bin/env bash
# scripts/fetch-upstream-pins.sh
#
# Materialize each upstream parity producer at its pinned commit as an
# independent worktree outside the cow-rs tree, so reviewers can pass
# the resolved paths into the upstream-root parity validator. Reads
# parity/source-lock.yaml; default destination is a sibling of the
# cow-rs checkout (override with `--into <dir>`). Existing destinations
# are skipped. Nothing inside the cow-rs tree is touched.
# Dependencies: git, awk (POSIX).

set -euo pipefail

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
source_lock="$repo_root/parity/source-lock.yaml"
into="$(cd "$repo_root/.." && pwd)"

usage() {
  cat <<EOF
Usage: $(basename "$0") [--into <directory>]

Materialize the pinned upstream parity producer repositories as
independent worktrees outside the cow-rs tree.

Options:
  --into <dir>   Provision worktrees under <dir> instead of the parent
                 directory of the cow-rs checkout.
  -h, --help     Show this message and exit.
EOF
}

while [ $# -gt 0 ]; do
  case "$1" in
    --into)
      if [ $# -lt 2 ]; then
        echo "error: --into requires a directory argument" >&2
        exit 1
      fi
      into="$2"
      shift 2
      ;;
    -h|--help) usage; exit 0 ;;
    *)
      echo "error: unrecognised argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [ ! -f "$source_lock" ]; then
  echo "error: source-lock not found at $source_lock" >&2
  exit 1
fi

emit_repos() {
  # Stream `id<TAB>remote<TAB>commit` rows for each repositories[] entry,
  # closing the stream when a sibling top-level key starts.
  awk '
    /^repositories:/ { state = "repos"; next }
    state == "repos" && /^[A-Za-z][A-Za-z_]*:/ {
      if (id != "") print id "\t" remote "\t" commit
      id = ""; remote = ""; commit = ""
      state = ""
      next
    }
    state == "repos" && /^- id:/ {
      if (id != "") print id "\t" remote "\t" commit
      id = $3; remote = ""; commit = ""
      next
    }
    state == "repos" && /^  remote:/ { remote = $2; next }
    state == "repos" && /^  commit:/ { commit = $2; next }
    END { if (id != "") print id "\t" remote "\t" commit }
  ' "$source_lock"
}

mkdir -p "$into"
resolved=""

while IFS=$'\t' read -r id remote commit; do
  [ -n "$id" ] || continue
  dest="$into/cowprotocol-${id}@${commit}"
  if [ -d "$dest" ]; then
    echo "skip: $dest already exists"
  else
    echo "clone: $remote -> $dest"
    git clone --filter=blob:none "$remote" "$dest"
    git -C "$dest" fetch origin "$commit"
    git -C "$dest" checkout --detach "$commit"
  fi
  echo "ready: $dest ($commit)"
  resolved="${resolved}  ${id}: ${dest} (${commit})\n"
done < <(emit_repos)

echo
printf 'Pinned upstream worktrees ready:\n'
printf '%b' "$resolved"
echo "Pass the resolved paths into the upstream-root parity validator."
