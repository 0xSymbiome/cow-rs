# Event Log Decoding Audit

Status: Current
Last reviewed: 2026-06-20
Owning surface: `cow-sdk-contracts` event-log decoders for `CoWSwapOnchainOrders` and `GPv2Settlement`
Refresh trigger: a change to the on-chain order events (`OrderPlacement` / `OrderInvalidation` / `OrderRefund`), the settlement events (`Trade` / `Interaction` / `Settlement` / `OrderInvalidated` / `PreSignature`), their `alloy::sol!` bindings or topic-0 hashes, the `decode_eth_flow_log` / `decode_settlement_log` dispatch sets, the `EthFlowEvent` / `SettlementEvent` domain enums, the on-chain signing-scheme set, the eth-flow trailing-data layout, the `GPv2` order markers, the shared `check_topics` guard, or the `compute_order_uid` hashing path
Related docs:
- [ADR 0054](../adr/0054-onchain-order-event-decoding-is-fail-closed.md)
- [Contract Bindings Parity Audit](contract-bindings-parity-audit.md)

## Scope

This audit covers:

- the on-chain order decoders: the `alloy::sol!` `ICoWSwapOnchainOrders` event bindings (`OrderPlacement`, `OrderInvalidation`), `decode_order_placement` / `decode_order_invalidation`, the `OnchainOrderPlacement` owner-resolution and UID-derivation surface, the eth-flow `OrderPlacement` trailing-data parser (`parse_eth_flow_onchain_data`), the `GPv2` order-marker reverse mapping, and the `ICoWSwapEthFlowEvents` `OrderRefund` binding with the unified `decode_eth_flow_log` / `EthFlowEvent` dispatcher
- the settlement decoder: the `alloy::sol!` `IGPv2SettlementEvents` event bindings (`Trade`, `Interaction`, `Settlement`, `OrderInvalidated`, `PreSignature`), `decode_settlement_log`, the `SettlementEvent` domain mapping, and the shared fail-closed `check_topics` topic-set guard with the 56-byte order-UID length check

It does not cover live log retrieval or RPC transport.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Topic identity | On-chain order and settlement event topic-0 values equal the canonical keccak-256 of their flattened-tuple signatures and are byte-locked, so a binding drift fails the byte-lock tests in CI | Conforms |
| Fail-closed decoding | Malformed topics, unknown topic-0, wrong indexed arity, signing scheme, EIP-1271 payload length, and non-56-byte UID return typed errors, never panic | Conforms |
| Owner resolution | PreSign owner is the event sender; EIP-1271 owner is the 20-byte signature payload, length-checked; settlement owners are recovered from the indexed topic | Conforms |
| UID derivation | A decoded order UID reuses `compute_order_uid` and reproduces the upstream order-hash vector | Conforms |
| Domain mapping | Each event maps into the `#[non_exhaustive]` `EthFlowEvent` / `SettlementEvent` enum using the crate domain types | Conforms |
| Provider independence | Decoding borrows `LogData` and performs no I/O | Conforms |
| Refund + dispatch | `OrderRefund` topic-0 is byte-locked and its 56-byte UID is length-checked; `decode_eth_flow_log` routes each topic-0 to the matching fail-closed decoder | Conforms |

## Current Contract

### Topic identity

The `alloy::sol!`-generated `SIGNATURE_HASH` topic-0 values for the on-chain order events (`OrderPlacement`, `OrderInvalidation`, `OrderRefund`) and the five settlement events (`Trade`, `Interaction`, `Settlement`, `OrderInvalidated`, `PreSignature`) are each byte-locked against an independent keccak-256 of the canonical signature string, so a binding drift fails the byte-lock tests in CI.

### Fail-closed decoding

`decode_order_placement` validates the topic count and topic-0 before ABI decoding, range-checks the on-chain signing scheme, and rejects unrecognized order markers; `decode_order_invalidation` length-checks the 56-byte UID. `decode_settlement_log` validates the topic count and topic-0 through the shared `check_topics` guard before ABI decoding and length-checks every decoded `bytes orderUid` to 56 bytes. Owner resolution length-checks the EIP-1271 payload. No path slices or indexes untrusted log bytes without a guard, and fuzz targets exercise arbitrary log inputs against the non-panic contract.

### Owner resolution and UID derivation

For a pre-signature placement the owner is the event sender; for an EIP-1271 placement the owner is the 20-byte address carried in the signature payload. A decoded order's UID is computed through `compute_order_uid` against the canonical settlement domain, so it equals the UID the settlement contract derives. The hashing path is locked against the upstream `dummyOrder()` order-hash vector.

### Settlement domain mapping

Each decoded settlement event maps into the `#[non_exhaustive]` `SettlementEvent` enum using `Address`, `Amount`, and `OrderUid`; the owner on `Trade`, `OrderInvalidated`, and `PreSignature` is recovered from the indexed topic.

### Refund and unified dispatch

`decode_order_refund` validates the topic set (topic-0 and the single indexed `refunder`) and length-checks the 56-byte order UID; its topic-0 is byte-locked against an independent keccak of the canonical signature. `decode_eth_flow_log` dispatches on topic-0 across `OrderPlacement`, `OrderInvalidation`, and `OrderRefund` and returns the `#[non_exhaustive]` `EthFlowEvent`, delegating to the matching fail-closed decoder and performing no I/O.

## Evidence

Primary implementation points:

- `crates/contracts/src/onchain_orders.rs`
- `crates/contracts/src/eth_flow.rs`
- `crates/contracts/src/settlement.rs`
- `crates/contracts/src/primitives.rs`

Primary regression coverage:

- `crates/contracts/tests/onchain_orders.rs::order_placement_topic0_matches_canonical_hash`
- `crates/contracts/tests/onchain_orders.rs::order_hash_matches_canonical_ethflow_foundry_vector`
- `crates/contracts/tests/onchain_orders.rs::eip1271_placement_decodes_owner_uid_and_trailer`
- `crates/contracts/tests/onchain_orders.rs::eip1271_owner_requires_twenty_byte_signature_payload`
- `crates/contracts/tests/onchain_orders.rs::order_invalidation_rejects_wrong_uid_length`
- `crates/contracts/src/primitives.rs::tests::order_kind_marker_round_trips_and_rejects_unknown`
- `crates/contracts/tests/eth_flow_events_contract.rs::order_refund_topic0_matches_canonical_hash`
- `crates/contracts/tests/eth_flow_events_contract.rs::decode_eth_flow_log_dispatches_all_three_events`
- `crates/contracts/tests/eth_flow_events_contract.rs::order_refund_wrong_uid_length_is_rejected`
- `crates/contracts/tests/settlement_events_contract.rs::settlement_event_topic0_byte_locks_match_canonical_keccak`
- `crates/contracts/tests/settlement_events_contract.rs::trade_round_trips`
- `crates/contracts/tests/settlement_events_contract.rs::interaction_round_trips_including_bytes4_selector`
- `crates/contracts/tests/settlement_events_contract.rs::unknown_topic0_is_rejected`
- `crates/contracts/tests/settlement_events_contract.rs::missing_indexed_topic_is_rejected`
- `crates/contracts/tests/settlement_events_contract.rs::wrong_order_uid_length_is_rejected`

Validation surface:

```text
cargo test -p cow-sdk-contracts --test onchain_orders
cargo test -p cow-sdk-contracts --test eth_flow_events_contract
cargo test -p cow-sdk-contracts --test settlement_events_contract
```
