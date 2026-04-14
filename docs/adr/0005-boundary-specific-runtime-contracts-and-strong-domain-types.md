# ADR 0005: Boundary-Specific Runtime Contracts And Strong Domain Types

- Status: Accepted
- Date: 2026-04-10
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: types, traits, boundaries
- Related: [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md), [ADR 0002](0002-dedicated-trading-orchestration-crate.md)

## Decision

Keep runtime traits and DTOs boundary-specific, and make strong domain types
the default public Rust contract.

## Why

The SDK spans user-domain models, normalized forms, wire DTOs, ABI-facing
structures, and runtime integration contracts. Flattening those boundaries or
defaulting to string-heavy public types would make misuse easier, obscure
semver-significant behavior, and encourage abstractions that do not match
actual runtime seams.

## Must Remain True

- Public surface: addresses, hashes, token amounts, identifiers, and similar
  protocol values use strong domain types by default. String-heavy forms remain
  limited to explicit wire, serialized, or compatibility boundaries.
- Runtime and support: active signer and provider traits remain real runtime
  contracts. Generic HTTP, GraphQL, or pinning traits stay adapter seams until
  concrete crates truly adopt them.
- Validation and review: conversions between user-domain, normalized, wire,
  and ABI forms stay explicit, test-backed, and documented. Order-like DTOs do
  not get merged just because they look similar.
- Cost: the workspace carries more explicit types, DTOs, and conversions, and
  it rejects some superficially convenient string-based APIs.

## Alternatives Rejected

- Use raw strings as the default public contract: easier to write, but too easy
  to misuse and too weak for long-term semver discipline.
- Collapse domain, wire, and ABI models into shared structs: reduces local
  boilerplate but makes boundaries ambiguous and harder to reason about.

## Links

- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
