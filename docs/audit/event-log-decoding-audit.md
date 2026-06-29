---
type: Audit
id: event-log-decoding
title: "Event Log Decoding Audit"
description: "The CoWSwapOnchainOrders and GPv2Settlement log decoders parse every field fail-closed, never panic on adversarial input, and perform no I/O."
status: Current
owning_surface: "contracts on-chain event-log decoders"
related: [ADR-0054]
timestamp: 2026-06-20
---

# Event Log Decoding Audit

## Scope

Reviews the `CoWSwapOnchainOrders` and `GPv2Settlement` event-log decoders in
`cow-sdk-contracts`: topic-0 byte-locks, fail-closed field parsing, owner
resolution, UID derivation, and the shared topic-set guard. It does not cover
log retrieval (the `LogProvider` capability in the Alloy Adapters Audit) or the
ABI call-data bindings (the Contract Bindings Parity Audit).

## Findings

- Topic-0 values are byte-locked against an independent keccak-256 of the
  canonical event signatures, so a binding drift fails the lock.
- Every malformed input — bad topics, an unknown topic-0, wrong indexed arity,
  an invalid signing scheme, a bad EIP-1271 payload length, or a non-56-byte UID
  — returns a typed error; no path panics.
- Owners resolve per event (the PreSign sender, the EIP-1271 payload, the
  settlement indexed topic), and the decoded UID reuses `compute_order_uid` and
  reproduces the upstream order-hash vector.
- Each event maps into a `#[non_exhaustive]` decoded enum built from crate
  domain types.
- Decoding borrows the `LogData` and performs no network or RPC I/O, so it is
  provider-independent and wasm-safe.

## Evidence

- Decision: [ADR 0054](../adr/0054-onchain-order-event-decoding-is-fail-closed.md).
- Invariants: the `PROP-CON` family ([contracts](../properties/contracts.md)).
- Governing gate: the topic-0 byte-lock test in `crates/contracts/tests/`.
- Code: `crates/contracts/src/onchain_orders.rs`, `crates/contracts/src/settlement.rs`, `crates/contracts/src/eth_flow.rs`.
