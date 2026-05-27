# ADR 0029: Trait Evolution Through Extension Traits

- Status: Accepted
- Date: 2026-04-29
- Last reviewed: 2026-04-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: traits, semver, compatibility, providers
- Anchors: Principle 11 (supporting)
- Related: [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md)

## Decision

`Provider` and `SigningProvider` freeze their `0.1.0`
shape. New capabilities ship as `*Ext` traits with blanket
implementations in their owning leaf crates. Default-method bodies on
the core traits are forbidden until Rust stabilizes full-bandwidth
`async fn` in trait objects.

Consumers opt in to new capabilities by importing the extension trait
that owns the capability. Core trait method sets remain stable through
`0.x.y` and are re-evaluated only at major-version boundaries.

## Why

For `dyn Trait` consumers, adding any method changes the vtable shape and
breaks consumers built against the prior trait. That is true even when a
method looks additive at the source level. Extension traits let the SDK
grow capability families without silently changing the object-safe core
traits consumers already hold behind trait objects.

The pattern is also familiar in the Rust ecosystem: `tokio::AsyncReadExt`
and `futures::StreamExt` keep core traits small while making richer
helpers available through explicit imports.

## Must Remain True

- `Provider` and `SigningProvider` method sets remain frozen
  through `0.x.y`.
- Each extension trait name ends in `Ext`.
- Each extension trait is documented as opt-in at the import boundary.
- New capability crates use blanket implementations when the capability
  can be expressed through existing core trait methods.
- Default-method bodies on the core traits remain forbidden until the
  object-safety and async-trait-object tradeoffs are deliberately
  re-reviewed at a major boundary.

## Alternatives Rejected

- Add methods directly to the core traits: source-compatible for some
  generic consumers, but a binary and object-shape break for `dyn Trait`
  users.
- Add default methods to the core traits: shorter in the short term, but
  still changes the trait contract consumers compile against.
- Put all future provider capabilities into one larger trait: easier to
  discover, but forces consumers to implement unrelated capability
  families and weakens the leaf-crate boundary.

## Anchors

This ADR supports Principle 11, Forward-Compatible Public Surfaces.

## Links

- [Principles](../principles.md)
- [Architecture](../architecture.md)
- [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)
- [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md)

**Proven by:**

- `crates/core/tests/trait_evolution_contract.rs`
