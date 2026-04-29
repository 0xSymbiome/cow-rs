# ADR 0032: Deployment Authority Uses Machine-Readable Provenance

- Status: Accepted
- Date: 2026-04-29
- Last reviewed: 2026-04-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: deployments, provenance, contracts, release
- Anchors: Principle 1 (supporting); Principle 10 (supporting)
- Related: [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md)

## Decision

`crates/contracts/registry.toml` keeps runtime address data.
`crates/contracts/deployment-provenance.yaml` keeps structured source
provenance keyed by `(contract_id, chain_id, env)`.

Each provenance entry records the address, authority class, source
repository, source commit, source path and symbol, and a structured
`live_confirmation` object. Release-facing live confirmation uses
`kind: code_hash`, records `code_hash = keccak256(eth_getCode)`, stores
the RPC chain id, and may include selector probes where the ABI permits.

`validation-smoke registry-confirm` has two independent axes:
`--mode local|release` and `--write|--check`. `--write` is the
maintainer refresh path. `--check` is the read-only CI path. Release mode
fails when a supported production chain lacks its required RPC endpoint.

## Why

A wrong settlement address is a wallet-draining bug. TOML comments and
free-form release notes are not enough evidence for a deployment registry
that callers trust. Structured provenance makes every row traceable to a
source repository and commit, while code-hash confirmation proves the
committed address resolves to the reviewed bytecode on the expected
chain.

The write/check split keeps evidence authored by maintainers and verified
by CI. A CI job must never silently rewrite the evidence it is supposed
to verify.

## Must Remain True

- Every registry row has one matching provenance entry.
- Every provenance entry is keyed by `(contract_id, chain_id, env)` with
  no duplicates.
- `--mode release --check` fails on missing production-chain RPC
  configuration.
- `--mode release --check` fails if live recomputation diverges from the
  committed `code_hash` evidence.
- `--write` is a maintainer action; release-readiness CI uses `--check`.
- Live RPC confirms bytecode identity; it never becomes the source of
  truth for which address should be used.

## Alternatives Rejected

- Keep source provenance in comments beside TOML rows: human-readable,
  but not parseable or enforceable.
- Treat `eth_getCode != 0x` as sufficient: proves code exists, not that
  the reviewed contract is deployed there.
- Let CI update provenance in release mode: convenient, but erases the
  review boundary around release evidence.

## Anchors

This ADR supports Principle 1, Deterministic Protocol Transforms, and
Principle 10, Evidence-Backed Public Claims.

## Links

- [Principles](../principles.md)
- [Deployments](../deployments.md)
- [Deployment Registry Audit](../audit/deployment-registry-audit.md)
- [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0026](0026-alloy-major-release-absorption-plan.md)

**Proven by:**

- [Deployment Registry Audit](../audit/deployment-registry-audit.md)
- `crates/contracts/tests/deployment_provenance_contract.rs`
- `scripts/validation-smoke/tests/registry_confirm.rs`
