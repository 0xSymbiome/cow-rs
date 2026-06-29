---
type: Decision Record
id: ADR-0054
title: "ADR 0054: On-Chain Event Decoding Is Fail-Closed And Provider-Free"
description: "The CoWSwapOnchainOrders event decoder in cow-sdk-contracts decodes OrderPlacement and OrderInvalidation logs through alloy::sol!-generated event types and is fail-closed."
status: Accepted
date: 2026-05-28
last_reviewed: 2026-05-28
authors: ["0xSymbiotic"]
tags: [contracts, bindings, events, decoding, defense-in-depth]
related: [ADR-0012, ADR-0020, ADR-0052]
timestamp: 2026-05-28T00:00:00Z
---

# ADR 0054: On-Chain Event Decoding Is Fail-Closed And Provider-Free

## Decision

The `CoWSwapOnchainOrders` event decoder in `cow-sdk-contracts` decodes
`OrderPlacement` and `OrderInvalidation` logs through `alloy::sol!`-generated
event types and is fail-closed. It accepts borrowed log bytes
(`alloy_primitives::LogData`) with no `Provider` or network dependency,
validates the topic set against the generated `SIGNATURE_HASH` and the indexed
arity before ABI decoding, range-checks the on-chain signing scheme,
length-checks the EIP-1271 owner payload, and maps every `GPv2` order marker
through the canonical label tables. Every malformed input returns a typed
`ContractsError`; no log, however adversarial, can panic the decoder. The same
fail-closed, provider-free posture extends to the `GPv2Settlement` event family
through `decode_settlement_log` (see Must Remain True).

## Why

On-chain logs are untrusted input. A decoder that slices a signature payload
without a length check, or reads a topic array without a count check, turns a
malformed or hostile log into a panic — a denial of service for any indexer,
watcher, or wallet that feeds it chain data. Borrowing the log bytes instead of
taking a `Provider` keeps the decoder transport-agnostic and wasm-safe, so one
implementation serves native, browser, and any RPC client. Reusing the
byte-locked order hashing keeps a decoded order's UID identical to the one the
settlement contract derives.

## Must Remain True

- Public surface: `decode_order_placement` and `decode_order_invalidation` take
  `&LogData` and return `Result<_, ContractsError>`; `OnchainOrderPlacement`
  exposes owner resolution and UID derivation as separate fallible steps. No
  decoding path takes a `Provider` or performs I/O.
- Runtime and support: the decoder depends only on `alloy::sol!` and
  `alloy-primitives` — wasm-safe, no tokio, no network. UID derivation reuses
  `compute_order_uid` and the canonical `Registry` settlement domain.
- Validation and review: topic count, topic-0, signing-scheme range, EIP-1271
  payload length, and order-marker resolution are each covered by a fail-closed
  test; a fuzz target asserts the decoders never panic on arbitrary `LogData`;
  topic-0 bytes are locked against an independent keccak of the canonical event
  signature, and the order hash is locked against an upstream contract vector.
- Cost: a decoded `GPv2` order carrying an unrecognized kind or balance marker
  is rejected rather than silently coerced, so a future marker addition is a
  deliberate code change.
- EthFlow refund + dispatcher: the same fail-closed, provider-free posture covers
  the `CoWSwapEthFlow` `OrderRefund` event (`decode_order_refund`) and a unified
  `decode_eth_flow_log` dispatcher that routes the `OrderPlacement` /
  `OrderInvalidation` / `OrderRefund` topic-0 to the matching decoder and returns
  the typed `#[non_exhaustive]` `EthFlowEvent`; the `OrderRefund` interface is a
  dedicated `ICoWSwapEthFlowEvents` `sol!` block.
- Settlement events: `decode_settlement_log` takes `&LogData` and returns
  `Result<SettlementEvent, ContractsError>` over the `GPv2Settlement` `Trade` /
  `Interaction` / `Settlement` / `OrderInvalidated` / `PreSignature` logs,
  validating the topic set through the shared `check_topics` guard and
  length-checking the 56-byte order UID before ABI decoding. `SettlementEvent`
  is `#[non_exhaustive]` and `upstream-growing` in `enum-policy.yaml`; the
  decoder takes no `Provider` and never panics on adversarial input.

## Alternatives Rejected

- Panic on malformed input (slice or index without guards): simpler, but turns
  hostile chain data into a crash for every consumer.
- Take a `Provider` and resolve the UID eagerly inside the decoder: couples a
  pure codec to a network client, breaks wasm-safety, and forces a chain
  round-trip on every decode.
- Decode topic-0 by hand: duplicates what `alloy::sol!` already derives and
  invites drift from the canonical signature hash.

## Links

- [ADR 0012](0012-alloy-sol-bindings-and-registry-authority.md)
- [ADR 0020](0020-ethflow-owner-threading.md)
- [Architecture](../guides/architecture.md)
- [Parity Matrix](../guides/parity.md)

**Proven by:**

- [Event Log Decoding Audit](../audit/event-log-decoding-audit.md)
