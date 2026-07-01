---
type: Decision Record
id: ADR-0021
title: "ADR 0021: Narrow Order.total_fee And Read-Only Legacy Executed-Fee Surface"
description: "Define the orderbook Order.total_fee narrowly as the canonical executed-fee component and surface the deprecated executedFeeAmount wire field as a typed read-only sibling, so consumers compute any legacy summation explicitly."
status: Accepted
date: 2026-04-22
last_reviewed: 2026-05-22
authors: ["0xSymbiotic"]
tags: [orderbook, dto, fees, legacy-fields, semver]
related: [ADR-0005, ADR-0013, ADR-0017, ADR-0052]
timestamp: 2026-05-22T00:00:00Z
---

# ADR 0021: Narrow `Order.total_fee` And Read-Only Legacy Executed-Fee Surface

## Decision

_Amended 2026-06-22: `executed_fee` is a typed `Amount` (serde-validated at the DTO
boundary), so the `calculate_total_fee` wire-string normalizer was redundant and has
been removed; `transform_order` now sets `total_fee` from the typed `executed_fee`
directly. The narrow-`total_fee` policy below is unchanged._

The orderbook `Order` DTO in `cow_sdk_orderbook::types::Order`
defines `total_fee` narrowly as the normalised executed-fee
component sourced from the wire `executedFee` field. The DTO also
surfaces the deprecated wire field `executedFeeAmount` as a typed
read-only sibling
`executed_fee_amount: Amount`, deserialized through the standard
camelCase DTO mapping and skipped on serialization when it is zero so
absence on the wire does not re-emit the legacy descriptor.
`transform_order` sets `total_fee` to the typed `executed_fee`
directly (zero when absent) and never folds `executed_fee_amount`
into the canonical sum. Consumers that need the legacy summation
compute it explicitly from the two typed fields.

## Why

A pre-release Rust SDK has the opportunity to ship a narrow,
explicit fee contract instead of inheriting a silent legacy
summation that binds the public type to deprecated wire shape
forever. Keeping `total_fee` equal to the canonical executed-fee
component matches the current backend exposure and gives every
consumer a single, named field to read for the protocol-side
fee. Surfacing `executedFeeAmount` as a typed read-only sibling
preserves the information channel for orders whose backing
records still carry the legacy descriptor — those consumers can
reconstruct the historical sum on demand without the SDK
guessing on their behalf, and any change in the legacy field's
semantics surfaces at the consumer rather than inside the SDK.
The split also keeps the public write surface unchanged: there
is no public builder setter for either fee field, so the
read-only sibling cannot be used as a back-channel to spoof
order-level fees on submission.

## Must Remain True

- Public surface: `Order.total_fee: Amount` is the canonical
  executed-fee exposure and is defined as the typed
  `executed_fee` (zero when absent). The
  `Order.executed_fee_amount: Amount` field is
  read-only on the public surface — it is populated only by
  deserialization of the legacy `executedFeeAmount` wire field
  and there is no public builder setter for it. The pre-existing
  `Order::new` constructor initialises the field to zero. The
  serializer skips the field when the value is zero so an
  `Order` round-tripped from a payload that did not carry
  `executedFeeAmount` does not re-emit the legacy descriptor.
- Runtime and support: `transform_order` is infallible and writes
  `total_fee` from the typed `executed_fee` (zero when absent); it
  never reads `executed_fee_amount` or silently sums the two fee
  fields. Consumers that want the legacy summation perform
  `executed_fee + executed_fee_amount` themselves at the call site.
- Validation and review: regression tests in
  `crates/orderbook/tests/transform_contract.rs` cover the four
  field-population transitions — both fields populated, only the
  canonical executed fee, only the legacy field, and neither
  field — and pin `total_fee` to the canonical executed-fee
  value in every case. The negative test in
  `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs`
  continues to assert that no public builder setter exposes a
  fee-amount write path.
- Cost: one legacy field on the `Order` DTO, one default
  initialiser update inside `Order::new`, and the documented
  serde emission rule. No change to the public write surface.

## Alternatives Rejected

- Sum both fields silently when computing `total_fee` so
  it matches the historical TypeScript behaviour:
  shorter migration for cross-SDK ports, but binds the canonical
  Rust type to the deprecated wire descriptor permanently and
  hides the legacy contribution from consumers that want to
  diagnose fee composition.
- Drop `executedFeeAmount` from the DTO entirely: simplest type,
  but loses information for orders whose backing records still
  carry the legacy field and forces consumers that need the
  legacy summation to bypass the typed surface.
- Expose `executed_fee_amount` as a public builder field:
  matches the read/write symmetry of other DTO fields, but
  re-opens a fee-amount write path that the orderbook-services
  contract rejects today and that the negative builder test
  exists to prevent.
- Compute the legacy sum on read through a non-stored derived
  method: hides the wire provenance, complicates the DTO surface
  with a derived accessor that diverges from the field-only
  shape every other DTO follows, and still fails to give
  consumers access to the raw legacy value.

## Links

- [Architecture](../guides/architecture.md)
- [Parity Matrix](../guides/parity.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)
- [ADR 0017](0017-typed-orderbook-rejection-parser.md)
