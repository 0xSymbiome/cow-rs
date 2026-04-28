$ErrorActionPreference = 'Stop'

$repo_root = (Resolve-Path "$PSScriptRoot/..").Path

& cargo run --quiet --manifest-path "$repo_root/scripts/policy-maintainer/Cargo.toml" -- check-stub @args
exit $LASTEXITCODE
