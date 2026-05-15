# Composable Contract Bindings Audit

Status: Current
Last reviewed: 2026-05-15
Owning surface: composable Solidity excerpts, deployment registry rows, and Layer A parity fixtures
Refresh trigger: Refresh when composable-cow deployments, contract ABIs, conditional-order type strings, or selector vectors change upstream.
Related docs:
- [ADR 0048](../adr/0048-composable-conditional-order-framework.md)
- [ADR 0050](../adr/0050-eip1271-signature-blob-encoding.md)
- [Composable Watch-Tower Boundary Audit](composable-watch-tower-boundary-audit.md)

## Scope

This audit covers:

- the vendored composable-cow Solidity excerpts that anchor the SDK's typed
  bindings;
- the schema v2 deployment registry rows that pin composable contract
  addresses per chain id;
- the Layer A parity fixtures (`selectors.json`, `params_hash.json`,
  `multiplexer_leaf.json`) that prove byte-identity against the upstream
  source;
- the Ink Reality Check that classifies Ink composable rows as coverage-only;
- the parity fixture catalog that lists every shipped composable fixture
  with its upstream provenance row.

It does not cover the composable helper crate's runtime API surface beyond
selectors and decoders; the watch-tower boundary is governed by the
[Composable Watch-Tower Boundary Audit](composable-watch-tower-boundary-audit.md).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Solidity excerpts | Vendored excerpts compile under `alloy::sol!` and produce selectors byte-identical to `forge methodIdentifiers` output | Conforms |
| Deployment registry | Schema v2 row count for composable contracts matches the pinned deployment set; no Ink composable rows are present in `registry.toml` | Conforms |
| Layer A fixtures | `selectors.json`, `params_hash.json`, and `multiplexer_leaf.json` carry real byte-identity values against the pinned upstream test vectors | Conforms |
| Ink Reality Check | Every Ink composable tuple appears in `deployment-coverage.yaml` with `not_deployed` status and an `eth_getCode` probe captured in `parity/ink-probe-results.json` | Conforms |
| Fixture catalog | Every catalogued composable parity fixture file exists on disk and ties to an upstream provenance row | Conforms |

## Current Contract

### Solidity excerpts

The vendored composable-cow Solidity excerpts live under
`crates/contracts/abi/composable-cow/`. The set covers
`ComposableCoW.sol`, `BaseConditionalOrder.sol`,
`ERC1271Forwarder.sol`, the four non-TWAP handler contracts
(`GoodAfterTime.sol`, `StopLoss.sol`, `TradeAboveThreshold.sol`,
`PerpetualStableSwap.sol`), the TWAP handler at
`types/twap/TWAP.sol`, the three interface modules under
`interfaces/`, the `CurrentBlockTimestampFactory.sol` value factory, and the
local CoW settlement excerpt. Selectors generated from these excerpts via
`alloy::sol!` are byte-identical to `forge methodIdentifiers` output for the
same contract set.

### Deployment registry

Composable contracts ship as capability rows under
`DeploymentEnv::EnvironmentAgnostic` in `crates/contracts/registry.toml`.
The capability set covers `ComposableCoW`, the four non-TWAP handlers, the
TWAP handler, the value factory, and the ERC-1271 forwarder. No Ink
composable row appears in `registry.toml`; every Ink composable tuple is a
coverage-only record under `crates/contracts/deployment-coverage.yaml` with
`coverage_status: not_deployed`.

### Layer A fixtures

The Layer A parity fixtures at
`parity/fixtures/composable/selectors.json`,
`parity/fixtures/composable/params_hash.json`, and
`parity/fixtures/composable/multiplexer_leaf.json` carry byte-identity
values against pinned upstream test vectors. Every row records the
contract, the function or struct signature, the canonical selector or
hash, and the upstream provenance source path.

### Ink Reality Check

The Ink Reality Check is the live-probe contract that prevents Ink
composable evidence from being promoted into `registry.toml` while the
deployed bytecode at Ink composable addresses returns `0x`. The contract
is mechanically enforceable: a probe artifact at
`parity/ink-probe-results.json` records every `eth_getCode` call against
the candidate addresses on chain id 57073; matching rows in
`parity/ink-composable-rows.json` enumerate the per-contract evidence
status; a build-time invariant in `crates/contracts/build.rs` rejects any
registry row whose chain id resolves to an Ink coverage record marked
`not_deployed`. Promoting an Ink composable row into `registry.toml`
without first updating the probe artifact to record non-empty code and
adding a matching `deployment-provenance.yaml` row is a regression of
this contract.

### Fixture catalog

The parity-maintainer binary owns a catalog of every shipped composable
parity fixture file under
`scripts/parity-maintainer/src/composable_fixtures.rs`. Each catalog row
pairs a shipped fixture path with one or more upstream provenance entries.
The `validate-fixture-catalog` subcommand of the parity-maintainer binary
walks the catalog and fails if any shipped fixture file is missing on disk.

## Evidence

Primary implementation points:

- `crates/contracts/abi/composable-cow/`
- `crates/contracts/registry.toml`
- `crates/contracts/deployment-coverage.yaml`
- `crates/contracts/build.rs`
- `parity/fixtures/composable/selectors.json`
- `parity/fixtures/composable/params_hash.json`
- `parity/fixtures/composable/multiplexer_leaf.json`
- `parity/ink-composable-rows.json`
- `parity/ink-probe-results.json`
- `scripts/parity-maintainer/src/composable_fixtures.rs`

Primary regression coverage:

- `crates/contracts/tests/schema_v2_success.rs`
- `crates/contracts/tests/schema_v2_rejection.rs`
- `crates/contracts/tests/trybuild_schema_v2.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --all-features
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate-fixture-catalog --root .
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml
```
