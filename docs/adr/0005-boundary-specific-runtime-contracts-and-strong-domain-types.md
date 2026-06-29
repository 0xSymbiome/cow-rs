---
type: Decision Record
id: ADR-0005
title: "ADR 0005: Boundary-Specific Runtime Contracts And Strong Domain Types"
description: "Keep runtime traits and DTOs boundary-specific, and make strong domain types the default public Rust contract."
status: Accepted
date: 2026-04-10
last_reviewed: 2026-06-15
authors: ["0xSymbiotic"]
tags: [types, traits, boundaries]
related: [ADR-0001, ADR-0002, ADR-0052]
timestamp: 2026-06-15T00:00:00Z
---

# ADR 0005: Boundary-Specific Runtime Contracts And Strong Domain Types

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
  default and the browser `FetchTransport` adapter. GraphQL dispatch adopted
  the same `HttpTransport` seam ([ADR 0013](0013-http-transport-injection-and-typestate-builders.md));
  no separate GraphQL transport trait exists.
- Validation and review: conversions between user-domain, normalized, wire,
  and ABI forms stay explicit, test-backed, and documented. Order-like DTOs do
  not get merged just because they look similar.
- Canonical primitives: the cow identity and numeric types (`Address`, `Hash32`,
  `AppDataHash`, `HexData`, `OrderUid`, `Amount`) are `#[repr(transparent)]`
  newtypes over the alloy-core primitives per
  [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md), preserving the
  type-system distinction between same-width bytes (`Hash32` vs `AppDataHash`).
- Serialized boundary: `Provider::read_contract` returns the ABI-decoded result
  as a serialized JSON `String` — a deliberate serialized boundary (the ABI is
  runtime-supplied and the result crosses the WASM callback boundary where JSON
  is the wire form), not a stringly-typed surface. Strong-typed decoding lives
  one layer up in the allowance reader and the EIP-1271 magic-value decoder, and
  the `Provider` method set stays frozen ([ADR 0057](0057-log-provider-capability-trait.md)).
- Cost: the workspace carries more explicit types, DTOs, and conversions, and
  it rejects some superficially convenient string-based APIs.

## Alternatives Rejected

- Use raw strings as the default public contract: easier to write, but too easy
  to misuse and too weak for long-term semver discipline.
- Collapse domain, wire, and ABI models into shared structs: reduces local
  boilerplate but makes boundaries ambiguous and harder to reason about.

## Links

- [Architecture](../guides/architecture.md)
- [Transport](../guides/transport.md)
- [Verification Guide](../guides/verification.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)

**Proven by:**

- [Credential Redaction Audit](../audit/credential-redaction-audit.md)
