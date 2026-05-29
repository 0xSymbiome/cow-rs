# Log-Provider Capability Audit

Status: Current
Last reviewed: 2026-05-29
Owning surface: `cow-sdk-core` `LogProvider` capability trait, the `LogQuery` / `RawLog` / `LogMeta` log types, the single bounded-call `get_logs` contract, and the `cow-sdk-alloy-provider` `LogProvider` implementation
Refresh trigger: a change to the `LogProvider` trait shape, the `LogQuery` / `RawLog` / `LogMeta` types, the single-call `get_logs` contract, the alloy-provider `LogProvider` implementation or its `LogQuery` / `RawLog` conversions, or the `Provider` / `SigningProvider` capability split
Related docs:
- [ADR 0057](../adr/0057-log-provider-capability-trait.md)
- [ADR 0029](../adr/0029-trait-evolution-extension-traits.md)
- [ADR 0024](../adr/0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [ADR 0048](../adr/0048-composable-conditional-order-framework.md)

## Scope

This audit covers:

- the `LogProvider: Provider` capability supertrait and its single
  `get_logs(&LogQuery) -> Result<Vec<RawLog>, Self::Error>` method
- the `LogQuery` / `RawLog` / `LogMeta` types in `cow-sdk-core`
- the single bounded-call `get_logs` scan contract
- the `RpcAlloyProvider` `LogProvider` implementation and the
  `LogQuery` → Alloy filter / Alloy log → `RawLog` conversions

It does not cover event-log decoding (the fail-closed decoders reviewed in the
on-chain order and settlement event log decoding audits), RPC transport, or
live log retrieval.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Capability split | `LogProvider` extends `Provider` as an opt-in supertrait; `Provider`'s method set is unchanged | Conforms |
| New-primitive rule | `get_logs` is a new RPC primitive modelled as a capability supertrait, never as a blanket `*Ext` | Conforms |
| Single-call scan | `get_logs` issues exactly one backend query over the caller-bounded block range and never loops, polls, watches, or expands the range | Conforms |
| Provider independence | `LogQuery` / `RawLog` / `LogMeta` carry no provider or network dependency | Conforms |
| Decoder feed | `RawLog::data` is the input the fail-closed decoders consume, keeping the fetch seam separate from decoding | Conforms |

## Current Contract

### Capability split

`LogProvider: Provider` mirrors the `SigningProvider: Provider` split: read-only
adapters implement only `Provider`, while an adapter that can serve `eth_getLogs`
additionally implements `LogProvider`. A leaf bounds on `P: LogProvider` to fetch
logs without depending on a concrete adapter. `Provider`'s eight-method shape is
frozen and is pinned by an unchanged trait-shape test; `LogProvider` adds only
`get_logs` and is pinned by its own trait-shape test.

### New-primitive rule

`get_logs` cannot be derived from existing `Provider` methods, so it is not an
`*Ext` blanket capability. Following the amended trait-evolution rule, a new RPC
primitive lands on the core read trait (while pre-`0.1.0`) or as a capability
supertrait; `LogProvider` is the latter. The core traits use native `async fn`
in trait and are not object-safe, so the `*Ext` dyn-vtable rationale does not
apply here.

### Single-call scan

`LogProvider::get_logs` issues exactly one backend query over the query's
caller-bounded `[from_block, to_block]` range and returns the raw logs for the
caller to decode. It is a single bounded call, not a watcher, iterator, or
indexer loop, honoring the off-chain orchestration boundary.

### Types and provider independence

`LogQuery` describes the address, topic-0 candidates, and block range for one
scan. `RawLog` carries the emitting address, the indexed-topics-plus-data
payload, and positional `LogMeta` (block number, transaction hash, log index).
None of these types depend on a provider or network, and `RawLog::data` is
handed directly to a fail-closed decoder.

## Evidence

Primary implementation points:

- `crates/core/src/traits/log_provider.rs`
- `crates/core/src/types/logs.rs`
- `crates/alloy-provider/src/provider.rs`
- `crates/alloy-provider/src/conversion.rs`

Primary regression coverage:

- `crates/core/tests/trait_evolution_contract.rs::log_provider_trait_shape`
- `crates/core/tests/trait_evolution_contract.rs::provider_trait_shape_unchanged`
- `crates/alloy-provider/src/conversion.rs::tests::cow_log_query_to_alloy_filter_sets_caller_bounded_range`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_log_to_cow_raw_log_maps_address_meta_and_payload`

Validation surface:

```text
cargo test -p cow-sdk-core --test trait_evolution_contract
cargo test -p cow-sdk-alloy-provider --lib
```
