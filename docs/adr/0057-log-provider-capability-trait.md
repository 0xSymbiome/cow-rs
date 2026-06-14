# ADR 0057: Log-Provider Capability Trait For Event-Log Fetching

- Status: Accepted
- Date: 2026-05-29
- Last reviewed: 2026-05-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: traits, providers, events, capability-split, semver
- Anchors: Chain-RPC Runtime Neutrality (supporting)
- Related: [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0029](0029-trait-evolution-extension-traits.md), [ADR 0048](0048-composable-conditional-order-framework.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)

## Decision

`cow-sdk-core` exposes an opt-in `LogProvider: Provider` capability supertrait
for event-log fetching, mirroring the `SigningProvider: Provider` split
(ADR 0024). `Provider` keeps its frozen read-only shape; an adapter that can
serve `eth_getLogs` additionally implements
`LogProvider::get_logs(&LogQuery) -> Result<Vec<RawLog>, Self::Error>`. The
`LogQuery` / `RawLog` / `LogMeta` types live in `cow-sdk-core` with no provider
or network dependency.

`LogQuery` mirrors the standard `eth_getLogs` filter so a caller can push every
predicate down to the node in one call: an address set (single or any-of), the
four independent EVM topic slots (topic-0 = event signature; topics 1-3 = the
indexed arguments, each an any-of set with an empty slot as wildcard), and a
block selection that is either an inclusive number range or a single block hash
(`LogBlockSelector`). Because every `CoW` on-chain event indexes its actor as
the first indexed argument, the common "events for my address" query is a
topic-1 filter built with `Hash32::from_indexed_address`. `RawLog` carries the
full mined-log metadata (block number and hash, optional block timestamp,
transaction hash and index, log index) plus the reorg `removed` flag.

`get_logs` is the single bounded-call event scan: it issues exactly one backend
log query over the caller-bounded selection and returns the raw logs for the
caller to decode. It is not a watcher, iterator, or indexer loop (ADR 0048):
richer per-call filters do not loosen the single-call boundary, and the
`removed` flag serves a caller composing its own watcher from successive bounded
calls.

The fail-closed, provider-free decoders (ADR 0054, ADR 0056) stay pure;
`LogProvider` is the optional fetch seam that feeds them through
`RawLog::data`. `LogProvider` is deliberately not an `*Ext` blanket trait:
`get_logs` is a genuinely new RPC primitive not derivable from existing
`Provider` methods, so the `*Ext` pattern (ADR 0029) does not apply.

## Why

- Decoding is already pure and provider-free (ADR 0054, ADR 0056): any
  `Provider` consumer can fetch logs by other means and hand `&RawLog::data` to
  a decoder. The fetch seam is therefore a separate, optional convenience, not a
  decoding dependency.
- A capability supertrait keeps the core `Provider` minimal (ADR 0008) and lets
  a leaf crate bound `P: LogProvider` without depending on any concrete adapter
  — the exact shape the SDK already uses for `SigningProvider`. Read-only
  adapters are never forced to carry log-fetch wiring they cannot serve.
- `get_logs` is a new primitive, not derivable from `get_code` / `call` /
  `read_contract`; an `*Ext` blanket trait cannot express it. The
  capability-supertrait is the ADR-0029-consistent home for a new primitive
  (see the ADR 0029 amendment landed with this decision).
- The core traits use native `async fn` in trait and are therefore not
  object-safe, so the `dyn`-vtable rationale that motivated `*Ext` does not
  apply here. The operative forward-compatibility basis is the semver patch gate
  (ADR 0030) plus core minimalism.
- Deciding before `0.1.0` (ADR 0030 skips semver-checks at `0.1.0`) bakes the
  capability into the baseline now, so it never needs a post-freeze `*Ext`
  retrofit.
- `get_logs` is a single bounded call, honoring ADR 0048's off-chain
  orchestration boundary (no watcher loop, no rolling scan). The `event_scan`
  vocabulary is reserved for a future composable-specific fetch-and-decode
  helper (ADR 0048's deferred `ComposableCowApi`), not a core pass-through.

## Must Remain True

- `Provider`'s method set stays frozen, pinned by
  `provider_trait_shape_unchanged`; `LogProvider` adds only `get_logs`, pinned
  by `log_provider_trait_shape`.
- `LogProvider: Provider` is opt-in by bound. Read-only adapters implement only
  `Provider`; an adapter that cannot fetch logs is never required to implement
  `LogProvider`.
- The native Alloy adapters that hold a capable provider serve it: the
  read-only `RpcAlloyProvider` leaf and the composed `AlloyClient` umbrella both
  implement `LogProvider`. The umbrella reuses the leaf's `LogQuery` → filter
  and Alloy-log → `RawLog` conversions through the doc-hidden inter-crate seam
  rather than forking them.
- `LogQuery` / `RawLog` / `LogMeta` carry no provider or network dependency, and
  a decoded `RawLog::data` feeds the fail-closed decoders directly.
- `LogQuery` exposes the full `eth_getLogs` filter surface — an address set, the
  four independent topic slots, and a number-range-or-block-hash selection — so a
  consumer filters indexed arguments (the `Trade`/`OrderInvalidated`/`PreSignature`
  owner, the `Settlement` solver, the eth-flow sender/refunder) server-side
  rather than scanning chain-wide. `LogBlockSelector` is protocol-fixed and
  exhaustive (the only two `eth_getLogs` block selections).
- `LogProvider`, `LogQuery`, `RawLog`, and `LogMeta` are ungated core surfaces,
  consistent with the `SigningProvider` capability split: the trait carries no
  feature flag and no extra dependency, while the concrete implementations live
  in the feature-gated native adapter crates (`cow-sdk-alloy-provider`,
  `cow-sdk-alloy`).
- `get_logs` issues exactly one backend query over the caller-bounded block
  selection (number range or block hash) and never loops, polls, watches, or
  expands the selection.
- A genuinely new RPC primitive lands on the core read trait (while pre-`0.1.0`)
  or as a capability supertrait — never as a non-derivable blanket `*Ext`
  (ADR 0029, amended).

## Alternatives Rejected

- An `*Ext` blanket trait for `get_logs`: not expressible, because `get_logs` is
  not derivable from existing `Provider` methods; and the `*Ext` dyn-vtable
  rationale does not apply to the non-object-safe core traits.
- A `get_logs` method directly on `Provider`: universalizes a leaf capability
  onto every adapter and test mock; rejected unless log-fetch becomes a
  universal read.
- A watcher, reader, or indexer loop: violates the off-chain orchestration
  boundary (ADR 0048). `get_logs` is a single bounded call instead.
- Shipping no fetch seam at all: viable, because the decoders are provider-free,
  but a capability supertrait baked in pre-`0.1.0` is cheap and avoids a
  post-freeze retrofit.

## Links

- [Principles](../principles.md)
- [Architecture](../architecture.md)
- [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md)
- [ADR 0029](0029-trait-evolution-extension-traits.md)
- [ADR 0048](0048-composable-conditional-order-framework.md)
- [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md)
- [ADR 0056](0056-settlement-event-decoding-is-fail-closed.md)

**Proven by:**

- `crates/core/tests/traits_contract.rs` (behavioral `Provider` capability coverage; the compiler enforces the supertrait shape across impls)
- `crates/alloy/tests/log_provider_contract.rs::alloy_client_implements_log_provider_and_returns_typed_error_on_unreachable_rpc`
- [Log-Provider Capability Audit](../audit/log-provider-capability-audit.md)
