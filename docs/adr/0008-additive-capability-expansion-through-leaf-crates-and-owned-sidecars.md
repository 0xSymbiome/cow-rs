# ADR 0008: Additive Capability Expansion Through Leaf Crates And Owned Sidecars

- Status: Accepted
- Date: 2026-04-13
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: extensibility, packages, sidecars, future-growth
- Related: [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md), [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)

## Decision

Grow new capability surfaces through additive leaf crates or separately owned
sidecars before widening `core`, `trading`, or the default `cow-sdk` facade.

## Why

Future expansion is a common point of architectural erosion. If new
ecosystems, generated artifacts, provider-specific logic, or tooling helpers
move directly into shared runtime crates, the default SDK surface becomes
broader, harder to review, and harder to evolve cleanly.

## Must Remain True

- Public surface: the default facade stays narrower than the full workspace.
  New ecosystems and optional capabilities are opt-in and should not become the
  only meaningful public contract through convenience layering.
- Runtime and support: provider-specific implementations, generated artifacts,
  tooling helpers, and evidence-refresh machinery stay separately owned when
  their dependency or runtime assumptions differ materially from the core SDK
  crates.
- Validation and review: opening a new capability wave requires explicit crate
  ownership, public-surface documentation, and proof lanes rather than silent
  widening of existing crates.
- Cost: the workspace may gain more crates and more deliberate design work
  before shipping new optional capabilities.

## Alternatives Rejected

- Widen shared crates as new features appear: simpler at first, but it mixes
  unrelated concerns and makes later boundaries harder to recover.
- Let helper layers, generated mirrors, or tooling sidecars become the de facto
  public contract: convenient in the short term, but weak for semver and review
  discipline.

## Links

- [Architecture](../architecture.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
- [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
