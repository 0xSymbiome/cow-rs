---
type: Decision Record
id: ADR-0032
title: "ADR 0032: Deployment Authority Uses Machine-Readable Provenance"
description: "Every deployed-contract address the SDK resolves at runtime comes from the typed Registry of committed CREATE2-singleton constants in crates/contracts/src/deployments.rs, keyed by (ContractId, chain, env) through Registry::address (per [ADR..."
status: Accepted
date: 2026-04-29
last_reviewed: 2026-06-15
authors: ["0xSymbiotic"]
tags: [deployments, provenance, contracts, release]
related: [ADR-0012, ADR-0026, ADR-0052]
timestamp: 2026-06-15T00:00:00Z
---

# ADR 0032: Deployment Authority Uses Machine-Readable Provenance

## Decision

Every deployed-contract address the SDK resolves at runtime comes from the typed
`Registry` of committed CREATE2-singleton constants in
`crates/contracts/src/deployments.rs`, keyed by `(ContractId, chain, env)`
through `Registry::address` (per [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md)).
`ContractId` is `Settlement` / `VaultRelayer` / `EthFlow`, and `DeploymentEnv` is
`Prod` / `Staging`; within an environment each address is a chain-invariant
CREATE2 singleton. The upstream commit each address derives from is pinned once
per source repository in `parity/source-lock.yaml`, not duplicated per row.

Deployment trust rests on three layers: (1) the pinned upstream `source_commit`
in `parity/source-lock.yaml` (where the deployment is explorer/Sourcify-verified),
(2) the deterministic CREATE2 address, and (3) a read-only live presence probe.
`cargo registry-confirm --mode {local|release}` iterates every
`(ContractId, chain, env)` constant, guards each RPC with `eth_chainId`, and
asserts `eth_getCode` returns non-empty bytecode at the resolved address; release
mode fails closed on a missing production-chain RPC or an absent deployment. The
probe never mutates a file and never becomes the source of truth for which
address to use.

Committed per-row code-hash confirmation is **not** used: every resolved contract
is a non-upgradeable CREATE2 singleton whose bytecode at a fixed address cannot
change, so a presence probe is the appropriate live check; committed code-hash
confirmation is reserved for any future upgradeable deployment. A dedicated
per-row provenance file is likewise declined — its only non-redundant payload,
the upstream commit, already has one authoritative home in
`parity/source-lock.yaml`.

## Why

A wrong settlement address is a wallet-draining bug. The committed constant plus
the per-repository commit pin make every address traceable to an upstream
repository and pinned commit; the deterministic CREATE2 address plus one-time
upstream explorer verification establish initial correctness; and the live
presence probe proves the claimed deployment exists on-chain. This matches what
every upstream CoW repository relies on (address + source, no committed code
hashes) while adding a matrix-driven live check the upstreams lack. Storing the
addresses as committed constants rather than a parsed manifest removes a
validator over data that does not vary — each runtime-resolved contract is a
single CREATE2 address repeated across chains — and keeps the provenance evidence
repository-visible.

## Must Remain True

- Every resolved address comes from the const `Registry` keyed by
  `(ContractId, chain, env)`, is non-zero, and resolves to a chain-invariant
  CREATE2 singleton within its environment.
- The upstream commit each address derives from is pinned in
  `parity/source-lock.yaml`.
- `cargo registry-confirm --mode release` fails on a missing production-chain RPC
  and on any resolved address whose `eth_getCode` is empty on the expected chain.
- The probe is read-only; it never mutates committed evidence and never overrides
  the resolved address.
- A distinct production and staging deployment is resolved for each family
  (`GPv2Settlement`, `GPv2VaultRelayer`, `CoWSwapEthFlow`): the staging settlement
  is the typed-data `verifyingContract` for staging orders and the staging vault
  relayer is the staging allowance spender.

## Alternatives Rejected

- Keep a parsed `registry.toml` manifest plus a `build.rs` schema validator and a
  runtime TOML parser: validates data that does not vary (one CREATE2 address per
  contract repeated across chains) and carries compile-time and runtime
  validators the const table makes unnecessary.
- Keep a dedicated per-row provenance file mirroring every registry row: its
  address and verification fields duplicate the registry, and its only unique
  payload (the upstream commit) already lives in `parity/source-lock.yaml`.
- Commit a per-row `keccak256(eth_getCode)` digest and fail-closed-compare it:
  guards only bytecode substitution (impossible for this non-upgradeable set) at
  far higher review cost than any upstream carries. Reserved for a future
  upgradeable deployment.
- Let CI mutate the registry in release mode: erases the review boundary around
  release evidence — the probe is read-only.

## Anchors

This ADR supports the Deterministic Protocol Transforms and Evidence-Backed
Public Claims principles.

## Links

- [Principles](../principles/index.md)
- [Deployments](../guides/deployments.md)
- [Deployment Registry Audit](../audit/deployment-registry-audit.md)
- [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0026](0026-alloy-major-release-absorption-plan.md)

**Proven by:**

- [Deployment Registry Audit](../audit/deployment-registry-audit.md)
- `crates/contracts/src/deployments.rs` (tests)
- `xtask/tests/registry_confirm.rs`
