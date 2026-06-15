# ADR 0021: Narrow `Order.total_fee` And Read-Only Legacy Executed-Fee Surface

- Status: Accepted
- Date: 2026-04-22
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: orderbook, dto, fees, legacy-fields, semver
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0013](0013-http-transport-injection-and-typestate-builders.md), [ADR 0017](0017-typed-orderbook-rejection-parser.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

The orderbook `Order` DTO in `cow_sdk_orderbook::types::Order`
defines `total_fee` narrowly as the normalised executed-fee
component sourced from the wire `executedFee` field. The DTO also
surfaces the deprecated wire field `executedFeeAmount` as a typed
read-only sibling
`executed_fee_amount: Amount`, deserialized through the standard
camelCase DTO mapping and skipped on serialization when it is zero so
absence on the wire does not re-emit the legacy descriptor.
`calculate_total_fee`
remains pure and continues to define `total_fee` as
`executed_fee` only — the helper does not fold
`executed_fee_amount` into the canonical sum. Consumers
that need the legacy summation compute it explicitly from the two
typed fields.

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
  executed-fee exposure and is defined as the normalised
  `executedFee` value through `calculate_total_fee`. The
  `Order.executed_fee_amount: Amount` field is
  read-only on the public surface — it is populated only by
  deserialization of the legacy `executedFeeAmount` wire field
  and there is no public builder setter for it. The pre-existing
  `Order::new` constructor initialises the field to zero. The
  serializer skips the field when the value is zero so an
  `Order` round-tripped from a payload that did not carry
  `executedFeeAmount` does not re-emit the legacy descriptor.
- Runtime and support: `calculate_total_fee` is pure, treats a
  missing `executed_fee` as zero, validates the value as an
  unsigned decimal string, and never reads
  `executed_fee_amount`. `transform_order` only writes
  `total_fee` from the canonical `executed_fee`; it never
  silently sums the two fee fields. Consumers that want the
  legacy summation perform `executed_fee + executed_fee_amount`
  themselves at the call site.
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
  serde emission rule. No change to `calculate_total_fee` or
  `transform_order`. No change to the public write surface.

## Alternatives Rejected

- Sum both fields silently inside `calculate_total_fee` so
  `total_fee` matches the historical TypeScript behaviour:
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

- [Architecture](../architecture.md)
- [Parity Matrix](../parity.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)
- [ADR 0017](0017-typed-orderbook-rejection-parser.md)
