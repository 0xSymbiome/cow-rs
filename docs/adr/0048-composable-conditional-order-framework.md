---
type: Decision Record
id: ADR-0048
title: "ADR 0048: Composable Conditional Order Framework"
description: "Composable ships as the off-by-default composable feature on cow-sdk-contracts, surfaced under cow_sdk_contracts::composable and re-exported through the cow-sdk facade's composable feature."
status: Accepted (TWAP shipped in `cow-sdk-contracts`; broader handler taxonomy deferred)
date: 2026-05-15
last_reviewed: 2026-06-26
authors: ["0xSymbiotic"]
tags: [composable, conditional-orders, twap, off-chain-orchestration, watch-tower-boundary]
related: [ADR-0001, ADR-0010, ADR-0024, ADR-0049, ADR-0050, ADR-0052, ADR-0054, ADR-0070]
timestamp: 2026-06-26T00:00:00Z
---

# ADR 0048: Composable Conditional Order Framework

## Context

The CoW Protocol composable-order surface defines a conditional-order framework
on top of the `ComposableCoW` registry contract. Conditional orders are not
posted by the user at trade time; instead, the user pre-authorizes a
conditional-order handler (TWAP, GoodAfterTime, StopLoss, TradeAboveThreshold,
PerpetualStableSwap, or a custom handler), and an off-chain watcher discovers
when the next constituent order becomes tradeable and posts it. Production
deployments rely on a cowprotocol-operated watcher service for discovery and
orderbook posting.

A Rust SDK for composable orders has two natural shapes that must be kept
separate. The first is a library of typed encoders, hashes, a merkle helper, and
a pure schedule classifier — primitives any consumer composes into their own
watcher. The second is a production watcher service that maintains block
subscriptions, persistent registry storage, retry timers, notification systems,
and automatic order posting credentials. Shipping the second inside the SDK
turns the library into a service and absorbs operational concerns that belong to
the cowprotocol-operated watcher or to a consumer who builds a self-hosted
watcher on top of the primitives.

The owner of a conditional order is a smart-contract account that authenticates
through EIP-1271 (a Safe with the `ExtensibleFallbackHandler`, or a custom
forwarder), never an externally owned account. The composable surface therefore
intersects the EIP-1271 signature encoders (see
[ADR 0050](0050-eip1271-signature-blob-encoding.md)); the COW Shed
account-abstraction proxy (see [ADR 0049](0049-cow-shed-account-abstraction-proxy.md))
is a separately deployed forwarder the framework can target on Gnosis Chain.

## Decision

Composable ships as the off-by-default `composable` feature on
`cow-sdk-contracts`, surfaced under `cow_sdk_contracts::composable` and
re-exported through the `cow-sdk` facade's `composable` feature. It is **not** a
separate `cow-sdk-composable` crate — this reverses the earlier separate-crate
plan.

The shipped surface is TWAP encoding, identity, a merkle multiplexer, and a pure
schedule classifier: a pure-transform peer to the eth-flow encoders, which
already live in `cow-sdk-contracts`. A separate crate would duplicate the
`sol!` bindings, the primitive seam, and the transaction-helper boundary for a
surface that is, today, contract call-data construction. Hosting it as a
feature-module mirrors `cow_shed` (also an off-by-default feature-module in
`cow-sdk-contracts`) and keeps the dependency graph additive without a new crate.

The bindings are authored inline as `alloy::sol!` against the on-chain
`ComposableCoW` / TWAP Solidity surface, pinned by commit in
`parity/source-lock.yaml`.

### Watch-Tower Boundary

The watch-tower boundary is binding. `cow_sdk_contracts::composable` DOES expose:

- the typed `ConditionalOrderParams` and `conditional_order_id`, byte-identical
  to the on-chain `ComposableCoW.hash(params)` (`keccak256(abi.encode(params))`);
- the `create` / `createWithContext` / `remove` call-data encoders;
- the `TwapData` builder, the validated per-part `TwapStaticInput` (the 320-byte
  handler input), and the `twap_create_transaction` / `twap_remove_transaction`
  gas-free `UnsignedTransaction` builders (per
  [ADR 0070](0070-onchain-transaction-helper-boundary.md)), where
  `twap_create_transaction` routes start-at-mining-time orders through
  `createWithContext` and start-at-epoch orders through `create`;
- the hand-rolled `Multiplexer` merkle helper, `merkle_leaf`, and
  `verify_merkle_proof` over an order's params, for the `setRoot` batch path; and
- the pure `TwapStaticInput::timing_at` classifier returning `TwapTiming`, which
  mirrors `TWAPOrderMathLib.calculateValidTo` — it selects the live discrete part
  from a TWAP's schedule with no RPC.

`cow_sdk_contracts::composable` DOES NOT expose (now or under any feature):

- a revert decoder or poll-error selector constants. The TWAP handler reverts
  with a generic `IConditionalOrder.OrderNotValid(string)` carrying no epoch, and
  the `Provider` seam collapses a revert to an opaque error — there is nothing for
  a decoder to read, so discovery timing is read from the schedule instead;
- a provider-consuming `poll` or `event_scan` wrapper. Discovery reads
  (`cabinet`, `getTradeableOrderWithSignature`) are ordinary `read_contract`
  calls a consumer composes with `timing_at` through their own `Provider`/RPC;
- service loops, persistence adapters, notification systems, automatic order
  posting, a global retry cadence, chain event indexing, or any background task
  (`tokio::spawn` / `wasm_bindgen_futures::spawn_local`).

The cowprotocol-operated watcher remains canonical for production discovery and
orderbook posting. A consumer who wants a self-hosted watcher composes it from
`timing_at`, their own reads, and the existing `OrderbookClient::send_order`.

### Ergonomic Surface

TWAP ships first as a typed builder with build-time validation that mirrors the
Solidity revert sites of the TWAP handler, plus divisibility guards so totals are
never silently floored; the typed errors form `TwapValidationError`. The other
handler types (GoodAfterTime, StopLoss, TradeAboveThreshold, PerpetualStableSwap,
custom) are deferred to demand and land additively under the same module and
boundary.

The `Multiplexer` merkle helper is hand-rolled: double-hashed leaves
(`keccak256(keccak256(abi.encode(params)))`, the form `ComposableCoW._auth`
checks) and sorted-pair keccak internal nodes, matching the on-chain
`OpenZeppelin MerkleProof.verify`. Params hashing uses `abi.encode` of the
params struct — carrying Solidity's dynamic-struct offset, the form the contract
hashes — never `abi.encodePacked` or a bare field tuple.

## Why

A library that silently embeds a watch-tower loop becomes a service: it owns
block subscriptions, persistent storage, retry timers, and orderbook
credentials. This module ships typed encoders, a merkle helper, and a pure
schedule classifier — primitives that compose into a watcher when the consumer
chooses, but never a watcher by default. The DOES / DOES NOT lists make the
boundary concrete: reviewers cite the list rather than reasoning from first
principles each time a feature lands.

Reading discovery timing from the schedule is forced by the contract and the
transport seam: the handler reverts with a hint-free `OrderNotValid`, and the
`Provider` trait does not surface revert bytes. `timing_at` reproduces the
handler's own `calculateValidTo` arithmetic, so a consumer answers "which part is
live now, and when is the next one" without an RPC, and reads only to fetch the
order to post.

Hosting composable as a feature-module rather than a crate keeps the surface a
peer to the other `cow-sdk-contracts` encoders (eth-flow, settlement, cow-shed)
without a new crate boundary, and keeps it runtime-neutral per
[ADR 0010](0010-runtime-neutral-async-and-transport-posture.md): no `Provider`,
no runtime, so the module builds on every target including wasm32.

## Must Remain True

- Public surface: the items in the DOES list are reachable behind the
  `composable` feature; no item in the DOES NOT list is reachable at any feature
  combination.
- No runtime or service: no `tokio::spawn`, no `wasm_bindgen_futures::spawn_local`,
  no `start()` / `run_forever()`, no persistence trait, no notification hook, and
  no internal orderbook write may appear in the module.
- Contract fidelity: `conditional_order_id` equals the on-chain
  `ComposableCoW.hash`; the merkle root and proofs verify under the on-chain
  sorted-pair double-hashed algorithm; `TwapStaticInput` matches the upstream
  `TWAPOrder.Data` field order as a 320-byte static input; `timing_at` matches
  `TWAPOrderMathLib.calculateValidTo`. Each is bound to the pinned
  `source_commit` in `parity/source-lock.yaml`, and tracked by `PROP-CON-026`.
- Pinned binding: the module binds only the contract pinned in
  `parity/source-lock.yaml`, confirmed on-chain per the deployment-authority rule.

## Alternatives Rejected

- A separate `cow-sdk-composable` crate: for a TWAP-encoding surface this
  duplicates the binding and primitive stack of `cow-sdk-contracts` and adds a
  crate boundary with no isolation benefit; the feature-module is the lean peer
  to eth-flow and cow-shed.
- An SDK-side revert decoder: the handler emits a hint-free `OrderNotValid` and
  the `Provider` seam yields no revert bytes, so the decoder would have no input.
- A provider-consuming `poll` wrapper: the pure `timing_at` plus the consumer's
  own reads compose the same answer while keeping the module provider-free and
  runtime-neutral, matching the pure-pieces-plus-consumer-fetch posture of
  [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md).
- Shipping without build-time validation: validation errors would surface only
  as Solidity reverts at simulation time, losing the builder-time feedback that
  distinguishes the Rust surface.
- A maintained merkle crate (`rs_merkle` or similar) for the `Multiplexer`: a
  generic merkle crate pairs siblings in fixed index order and carries a lone odd
  node up unchanged, so its proofs do not fold under `OpenZeppelin`'s sorted-pair
  `MerkleProof.verify`; a custom hasher recovers the pairing but not the odd-node
  handling, so any non-power-of-2 batch fails on-chain. No Rust crate implements
  the contract's verifier, so the helper is hand-rolled (no dependency).
- Building the tree as `@openzeppelin/merkle-tree`'s `StandardMerkleTree` (the
  shape the TypeScript SDK uses): `StandardMerkleTree` ABI-encodes the leaf as a
  bare `(address, bytes32, bytes)` tuple, but `ComposableCoW.hash` encodes the
  `ConditionalOrderParams` struct, which Solidity prepends with the dynamic-struct
  `0x20` offset. The two leaves differ, so a `StandardMerkleTree` root does not
  authenticate on-chain. The hand-rolled tree uses the contract's struct-encoded
  leaf, pinned to it by a golden-vector test against an independent ABI encoding.

## Links

- [Architecture](../guides/architecture.md)
- [Principles](../principles/index.md)
- [ADR 0049](0049-cow-shed-account-abstraction-proxy.md)
- [ADR 0050](0050-eip1271-signature-blob-encoding.md)
- [ADR 0054](0054-onchain-order-event-decoding-is-fail-closed.md)
- [ADR 0070](0070-onchain-transaction-helper-boundary.md)
