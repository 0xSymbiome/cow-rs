# ADR 0061: WASM order receiver omission resolves to the pay-to-owner sentinel

- Status: Accepted
- Date: 2026-05-31
- Last reviewed: 2026-05-31
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, abi, order-construction, receiver
- Related: [ADR 0020](0020-ethflow-owner-threading.md), [ADR 0059](0059-hash-concrete-orderdata-directly.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md)

## Decision

At the WASM order boundary, an omitted `receiver` and an explicit
zero-address `receiver` are not distinguished. Both resolve to the zero address,
which CoW Protocol settlement defines as `RECEIVER_SAME_AS_OWNER` — the order
proceeds are paid to the order owner. The boundary performs no
receiver-to-owner reinterpretation: it never rewrites a concrete receiver to the
owner, and it never collapses the owner into the receiver field.

The native `cow_sdk_core::OrderData` crosses the WASM ABI directly, so its
`receiver` field carries this rule. Concretely:

- Input: `OrderData.receiver` is a concrete `Address` that omitting consumers may
  leave out of the JSON payload, because it deserializes through
  `#[serde(default = "default_order_receiver")]` to the zero address. An omitted
  `receiver` and an explicit `"0x0000…0000"` therefore construct an identical
  `OrderData` (and therefore an identical EIP-712 struct hash and order UID).
- Output: an `OrderData` always serializes a concrete `receiver` string, because
  the native order receiver is a concrete `Address` after
  [ADR 0059](0059-hash-concrete-orderdata-directly.md). A pay-to-owner order
  therefore round-trips as the explicit zero address.

## Why

The zero-address pay-to-owner marker is a settlement-contract invariant, not an
SDK convenience: the general order hash path accepts a zero receiver verbatim
and the proceeds are paid to the owner. Distinguishing "field omitted" from
"field present and zero" at the WASM ABI would invent a difference the protocol
does not have, and would push optional/zero-handling complexity onto every JS
consumer for no behavioral gain. Treating both as the pay-to-owner sentinel
keeps the WASM ABI faithful to the on-chain contract and to the native
`OrderData` construction path.

Recording the rule prevents a future contributor from "helpfully" resolving an
omitted receiver to the owner address at the boundary, which would change the
signed struct, or from treating omission as an error.

## Must Remain True

- Public surface: `OrderData.receiver` stays a concrete `Address` with a serde
  default; omission from the wire payload is valid input and means pay-to-owner.
- Runtime and support: an omitted receiver and an explicit zero receiver
  construct byte-identical `OrderData`, so they produce the same order UID and
  the same signature. The eth-flow native-currency path keeps its own narrower
  rule that rejects a zero receiver; that rejection is separate from this
  general-order ABI rule and is unaffected.
- Validation and review: no boundary step may reinterpret receiver and owner for
  each other, consistent with [ADR 0020](0020-ethflow-owner-threading.md). The
  omit and explicit-zero inputs must remain equivalent (same order UID), and
  that equivalence should be pinned by a regression test.
- Cost: the WASM ABI cannot express a future protocol meaning for "receiver
  omitted" that differs from "receiver is the zero address" without a breaking
  change; this is acceptable because the contract defines no such distinction.

## Alternatives Rejected

- Distinguish omission from an explicit zero receiver: rejected. The settlement
  contract treats the zero address as pay-to-owner, so the distinction has no
  protocol meaning and would only add consumer-visible complexity.
- Eager-resolve an omitted receiver to the owner address at the boundary
  (mirroring the upstream TypeScript SDK's `receiver || from` normalization
  before signing): rejected as the default. The zero address already encodes
  pay-to-owner, so rewriting the field would change the signed `OrderData`
  rather than preserve it. This remains an optional, non-breaking parity polish
  if a future consumer needs the resolved receiver echoed back.

## Links

- [Native order receiver and its serde default](../../crates/core/src/types/order.rs)
- [Host-safe order mapping](../../crates/js/src/helpers/dto.rs)
- [ADR 0020](0020-ethflow-owner-threading.md)
- [ADR 0059](0059-hash-concrete-orderdata-directly.md)
