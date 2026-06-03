# Validation Smoke

This directory contains the optional `registry-confirm` smoke runner: a
read-only confirmation that the committed CoW Protocol deployment registry
matches live on-chain bytecode.

This kit is intentionally separate from routine deterministic validation. It
does not belong in branch protection and it does not replace the maintained
deterministic proof surfaces documented in the repository root and validation
docs.

## Command

```text
cargo registry-confirm --mode local --chain-ids 1,100
cargo registry-confirm --mode release --release-version <version>
```

(Equivalently: `cargo run --manifest-path scripts/validation-smoke/Cargo.toml -- registry-confirm ...`.)

## Exit Codes

- `0`: every selected deployment was confirmed on-chain
- `1`: at least one selected deployment failed confirmation (bytecode missing or chain mismatch)
- `2`: at least one selected deployment was skipped because its RPC endpoint was not reachable

## Environment

- `RPC_<chain_id>`
  - required for each selected production chain in `registry-confirm --mode release`
  - optional in `registry-confirm --mode local`; missing RPC endpoints are reported as skipped
- `RPC_MAINNET`, `RPC_GNOSIS`, `RPC_ARBITRUM`, `RPC_BASE`, `RPC_POLYGON`, `RPC_AVALANCHE`, `RPC_BNB`, `RPC_SEPOLIA`, `RPC_PLASMA`, `RPC_LINEA`, `RPC_INK`, `RPC_LENS`
  - accepted aliases for the corresponding deployment chain ids

`registry-confirm` is read-only: it confirms on-chain presence and never mutates
committed files. Trust rests on the pinned `source_commit` plus the
deterministic CREATE2 address; the probe adds the live check that the deployment
exists.
