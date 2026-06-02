# ADR 0031: Wire DTOs Follow OpenAPI With Separate Order And AuctionOrder Types

- Status: Accepted (amended)
- Date: 2026-04-29
- Last reviewed: 2026-05-30
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: orderbook, dto, openapi, compatibility
- Anchors: Forward-Compatible Public Surfaces (primary)
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
- `Order` is exercised by a recorded fixture that passes
  `openapi-coverage --validate`.
- `Order` does not carry auction-only fields (`protocolFees`,
  `preInteractions`, `postInteractions`, `created`, `executed`, or an
  auction-side `quote`); those fields belong to the auction schema, which has
  no public producer and is not mirrored (see the amendment below).
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

This ADR is the primary anchor for the
Forward-Compatible Public Surfaces principle.

## Amendment (2026-05-29)

Solver-competition reads target the orderbook `v2` routes and decode into a
fully typed `SolverCompetitionResponse` — typed addresses, amounts, order UIDs,
and transaction hashes; per-solver reference scores; and each solution's touched
orders (`SolverCompetitionOrder`). A shared `AuctionPrices`
(`BTreeMap<Address, Amount>`) types the clearing- and reference-price maps. This
is consistent with this ADR's OpenAPI-driven, typed-response posture and with
[ADR 0058](0058-typed-quote-request-response-surface.md).

`OrderbookApi::get_auction` and the `Auction` response wrapper are not exposed:
`/api/v1/auction` is not reachable for public clients and is treated upstream as
a liveness probe rather than a consumer data feed. Because no public endpoint
produces an auction snapshot, the `AuctionOrder` mirror and its auction-side
`quote: Quote` had no reachable producer and are removed; the order-shaped
response surface collapses to the single `Order` type, and the OpenAPI-driven
coverage discipline above now governs `Order` and the remaining response DTOs
(`OrderQuoteResponse`, `OrderParameters`/`QuoteData`, `Trade`,
`StoredOrderQuote`, `OnchainOrderData`, `TotalSurplus`, and `SolverExecution`).
An auction-retrieval method, the `AuctionOrder` mirror, and its quote can return
additively if `/api/v1/auction` becomes publicly consumable.

`SolverCompetitionResponse` is intentionally not enrolled in the
`openapi-coverage --validate` manifest. The vendored v2 schema omits a
`required:` block, so the optionality check would demand an all-`Option` shape;
the upstream producer (the `Response` struct in `solver_competition_v2.rs`)
instead treats the identity and collection fields as required and only
`txHash` / `referenceScore` as optional, and the SDK mirrors that producer
contract. The type is covered by a producer-pinned round-trip fixture
(`parity/fixtures/orderbook/solver_competition_response.json`) rather than the
OpenAPI-optionality manifest, which would degrade the typed boundary against the
verified producer.

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
