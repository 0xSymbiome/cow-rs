# ADR 0032: Deployment Authority Uses Machine-Readable Provenance

- Status: Accepted (amended)
- Date: 2026-04-29
- Last reviewed: 2026-06-01
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: deployments, provenance, contracts, release
- Anchors: Deterministic Protocol Transforms (supporting); Evidence-Backed Public Claims (supporting)
- Related: [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

`crates/contracts/registry.toml` keeps runtime address data.
`crates/contracts/deployment-provenance.yaml` keeps structured source
provenance keyed by `(contract_id, chain_id, env)`.

Each provenance entry records the address, the `verification` status and
source, and the source repository, commit, path, and symbol the address was
taken from. Deployment trust rests on three layers: (1) the pinned
`source_commit` to an upstream machine-readable manifest (where deployments are
explorer/Sourcify-verified), (2) the deterministic CREATE2 address, and (3) a
read-only live presence probe.

`validation-smoke registry-confirm --mode {local|release}` guards each RPC with
`eth_chainId` and asserts `eth_getCode` returns non-empty bytecode at every
recorded address. Release mode fails closed on a missing production-chain RPC or
an absent deployment. The probe never mutates a file.

Committed per-row code-hash confirmation is **not** used for the current
contract set: every deployed contract is a non-upgradeable CREATE2 singleton
whose bytecode at a fixed address cannot change, so a presence probe is the
appropriate live check. Committed code-hash confirmation is reserved for any
future upgradeable deployment.

## Why

A wrong settlement address is a wallet-draining bug. Structured provenance makes
every row traceable to an upstream repository and pinned commit; the
deterministic CREATE2 address plus one-time upstream explorer verification
establish initial correctness; and the live presence probe proves the claimed
deployment actually exists on-chain. This matches what every upstream CoW
repository relies on (address + source, no committed code hashes) while adding a
matrix-driven live check the upstreams lack.

A committed per-row code hash would only catch bytecode *substitution* at a fixed
address — impossible for non-upgradeable CREATE2 singletons — at far higher
committed-evidence and review cost than any upstream carries, so it is declined
here.

## Must Remain True

- Every registry row has one matching provenance entry.
- Every provenance entry is keyed by `(contract_id, chain_id, env)` with
  no duplicates.
- Each row pins a 40-hex `source_commit` to the upstream manifest it came from.
- `registry-confirm --mode release` fails on a missing production-chain RPC and
  on a registry row whose `eth_getCode` is empty on the expected chain.
- The probe is read-only; it never mutates committed evidence.
- Live RPC confirms a deployment exists; it never becomes the source of truth
  for which address should be used.

## Alternatives Rejected

- Keep source provenance in comments beside TOML rows: human-readable, but not
  parseable or enforceable.
- Commit a per-row `keccak256(eth_getCode)` digest and fail-closed-compare it:
  strongest in principle, but it guards only bytecode substitution (impossible
  for this non-upgradeable set) at far higher review/maintenance cost than any
  upstream carries. Reserved for a future upgradeable deployment.
- Let CI mutate the manifest in release mode: convenient, but erases the review
  boundary around release evidence — the probe is read-only.

## Anchors

This ADR supports the Deterministic Protocol Transforms and
Evidence-Backed Public Claims principles.

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

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The deployed-contract addresses in `crates/contracts/registry.toml` and
`crates/contracts/deployment-provenance.yaml` deserialize through the
cow-owned `#[repr(transparent)]` newtype around
`alloy_primitives::Address` per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md).

## Amendment 2026-06-01: presence probe replaces committed code-hash confirmation

The original decision committed a per-row `live_confirmation` code-hash object
and fail-closed-compared it in CI. Analysis (threat model, ecosystem baseline,
and a review-LOC / sync-churn measurement) showed that for this non-upgradeable
CREATE2 contract set the committed code hash guards a threat that cannot occur,
is heavier than any upstream (none commit code hashes), and produced per-sync
review churn. The decision now relies on the pinned `source_commit` plus a
read-only `eth_getCode` presence probe; committed code-hash confirmation is
reserved for upgradeable deployments. The `code_hash_verified`
`verification.status` denotes upstream explorer/manifest verification, not a
locally committed digest.
