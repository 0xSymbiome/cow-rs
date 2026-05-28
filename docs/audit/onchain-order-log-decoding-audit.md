# On-Chain Order Log Decoding Audit

Status: Current
Last reviewed: 2026-05-28
Owning surface: `cow-sdk-contracts` `CoWSwapOnchainOrders` event decoder
Refresh trigger: a change to the `OrderPlacement` / `OrderInvalidation` event ABI, the on-chain signing-scheme set, the eth-flow trailing-data layout, the `GPv2` order markers, or the `compute_order_uid` hashing path
Related docs:
- [ADR 0054](../adr/0054-onchain-order-event-decoding-is-fail-closed.md)
- [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md)
- [Parity Matrix](../parity-matrix.md)

## Scope

This audit covers:

- the `alloy::sol!` `ICoWSwapOnchainOrders` event bindings (`OrderPlacement`, `OrderInvalidation`) and their topic-0 signature hashes
- `decode_order_placement` / `decode_order_invalidation` and the `OnchainOrderPlacement` owner-resolution and UID-derivation surface
- the eth-flow `OrderPlacement` trailing-data parser and the `wrapAll()` pre-interaction selector
- the `GPv2` order-marker reverse mapping (`order_kind_from_marker`, `sell_balance_from_marker`, `buy_balance_from_marker`)

It does not cover live log retrieval, RPC transport, or settlement-event decoding beyond the on-chain order surface.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Topic identity | `OrderPlacement` / `OrderInvalidation` topic-0 equal the canonical keccak of the flattened-tuple signatures | Conforms |
| Fail-closed decoding | Malformed topics, signing scheme, EIP-1271 payload length, and UID length return typed errors, never panic | Conforms |
| Owner resolution | PreSign owner is the event sender; EIP-1271 owner is the 20-byte signature payload, length-checked | Conforms |
| UID derivation | A decoded order UID reuses `compute_order_uid` and reproduces the upstream order-hash vector | Conforms |
| Provider independence | Decoding borrows `LogData` and performs no I/O | Conforms |

## Current Contract

### Topic identity

`OrderPlacement::SIGNATURE_HASH` and `OrderInvalidation::SIGNATURE_HASH` are the `alloy::sol!`-generated topic-0 values for the flattened `GPv2` order-tuple signatures. Both are byte-locked against an independent keccak-256 of the canonical signature strings, so a binding drift breaks the build.

### Fail-closed decoding

`decode_order_placement` validates the topic count and topic-0 before ABI decoding, range-checks the on-chain signing scheme, and rejects unrecognized order markers; `decode_order_invalidation` length-checks the 56-byte UID. Owner resolution length-checks the EIP-1271 payload. No path slices or indexes untrusted log bytes without a guard, and a fuzz target exercises arbitrary log inputs against the non-panic contract.

### Owner resolution and UID derivation

For a pre-signature placement the owner is the event sender; for an EIP-1271 placement the owner is the 20-byte address carried in the signature payload. A decoded order's UID is computed through `compute_order_uid` against the canonical settlement domain, so it equals the UID the settlement contract derives. The hashing path is locked against the upstream `dummyOrder()` order-hash vector.

## Evidence

Primary implementation points:

- `crates/contracts/src/onchain_orders.rs`
- `crates/contracts/src/eth_flow.rs`
- `crates/contracts/src/primitives.rs`

Primary regression coverage:

- `crates/contracts/tests/onchain_orders.rs::order_placement_topic0_matches_canonical_hash`
- `crates/contracts/tests/onchain_orders.rs::order_hash_matches_canonical_ethflow_foundry_vector`
- `crates/contracts/tests/onchain_orders.rs::eip1271_placement_decodes_owner_uid_and_trailer`
- `crates/contracts/tests/onchain_orders.rs::eip1271_owner_requires_twenty_byte_signature_payload`
- `crates/contracts/tests/onchain_orders.rs::order_invalidation_rejects_wrong_uid_length`
- `crates/contracts/src/primitives.rs::tests::order_kind_marker_round_trips_and_rejects_unknown`

Validation surface:

```text
cargo test -p cow-sdk-contracts --test onchain_orders
cargo parity-verify-sol-provenance
```
