# COW Shed App-Data Integration Audit

Status: Current
Last reviewed: 2026-06-04
Owning surface: COW Shed hook metadata emission and app-data schema integration
Refresh trigger: Refresh when COW Shed hook metadata, app-data hook schemas, or the EIP-1271 signing trait boundary change.
Related docs:
- [ADR 0049](../adr/0049-cow-shed-account-abstraction-proxy.md)
- [ADR 0050](../adr/0050-eip1271-signature-blob-encoding.md)
- [ADR 0051](../adr/0051-signing-owned-eip1271-signature-provider-trait.md)
- [COW Shed Contract Bindings Audit](cow-shed-contract-bindings-audit.md)

## Scope

This audit covers:

- the integration between the COW Shed helper crate and the existing
  app-data crate's `Hook` schema;
- the EIP-1271 signing trait boundary as it applies to COW Shed
  account-abstraction signers;
- the crate-graph posture that keeps the COW Shed helper crate as a peer
  leaf to trading rather than a dependency layer above or below it.

It does not cover the per-chain deployment evidence or proxy creation-code
artifacts; those are governed by the
[COW Shed Contract Bindings Audit](cow-shed-contract-bindings-audit.md).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Hook schema reuse | COW Shed hook metadata emits through the existing `crates/app-data/src/metadata/hooks.rs::Hook` schema via `SignedCowShedCall::to_app_data_hook`, with no parallel metadata format | Conforms |
| EIP-1271 trait boundary | Custom COW Shed signers consume the signing-owned `Eip1271SignatureProvider` trait from `cow_sdk_signing::eip1271`; no parallel trait definition exists in the COW Shed helper crate | Conforms |
| Crate-graph posture | `cow-sdk-cow-shed ⇏ cow-sdk-trading`, `cow-sdk-cow-shed ⇏ cow-sdk-orderbook`, `cow-sdk-cow-shed ⇏ cow-sdk-subgraph`, `cow-sdk-cow-shed ⇏ alloy-provider` all hold under `cargo metadata` | Conforms |
| Version forwarding discipline | The caller-selected `CowShedVersion` is threaded through every internal builder; `distinct_versions_derive_distinct_proxies` asserts distinct versions produce distinct CREATE2 proxy addresses | Conforms |

## Current Contract

### Hook schema reuse

COW Shed hook metadata reuses the existing app-data hook schema at
`crates/app-data/src/metadata/hooks.rs::Hook` and `HookList`. The COW Shed
helper crate does not define a parallel metadata format:
`SignedCowShedCall::to_app_data_hook(gas_limit)` produces the `Hook` whose
`target` is the COW Shed factory and whose `callData` is the encoded
`executeHooks` calldata. Hook entries emit into the app-data document as
ordinary hook entries; the COW Shed-specific fields (proxy address, version,
signed digest, signature bytes) live inside the hook's `callData` payload
rather than as new schema columns, preserving the app-data schema's stability.

### EIP-1271 trait boundary

Custom COW Shed account-abstraction signers consume the signing-owned
`Eip1271SignatureProvider` trait at `cow_sdk_signing::eip1271`. The COW
Shed helper crate does not define a parallel trait; it imports the
canonical signing path. Trading-side call sites that surface signature
failures use inline `map_err` per ADR 0051; no blanket
`From<Eip1271SignatureError> for TradingError` bridge exists anywhere in
the workspace.

### Crate-graph posture

The COW Shed helper crate depends on `cow-sdk-core`, `cow-sdk-contracts`,
`cow-sdk-signing`, `cow-sdk-app-data`, and `cow-sdk-pure-helpers`. The
negative-edge invariants `cow-sdk-cow-shed ⇏ cow-sdk-trading`,
`cow-sdk-cow-shed ⇏ cow-sdk-orderbook`,
`cow-sdk-cow-shed ⇏ cow-sdk-subgraph`, and
`cow-sdk-cow-shed ⇏ alloy-provider` hold under `cargo metadata`. The
COW Shed helper crate is a peer leaf to trading rather than a dependency
layer above or below it; embedding the helper crate behind the facade-level
`cow-shed` feature keeps the default `cow-sdk` dependency closure free of
COW Shed types.

### Version forwarding discipline

The caller-selected `CowShedVersion` is threaded through every internal
builder. The SDK signs and derives proxy addresses against deployed reality
(`V1_0_1`) by default; `distinct_versions_derive_distinct_proxies`
(`crates/cow-shed/src/address/mod.rs`) asserts that distinct `CowShedVersion`
variants produce distinct CREATE2 proxy addresses for the same user, so the
version selected at construction is never dropped before signing.

## Evidence

Primary implementation points:

- `docs/adr/0049-cow-shed-account-abstraction-proxy.md`
- `docs/adr/0050-eip1271-signature-blob-encoding.md`
- `docs/adr/0051-signing-owned-eip1271-signature-provider-trait.md`
- `crates/app-data/src/metadata/hooks.rs` (existing hook schema)
- `crates/signing/src/eip1271/` (signing-owned trait home)
- `crates/cow-shed/` (leaf crate)
- `crates/cow-shed/src/hooks.rs` (`SignedCowShedCall::to_app_data_hook`)

Primary regression coverage:

- `cargo metadata --format-version 1` proves the four negative-edge
  invariants
- `crates/trading/tests/eip1271_signature_provider_no_reexport.rs`
  (compile-fail regression for the trading re-export contract)
- `crates/cow-shed/src/address/mod.rs::distinct_versions_derive_distinct_proxies`
  (per-version distinct-proxy regression)

Validation surface:

```text
cargo test -p cow-sdk-app-data --all-features
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- check-deps --negative-edge cow-sdk-cow-shed::cow-sdk-trading
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- check-deps --negative-edge cow-sdk-cow-shed::cow-sdk-orderbook
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- check-deps --negative-edge cow-sdk-cow-shed::alloy-provider
```
