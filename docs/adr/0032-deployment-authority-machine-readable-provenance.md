# ADR 0032: Deployment Authority Uses Machine-Readable Provenance

- Status: Accepted (amended)
- Date: 2026-04-29
- Last reviewed: 2026-06-08
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: deployments, provenance, contracts, release
- Anchors: Deterministic Protocol Transforms (supporting); Evidence-Backed Public Claims (supporting)
- Related: [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md), [ADR 0026](0026-alloy-major-release-absorption-plan.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

`crates/contracts/registry.toml` is the runtime address authority, keyed by
`(contract_id, chain_id, env)`. Each row records the deployed `address` and a
`verification` status and source. The upstream commit each address was taken
from is pinned once per source repository in `parity/source-lock.yaml`
(per [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md)), not
duplicated on every row.

Deployment trust rests on three layers: (1) the pinned upstream `source_commit`
in `parity/source-lock.yaml` (where deployments are explorer/Sourcify-verified),
(2) the deterministic CREATE2 address, and (3) a read-only live presence probe.

`validation-smoke registry-confirm --mode {local|release}` reads every
`(contract_id, chain_id, env, address)` row from `registry.toml`, guards each
RPC with `eth_chainId`, and asserts `eth_getCode` returns non-empty bytecode at
the recorded address. Release mode fails closed on a missing production-chain RPC
or an absent deployment. The probe never mutates a file.

Committed per-row code-hash confirmation is **not** used for the current
contract set: every deployed contract is a non-upgradeable CREATE2 singleton
whose bytecode at a fixed address cannot change, so a presence probe is the
appropriate live check. Committed code-hash confirmation is reserved for any
future upgradeable deployment.

## Why

A wrong settlement address is a wallet-draining bug. The registry row plus the
per-repository commit pin in `parity/source-lock.yaml` make every address
traceable to an upstream repository and pinned commit; the deterministic CREATE2
address plus one-time upstream explorer verification establish initial
correctness; and the live presence probe proves the claimed deployment actually
exists on-chain. This matches what every upstream CoW repository relies on
(address + source, no committed code hashes) while adding a matrix-driven live
check the upstreams lack.

A separate, per-row provenance file duplicating every registry row is declined:
its only non-redundant payload is the upstream commit, which already has one
authoritative home in `parity/source-lock.yaml`. A committed per-row code hash
would only catch bytecode *substitution* at a fixed address — impossible for
non-upgradeable CREATE2 singletons — at far higher committed-evidence and review
cost than any upstream carries, so it is declined here too.

## Must Remain True

- Every `registry.toml` row is keyed by `(contract_id, chain_id, env)` with no
  duplicates and resolves to a non-zero address.
- The upstream commit each address derives from is pinned in
  `parity/source-lock.yaml`.
- `registry-confirm --mode release` fails on a missing production-chain RPC and
  on a registry row whose `eth_getCode` is empty on the expected chain.
- The probe is read-only; it never mutates committed evidence.
- Live RPC confirms a deployment exists; it never becomes the source of truth
  for which address should be used.

## Alternatives Rejected

- Keep a dedicated per-row provenance file mirroring every registry row: its
  address and verification fields duplicate `registry.toml` and its only unique
  payload (the upstream commit) already lives in `parity/source-lock.yaml`, so
  the file is almost entirely redundant and adds per-sync review churn.
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
- `crates/contracts/tests/registry.rs`
- `scripts/validation-smoke/tests/registry_confirm.rs`

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The deployed-contract addresses in `crates/contracts/registry.toml`
deserialize through the cow-owned `#[repr(transparent)]` newtype around
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

## Amendment 2026-06-08: source-commit pin moves to source-lock; dedicated provenance file retired

The original decision kept a dedicated `crates/contracts/deployment-provenance.yaml`
that mirrored every `registry.toml` row and added per-row `source_repo`,
`source_commit`, `source_path`, and `source_symbol`. Measurement showed the file
was almost entirely redundant: its address and verification fields duplicated the
registry, and its only non-redundant payload was the upstream commit, which is
already pinned once per source repository in `parity/source-lock.yaml`
(ADR 0012). The dedicated file is retired. The upstream commit pin now has a
single authoritative home in `parity/source-lock.yaml`; the deterministic CREATE2
address and the read-only `registry-confirm` `eth_getCode` probe (now reading the
rows directly from `registry.toml`) are unchanged. This matches the upstream
posture (address + source, no committed per-row provenance file) and removes the
compile-time registry/provenance lockstep validator that previously forced the
two files to stay byte-aligned.

## Amendment 2026-06-08: registry collapsed to a const table

The committed `crates/contracts/registry.toml`, the `build.rs` schema
validator, the `deployment-coverage.yaml` manifest, and the runtime TOML
parser (`Registry::from_toml_str` and the typed `RegistryError`) are retired.
Measurement showed the manifest carried, for every contract the SDK resolves at
runtime, a single CREATE2 address repeated across every chain: `GPv2Settlement`
and `GPv2VaultRelayer` are one address each, and `CoWSwapEthFlow` is one
production and one staging address, all identical on every supported chain. The
1,595-row manifest, its compile-time and runtime validators, and the coverage
manifest therefore validated data that does not vary. `Registry` now resolves
those addresses from four committed constants behind the unchanged
`Registry::address(ContractId, chain, env)` lookup, and `ContractId` narrows to
the three runtime-resolved identifiers (`Settlement`, `VaultRelayer`, `EthFlow`).

The deployment-trust model is unchanged in substance: the upstream
`source_commit` each address derives from remains pinned per source repository
in `parity/source-lock.yaml`, the addresses remain deterministic CREATE2
singletons, and the read-only `validation-smoke registry-confirm` probe still
asserts `eth_getCode` returns non-empty bytecode at each resolved address — it
now iterates the const registry instead of reading `registry.toml`, and release
mode still fails closed on a missing production-chain RPC. The "Must Remain
True" clauses that referenced `registry.toml` rows now read against the const
table: every resolved address is non-zero, pins to a `source-lock` commit, and
is confirmed on-chain by the read-only probe.
