# Log-Provider Capability Audit

Status: Current
Last reviewed: 2026-06-03
Owning surface: `cow-sdk-core` `LogProvider` capability trait, the `LogQuery` / `RawLog` / `LogMeta` log types, the single bounded-call `get_logs` contract, and the `cow-sdk-alloy-provider` leaf plus `cow-sdk-alloy` umbrella `LogProvider` implementations
Refresh trigger: a change to the `LogProvider` trait shape, the `LogQuery` / `RawLog` / `LogMeta` types, the single-call `get_logs` contract, the alloy-provider or alloy umbrella `LogProvider` implementations or their `LogQuery` / `RawLog` conversions, the provider-leaf seam entries the umbrella consumes, or the `Provider` / `SigningProvider` capability split
Related docs:
- [ADR 0057](../adr/0057-log-provider-capability-trait.md)
- [ADR 0029](../adr/0029-trait-evolution-extension-traits.md)
- [ADR 0024](../adr/0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [ADR 0037](../adr/0037-alloy-umbrella-adapter.md)
- [ADR 0048](../adr/0048-composable-conditional-order-framework.md)

## Scope

This audit covers:

- the `LogProvider: Provider` capability supertrait and its single
  `get_logs(&LogQuery) -> Result<Vec<RawLog>, Self::Error>` method
- the `LogQuery` / `RawLog` / `LogMeta` types in `cow-sdk-core`
- the single bounded-call `get_logs` scan contract
- the `RpcAlloyProvider` leaf `LogProvider` implementation and the
  `LogQuery` → Alloy filter / Alloy log → `RawLog` conversions
- the composed `AlloyClient` umbrella `LogProvider` implementation, which
  delegates to the provider it already holds and reuses the leaf's conversions
  through the doc-hidden inter-crate seam

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
| Filter expressiveness | `LogQuery` mirrors `eth_getLogs` (address set, four independent topic slots, number range or block hash) so a consumer filters indexed arguments server-side instead of scanning chain-wide | Conforms |
| Umbrella coverage | The composed `AlloyClient` implements `LogProvider` over its held provider and reuses the leaf's `LogQuery` / `RawLog` conversions through the seam, so a consumer fetches logs from the trading client without a second provider | Conforms |

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

`LogQuery` mirrors the standard `eth_getLogs` filter: an address set (single or
any-of), four independent topic slots (topic-0 = event signature, topics 1-3 =
indexed arguments, each an any-of set, empty = wildcard), and a
`LogBlockSelector` that is an inclusive number range or a single block hash. A
caller filters an indexed-address argument with `Hash32::from_indexed_address`,
so the common "events for my address" query runs server-side instead of as a
chain-wide scan. `RawLog` carries the emitting address, the
indexed-topics-plus-data payload, the reorg `removed` flag, and positional
`LogMeta` (block number and hash, optional block timestamp, transaction hash
and index, log index). None of these types depend on a provider or network, and
`RawLog::data` is handed directly to a fail-closed decoder.

### Adapter implementations

The read-only `RpcAlloyProvider` leaf implements `LogProvider` over its inner
Alloy provider. The composed `AlloyClient` umbrella implements the same
capability over the provider it already holds, so a consumer fetches event logs
from the same client it trades through rather than constructing a second
provider for the same RPC endpoint. The umbrella reuses the leaf's
`cow_log_query_to_alloy_filter` and `alloy_log_to_cow_raw_log` conversions
through the leaf's `#[doc(hidden)]` inter-crate seam rather than forking them,
keeping a single reviewed conversion shared across both adapters.

## Evidence

Primary implementation points:

- `crates/core/src/traits/log_provider.rs`
- `crates/core/src/types/logs.rs`
- `crates/alloy-provider/src/provider.rs`
- `crates/alloy-provider/src/conversion.rs`
- `crates/alloy-provider/src/lib.rs`
- `crates/alloy/src/client.rs`

Primary regression coverage:

- `crates/core/tests/trait_evolution_contract.rs::log_provider_trait_shape`
- `crates/core/tests/trait_evolution_contract.rs::provider_trait_shape_unchanged`
- `crates/core/src/types/logs.rs::tests::builders_populate_addresses_and_topic_slots`
- `crates/core/src/types/logs.rs::tests::from_indexed_address_left_pads_to_a_topic`
- `crates/alloy-provider/src/conversion.rs::tests::cow_log_query_to_alloy_filter_sets_caller_bounded_range`
- `crates/alloy-provider/src/conversion.rs::tests::cow_log_query_to_alloy_filter_maps_topics_addresses_and_block_hash`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_log_to_cow_raw_log_maps_address_meta_and_payload`
- `crates/alloy-provider/tests/seam_contract.rs::seam_exposes_log_conversions_for_the_umbrella`
- `crates/alloy/tests/log_provider_contract.rs::alloy_client_implements_log_provider_and_returns_typed_error_on_unreachable_rpc`

Validation surface:

```text
cargo test -p cow-sdk-core --lib logs
cargo test -p cow-sdk-core --test trait_evolution_contract
cargo test -p cow-sdk-alloy-provider --lib
cargo test -p cow-sdk-alloy-provider --test seam_contract
cargo test -p cow-sdk-alloy --test log_provider_contract
```
