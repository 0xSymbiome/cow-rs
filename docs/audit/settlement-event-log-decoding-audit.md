# Settlement Event Log Decoding Audit

Status: Current
Last reviewed: 2026-05-29
Owning surface: `cow-sdk-contracts` `GPv2Settlement` event decoder
Refresh trigger: a change to the `Trade` / `Interaction` / `Settlement` / `OrderInvalidated` / `PreSignature` event ABI, the `SettlementEvent` domain enum, or the shared `check_topics` topic-set guard
Related docs:
- [ADR 0056](../adr/0056-settlement-event-decoding-is-fail-closed.md)
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [Parity Matrix](../parity.md)

## Scope

This audit covers:

- the `alloy::sol!` `IGPv2SettlementEvents` event bindings (`Trade`, `Interaction`, `Settlement`, `OrderInvalidated`, `PreSignature`) and their topic-0 signature hashes
- `decode_settlement_log` and the `SettlementEvent` domain mapping
- the shared fail-closed `check_topics` topic-set guard and the 56-byte order-UID length check

It does not cover live log retrieval, RPC transport, or the on-chain order event surface, which is reviewed in the on-chain order log decoding audit.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Topic identity | The five settlement event topic-0 values equal the canonical keccak-256 of their signatures | Conforms |
| Fail-closed decoding | Unknown topic-0, wrong indexed arity, and a non-56-byte order UID return typed errors, never panic | Conforms |
| Domain mapping | Each event maps into the `#[non_exhaustive]` `SettlementEvent` enum using the crate domain types | Conforms |
| Provider independence | Decoding borrows `LogData` and performs no I/O | Conforms |

## Current Contract

### Topic identity

The `Trade`, `Interaction`, `Settlement`, `OrderInvalidated`, and `PreSignature`
`SIGNATURE_HASH` values are the `alloy::sol!`-generated topic-0 hashes for the
canonical event signatures. Each is byte-locked against an independent keccak-256
of the signature string, so a binding drift breaks the build.

### Fail-closed decoding

`decode_settlement_log` validates the topic count and topic-0 through the shared
`check_topics` guard before ABI decoding and length-checks every decoded
`bytes orderUid` to 56 bytes. No path slices or indexes untrusted log bytes
without a guard, and a fuzz target exercises arbitrary log inputs against the
non-panic contract.

### Domain mapping and provider independence

Each decoded event maps into the `#[non_exhaustive]` `SettlementEvent` enum using
`Address`, `Amount`, and `OrderUid`; the owner on `Trade`, `OrderInvalidated`, and
`PreSignature` is recovered from the indexed topic. The decoder borrows `LogData`
and performs no I/O, so one implementation serves native, browser, and any RPC
client.

## Evidence

Primary implementation points:

- `crates/contracts/src/settlement/events.rs`
- `crates/contracts/src/primitives.rs`

Primary regression coverage:

- `crates/contracts/tests/settlement_events_contract.rs::settlement_event_topic0_byte_locks_match_canonical_keccak`
- `crates/contracts/tests/settlement_events_contract.rs::trade_round_trips`
- `crates/contracts/tests/settlement_events_contract.rs::interaction_round_trips_including_bytes4_selector`
- `crates/contracts/tests/settlement_events_contract.rs::unknown_topic0_is_rejected`
- `crates/contracts/tests/settlement_events_contract.rs::missing_indexed_topic_is_rejected`
- `crates/contracts/tests/settlement_events_contract.rs::wrong_order_uid_length_is_rejected`

Validation surface:

```text
cargo test -p cow-sdk-contracts --test settlement_events_contract
cargo parity-verify-sol-provenance
```
