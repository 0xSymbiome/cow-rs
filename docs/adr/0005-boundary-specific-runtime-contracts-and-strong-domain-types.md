# ADR 0005: Boundary-Specific Runtime Contracts And Strong Domain Types

- Status: Accepted (amended)
- Date: 2026-04-10
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: types, traits, boundaries
- Related: [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md), [ADR 0002](0002-dedicated-trading-orchestration-crate.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

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
  contracts. The HTTP transport seam has crossed into a production trait in
  `cow-sdk-core` (`HttpTransport`) adopted by the native `ReqwestTransport`
  default and the browser `FetchTransport` adapter; GraphQL and pinning
  traits stay adapter seams until concrete crates truly adopt them.
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
- [Transport](../transport.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)

**Proven by:**

- [Credential Surface Contract Hygiene Audit](../audit/credential-surface-contract-hygiene-audit.md)
- [Shared Logic Reviewability Audit](../audit/shared-logic-reviewability-audit.md)
- [Cooperative Cancellation Contract Audit](../audit/cooperative-cancellation-contract-audit.md)

## Amendment 2026-05-22: canonical primitive layer (per ADR 0052)

The strong-domain-type contract above is anchored to the canonical
primitive layer per
[ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md). The
cow-named identity types `Address`, `Hash32`, `AppDataHash`, `HexData`,
and `OrderUid` ship as cow-owned `#[repr(transparent)]` newtypes around
`alloy_primitives::Address`, `alloy_primitives::B256` (twice),
`alloy_primitives::Bytes`, and `alloy_primitives::FixedBytes<56>`
respectively; the type-system distinction between same-width byte
primitives (`Hash32` vs `AppDataHash`, both 32 bytes wide) is preserved
by the newtype layer rather than by naming convention or extension
traits. The wire-form preservation contract is unchanged.
