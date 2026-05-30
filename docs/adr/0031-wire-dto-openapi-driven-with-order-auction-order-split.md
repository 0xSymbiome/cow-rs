# ADR 0031: Wire DTOs Follow OpenAPI With Separate Order And AuctionOrder Types

- Status: Accepted
- Date: 2026-04-29
- Last reviewed: 2026-04-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: orderbook, dto, openapi, compatibility
- Anchors: Principle 11 (primary)
- Related: [ADR 0017](0017-typed-orderbook-rejection-parser.md), [ADR 0027](0027-post-quantum-signing-absorption-plan.md), [ADR 0058](0058-typed-quote-request-response-surface.md)

## Decision

Public wire DTOs in response position never use
`#[serde(deny_unknown_fields)]`. Public wire DTOs in request position may
use `deny_unknown_fields` only when the SDK owns the schema.

The source of truth for orderbook response DTOs is the
source-lock-pinned `services/crates/orderbook/openapi.yml`, vendored into
`parity/openapi/`. `Order` and `AuctionOrder` are separate Rust types
covering separate OpenAPI schemas. `protocolFees`, `preInteractions`,
`postInteractions`, `created`, `executed`, and the auction-side
`quote: Quote` live on `AuctionOrder`; they do not appear on `Order`.

`parity-maintainer openapi-coverage --validate` parses the OpenAPI
inventory and validates Rust DTO coverage against required, nullable, and
default semantics. Hand-written DTO snippets are not authoritative; the
inventory is.

## Why

The orderbook backend can add optional response fields faster than the
Rust SDK releases. Open response DTOs keep existing SDK builds
deserializing those additions, while inventory-backed typed fields make
the modeled surface auditable. Splitting `Order` and `AuctionOrder`
prevents auction-only fields from leaking onto ordinary order records and
keeps each Rust type faithful to its upstream schema.

## Must Remain True

- Every public response DTO listed in `parity/openapi/coverage.yaml` has
  a Rust mirror that passes `openapi-coverage --validate`.
- `OrderParameters` (the quote response payload) is covered by the
  `cow_sdk_orderbook::QuoteData` mirror, so the `OrderQuoteResponse` `quote`
  field is validated for field-level fidelity rather than as an opaque object
  (see [ADR 0058](0058-typed-quote-request-response-surface.md)).
- `Order` and `AuctionOrder` are exercised by separate fixtures.
- `Order` does not carry auction-only fields.
- `AuctionOrder` carries auction-only protocol-fee, interaction, created,
  executed, and auction-side quote fields.
- Adding a new upstream response field is a `0.x.0` minor update on the
  Rust SDK, never a `0.x.y` patch.
- Response DTOs remain open to unknown fields under serde defaults.

## Alternatives Rejected

- Use hand-written DTO snippets as the authority: readable in prose, but
  too easy to drift from OpenAPI.
- Merge `Order` and `AuctionOrder` into one Rust shape: convenient for
  callers that want one struct, but incorrect for schema fidelity.
- Add `deny_unknown_fields` to response DTOs: catches typos in fixtures,
  but breaks consumers when upstream services add fields.

## Anchors

This ADR is the primary anchor for Principle 11,
Forward-Compatible Public Surfaces.

## Links

- [Principles](../principles.md)
- [Parity Matrix](../parity-matrix.md)
- [Wire DTO Coverage Audit](../audit/wire-dto-coverage-audit.md)
- `parity/openapi/coverage.yaml`

**Proven by:**

- [Wire DTO Coverage Audit](../audit/wire-dto-coverage-audit.md)
- [Quote Response Surface Audit](../audit/quote-response-surface-audit.md)
- `scripts/parity-maintainer/src/openapi_coverage.rs`
- `crates/orderbook/tests/transform_contract.rs`
- `crates/orderbook/tests/openapi_dto_coverage.rs`
