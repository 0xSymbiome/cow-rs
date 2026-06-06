# Public API Naming Convention Audit

Status: Current
Last reviewed: 2026-06-06
Owning surface: public method naming across the SDK crates
Refresh trigger: a new public method whose name uses a `get_` prefix outside the chain-RPC `Provider`/`LogProvider` traits, or a change to that trait method set
Related docs:
- [ADR 0067](../adr/0067-idiomatic-accessor-naming.md)
- [ADR 0035](../adr/0035-alloy-provider-adapter.md)

## Scope

This audit covers the accessor-naming convention across the public API of the SDK crates:
the orderbook, trading, app-data, subgraph, and signing surfaces, and the chain-RPC
`Provider`, `LogProvider`, and `Signer` traits.

It does not cover non-naming API design, nor the TypeScript export names, which are fixed
by the wasm bindings and are out of scope for the Rust accessor convention.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Domain accessors | Public getters and domain fetches use bare nouns, no `get_` prefix | Conforms |
| Chain-RPC seam | `Provider` and `LogProvider` retain `get_` only as `eth_get*` mirrors | Conforms |
| Signer address | The signer address accessor is `address()` | Conforms |

## Current Contract

### Domain accessors

Public accessors and domain fetches on the orderbook, trading, app-data, subgraph, and
signing surfaces use bare domain nouns and carry no `get_` prefix, matching the Rust API
Guidelines C-GETTER convention and the upstream `cowprotocol/services` model.

### Chain-RPC seam

The `Provider` and `LogProvider` traits retain `get_` only where the method mirrors a
canonical `eth_get*` JSON-RPC name. These are fallible keyed lookups, not field getters,
per ADR 0035. Non-fetch operations on the seam stay bare (`call`, `read_contract`).

### Signer address

The signer address accessor is `address()`, matching the single-obvious-value rule and the
upstream `alloy` signer surface.

## Evidence

Primary implementation points:

- `crates/orderbook/src/api.rs`
- `crates/trading/src/`
- `crates/subgraph/src/api.rs`
- `crates/app-data/src/info.rs`
- `crates/core/src/traits/provider.rs`
- `crates/core/src/traits/signer.rs`

Primary regression coverage:

- the public-api surface snapshot tests under `crates/sdk/tests`
- the per-crate contract tests under each crate's `tests/`

Validation surface:

```text
cargo test --workspace
cargo doc --workspace --no-deps
```
