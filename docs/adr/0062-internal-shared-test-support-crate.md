# ADR 0062: Shared Test Support Lives In One Unpublished `cow-sdk-test-utils` Crate

- Status: Accepted
- Date: 2026-06-02
- Last reviewed: 2026-06-02
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: testing, crate-boundary, dev-dependencies
- Related: [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md), [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0063](0063-published-consumer-test-doubles-crate.md)

## Decision

Shared, cross-crate test support lives in a single `cow-sdk-test-utils` crate
marked `publish = false`. It owns the canonical test constants, an independent
EIP-712 keccak/ABI-word oracle, parity-fixture loaders, order/domain/signature
builders, and recording signer doubles. Every workspace crate consumes it only
through `[dev-dependencies]`; it never enters the normal or build dependency
graph of any published crate.

## Why

Several crates need the same constants, fixtures, builders, and oracle.
Duplicating them per crate drifts — a near-verbatim copy had already diverged on
address casing. One owned crate gives the workspace a single reviewed source of
test truth, and `publish = false` guarantees it cannot resolve into a consumer
build regardless of feature unification.

## Must Remain True

- Public surface: the crate stays `publish = false` and is never a normal or
  build dependency of any `publish = true` crate; it is reachable only from
  `[dev-dependencies]` and adds nothing to the published SDK surface.
- Runtime and support: it depends only on the workspace crates it supports plus
  test libraries, and its default dependency path stays target-agnostic so a
  core-only build still compiles for `wasm32`. Property-test generators that
  require a property-testing dependency stay on their owning crate behind an
  opt-in feature rather than moving here, keeping this crate dependency-light.
- Validation and review: it holds test scaffolding only — constants, oracle,
  fixtures, builders, recording doubles — never production behavior, and
  production code never imports it. As dev-only test code it sits outside the
  published panic-free surface of ADR 0033, so canned setup may use test-grade
  `expect`.
- Cost: one more workspace crate to keep coherent, in exchange for one reviewed
  home for shared test support.

## Alternatives Rejected

- Duplicate helpers per crate: simplest at first, but it drifts (already
  observed) and multiplies maintenance.
- Publish the shared helpers: would widen the public surface with code only the
  workspace tests use, and conflate internal plumbing with the consumer-facing
  doubles of ADR 0063.
- Share `#[cfg(test)]` modules by path include: fragile, not reusable across
  crate boundaries, and invisible to `cargo` resolution.

## Links

- [Architecture](../architecture.md)
- [Principles](../principles.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
- [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)
- [ADR 0063](0063-published-consumer-test-doubles-crate.md)
