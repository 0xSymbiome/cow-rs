# Deployment Registry Audit

Status: Current
Last reviewed: 2026-04-29
Owning surface: `cow-sdk-contracts::Registry` typed deployment authority and its embedded TOML manifest
Refresh trigger: Changes to the `(ContractId, SupportedChainId, CowEnv)` key shape, the `registry.toml` schema, the embedded manifest, the compile-time validator in `build.rs`, or the runtime parser in `Registry::from_toml_str`; a new deployed address or a new supported chain
Related docs:
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0032](../adr/0032-deployment-authority-machine-readable-provenance.md)
- [Deployments](../deployments.md)
- [Architecture](../architecture.md)
- [Parity Matrix](../parity-matrix.md)

## Scope

This audit covers:

- the `Registry` type shape and public API (`default`, `from_toml_str`,
  `address`, `with_override`, `entries`, `len`, `is_empty`)
- the embedded `registry.toml` manifest committed at
  `crates/contracts/registry.toml`
- the compile-time validator in `build.rs` and the runtime parser in
  `Registry::from_toml_str`
- the typed `RegistryError` surface returned by the runtime parser
- the single-authority posture: no shipped crate reads or writes a
  deployment address outside this path

It does not cover binding generation itself (a separate audit), partner
API routing, or chain-RPC resolution through `AsyncProvider`.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Single authority | Every deployed-address resolution in the workspace routes through `Registry::address` or `Registry::with_override` | Conforms |
| Compile-time validation | `build.rs` rejects malformed rows (bad hex, duplicate key, unsupported chain, wrong schema version) before the crate builds | Conforms |
| Typed runtime failures | `RegistryError` variants mirror the compile-time failure classes so downstream manifests surface the same contract | Conforms |
| Override composability | `with_override` layers a local deployment on top of `Registry::default` without discarding the embedded manifest | Conforms |

## Current Contract

### Lookup Surface

`Registry::address(contract_id, chain_id, env)` returns
`Option<Address>` for a typed
`(ContractId, SupportedChainId, CowEnv)` triple. The backing store is a
`BTreeMap` so iteration through `Registry::entries` is deterministic and
audit diffs remain stable. `Registry::default()` loads the embedded
`registry.toml` and `.expect()`s well-formedness; the embedded manifest
is gated by `build.rs` so this expect is unreachable in a released
build.

### Embedded Manifest

The canonical manifest lives at `crates/contracts/registry.toml` and is
embedded through `include_str!`. The schema carries a `schema_version`
integer and one row per `(contract_id, chain_id, env, address)` entry.
The manifest covers `Settlement`, `VaultRelayer`, and `EthFlow` across
every supported chain and both production `CowEnv` values.

### Compile-Time And Runtime Validation

`build.rs` runs the same validator as the runtime parser through a
shared code path, so a malformed manifest fails the crate build rather
than a runtime request. Validation classes rejected by both gates:

- unsupported schema version
- unknown contract identifier
- chain id outside the `SupportedChainId` domain
- malformed 20-byte hex address
- duplicate `(contract, chain, env)` key

Downstream consumers loading their own manifest through
`Registry::from_toml_str` see the same classes as typed
`RegistryError::UnsupportedSchemaVersion`,
`RegistryError::UnsupportedChainId`, `RegistryError::InvalidAddress`,
and `RegistryError::DuplicateEntry` variants. `RegistryError::Parse`
carries the typed `toml::de::Error` through a boxed source chain.

### Override Composition

`Registry::with_override(contract_id, chain_id, env, address)` returns
a registry with a single entry replaced. Consumers layer
test-net overrides, fork-specific deployments, or integration-test
fixtures by calling `Registry::default().with_override(...)` and
resolving through the same `Registry::address` API.

## Evidence

Primary implementation points:

- `crates/contracts/src/deployments/mod.rs`
- `crates/contracts/src/deployments/registry.rs`
- `crates/contracts/src/deployments/contract_id.rs`
- `crates/contracts/registry.toml`
- `crates/contracts/build.rs`

Primary regression coverage:

- `crates/contracts/tests/registry.rs`
- `crates/contracts/tests/build_rs_compile_fail.rs`

Validation surface:

```text
cargo test -p cow-sdk-contracts --all-features
cargo clippy -p cow-sdk-contracts --all-targets --all-features -- -D warnings
```
