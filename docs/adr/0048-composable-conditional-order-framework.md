# ADR 0048: Composable Conditional Order Framework

- Status: Accepted (amended)
- Date: 2026-05-15
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: composable, conditional-orders, off-chain-orchestration, watch-tower-boundary
- Related: [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0049](0049-cow-shed-account-abstraction-proxy.md), [ADR 0050](0050-eip1271-signature-blob-encoding.md), [ADR 0051](0051-signing-owned-eip1271-signature-provider-trait.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Context

The CoW Protocol composable-order surface defines a conditional-order framework
on top of the `ComposableCoW` registry contract. Conditional orders are
not posted by the user at trade time; instead, the user pre-authorizes a
conditional-order handler (TWAP, GoodAfterTime, StopLoss,
TradeAboveThreshold, PerpetualStableSwap, or a custom handler), and an
off-chain watcher polls the on-chain framework to discover when the next
constituent order becomes tradeable. Production deployments rely on a
cowprotocol-operated watcher service for discovery and orderbook posting.

A Rust SDK for composable orders has two natural shapes that must be kept
separate. The first is a library of typed encoders, decoders, custom-error
selector constants, single-call provider operations, and an offline simulator —
primitives that any consumer can compose into their own watcher. The second is
a production watcher service that maintains block subscriptions, persistent
registry storage, retry timers, notification systems, and automatic order
posting credentials. Shipping the second inside the SDK turns the library into
a service and absorbs operational concerns that belong to the cowprotocol-
operated watcher or to consumers who build a self-hosted watcher on top of the
primitives.

The composable surface also intersects the COW Shed account-abstraction proxy
(see [ADR 0049](0049-cow-shed-account-abstraction-proxy.md)) and the EIP-1271
signature blob encoders (see [ADR 0050](0050-eip1271-signature-blob-encoding.md)).
Those decisions are coupled because composable conditional orders are
EIP-1271-authenticated through the registered handler, and the COW Shed
account-abstraction proxy is a separately deployed forwarder that the
composable framework can target on Gnosis Chain only.

## Decision

`cow-sdk-composable` is an additive leaf crate per ADR 0008. The crate is
opt-in behind the facade-level `composable` feature and is never on the
default `cow-sdk` dependency closure. The crate's public surface is bound by
the watch-tower boundary stated below.

### Watch-Tower Boundary

The watch-tower boundary is binding. `cow-sdk-composable` DOES expose:

- custom-error selector constants for the five conditional-order poll outcomes
  plus the seven Rust-side decode, validation, and provider errors of the
  `PollResult` taxonomy;
- ABI decode helpers for those poll errors;
- the `#[non_exhaustive]` `PollResult` classification enum;
- selectors and pure offline encoders and decoders for
  `ConditionalOrderParams`, `GPv2Order.Data`, signature blobs (per
  [ADR 0050](0050-eip1271-signature-blob-encoding.md)), and merkle leaves;
- the single-call `ComposableCowApi::poll_async` over an injected
  `Provider` (one `eth_call` per invocation);
- `event_scan_async` as a single-call provider operation over a
  caller-bounded block range (one `eth_getLogs` per invocation);
- the local poll simulator `local_poll_async` that replays a `PollResult`
  from a captured `(owner, params, offchainInput, proof, block_state)` tuple
  without any RPC; and
- the reference watcher example crate (out of the published library) showing
  how an external service composes these primitives into a watcher.

`cow-sdk-composable` DOES NOT expose:

- service loops;
- persistence adapters (no Redis, no Postgres, no on-disk registry);
- notification or alerting systems;
- automatic order posting (the published `OrderbookExt::post_composable_order`
  is a blanket-impl helper that wraps the existing
  `OrderbookClient::send_order` and the caller controls when to invoke it);
- global retry cadence policy as default behavior;
- chain event indexing beyond the single-call `event_scan_async`;
- production watch-tower state machines; or
- any `tokio::spawn`, `wasm_bindgen_futures::spawn_local`, or background task.

The cowprotocol-operated watcher remains canonical for production discovery
and orderbook posting. Consumers who want a self-hosted watcher build it from
the crate's primitives inside their own runtime; the reference example crate
is documentation, not a published library.

### Ergonomic Surface

Five first-release conditional-order types ship as typed builders with
typestate-enforced required-field discipline: `Twap`, `GoodAfterTime`,
`StopLoss`, `TradeAboveThreshold`, and `PerpetualStableSwap`. Each builder
runs pre-flight validation that mirrors the Solidity revert sites of the
corresponding handler contract; the typed errors form the `ComposableError`
enum.

The `Multiplexer` merkle helper uses OpenZeppelin double-hashed leaves and
sorted-pair internal nodes. Params hashing uses `abi.encode` (never
`abi.encodePacked`). Twelve custom-error selectors are byte-identical to
`forge methodIdentifiers` output. EIP-1271 dual blob encoders cover Shape A
(Safe-muxer with `safeSignature(...)` selector prefix) and Shape B (raw
`ERC1271Forwarder` order + payload tuple) per
[ADR 0050](0050-eip1271-signature-blob-encoding.md).

### Crate-Graph Invariants

`cow-sdk-composable` depends on `cow-sdk-core`, `cow-sdk-contracts`,
`cow-sdk-signing`, `cow-sdk-orderbook`, and `cow-sdk-pure-helpers`. It MUST
NOT depend on `cow-sdk-trading`, `alloy-provider`, or `alloy-signer`. The
negative-edge invariants `cow-sdk-composable ⇏ cow-sdk-trading` and
`cow-sdk-composable ⇏ alloy-provider` are asserted via `cargo metadata` and
the `parity-maintainer check-deps` validator in CI. An optional
`composable-with-cow-shed` feature lifts a non-default dependency on
`cow-sdk-cow-shed` for the narrow Gnosis-only `COWShedForComposableCoW`
forwarder flow.

## Why

A library that silently embeds a watch-tower loop becomes a service: it owns
block subscriptions, persistent registry storage, retry timers, and orderbook
credentials. The composable strategy ships typed encoders, on-chain reads, and
a local simulator — primitives that compose into a watcher when the consumer
chooses, but never a watcher by default. The DOES / DOES NOT lists make the
boundary concrete: reviewers can cite the list rather than reasoning from
first principles each time a new feature lands.

The off-chain orchestration boundary is also a portability boundary. The
composable crate is runtime-neutral per [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md);
embedding `tokio::spawn` or `wasm_bindgen_futures::spawn_local` would tie the
crate to a runtime and break the wasm32 target.

The negative-edge invariant against `cow-sdk-trading` keeps the dependency
graph additive per [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md):
composable is a peer leaf to trading, not a layer above or below it. The
invariant prevents a future change from making composable a transitive
dependency of every facade consumer.

## Must Remain True

- Public surface: every item in the DOES list is reachable behind the
  facade-level `composable` feature; no item in the DOES NOT list is reachable
  at any feature combination.
- Runtime and support: no `tokio::spawn` site, no `wasm_bindgen_futures::spawn_local`
  site, no `start()` or `run_forever()` method, no `Storage` trait
  implementation, no Slack / Discord / email / webhook hook, and no internal
  call to `OrderbookClient::create_order` may appear in `cow-sdk-composable`.
- Crate graph: `cargo metadata` continues to prove
  `cow-sdk-composable ⇏ cow-sdk-trading` and
  `cow-sdk-composable ⇏ alloy-provider` (default features). The
  reverse-edge guard `cow-sdk-orderbook ⇏ cow-sdk-composable` continues to
  hold.
- Validation and review: the composable contract bindings audit and the
  composable watch-tower boundary audit cross-link this ADR as the governing
  decision. Both audits stay `Current` whenever the audited surface moves.
- Cost: any future PR that adds a service loop, a persistence adapter, a
  notification system, an automatic order poster, a global retry cadence, a
  chain event indexer beyond `event_scan_async`, or any background task into
  `cow-sdk-composable` violates this ADR and must be rejected at review.

## Alternatives Rejected

- Embed a production watcher service inside `cow-sdk-composable`: this would
  absorb operational concerns (persistence, notifications, retry cadence,
  orderbook credentials) that belong to the cowprotocol-operated watcher or
  to consumers building a self-hosted watcher.
- Surface only `local_poll_async` and omit `poll_async`: the offline simulator
  alone cannot answer "is this order tradeable now"; consumers would still
  need a single-call probe. Splitting the answer into two crates removes a
  coherent boundary.
- Merge `cow-sdk-composable` into `cow-sdk-trading`: this would force every
  trading-facade consumer to depend transitively on composable encoders and
  break the additive-leaf-crates discipline.
- Ship without typed pre-flight validation: validation errors would surface
  only as Solidity reverts at simulation time, losing the ergonomic
  builder-time feedback that distinguishes Rust-over-TS.

## Links

- [Architecture](../architecture.md)
- [Principles](../principles.md)
- [Composable Contract Bindings Audit](../audit/composable-contract-bindings-audit.md)
- [Composable Watch-Tower Boundary Audit](../audit/composable-watch-tower-boundary-audit.md)
- [ADR 0049](0049-cow-shed-account-abstraction-proxy.md)
- [ADR 0050](0050-eip1271-signature-blob-encoding.md)
- [ADR 0051](0051-signing-owned-eip1271-signature-provider-trait.md)

**Proven by:**

- [Composable Contract Bindings Audit](../audit/composable-contract-bindings-audit.md)
- [Composable Watch-Tower Boundary Audit](../audit/composable-watch-tower-boundary-audit.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The `cow-sdk-composable` crate is not yet rooted in the workspace
members list and is deferred to a later capability landing. The
prescribed shape above anchors to the canonical primitive layer per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). When
the crate lands per the watch-tower boundary above, the
`#[non_exhaustive]` `PollResult` classification enum sources its
custom-error selector constants from `alloy_sol_types::SolError::SELECTOR`
and routes poll-error decoding through `alloy_sol_types::SolInterface`;
the `Multiplexer` merkle helper routes through
`rs_merkle::MerkleTree` with an `OzSortedPairKeccakHasher` wrapper that
preserves the OpenZeppelin double-hashed leaves and sorted-pair
internal-node contract; params hashing through
`alloy_sol_types::SolValue::abi_encode` matches `abi.encode(...)`
byte-for-byte; and the byte-typed identity parameters flowing through
the composable encoders resolve through the cow-owned
`#[repr(transparent)]` newtypes per ADR 0052.
