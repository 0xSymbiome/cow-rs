# ADR 0031: Wire DTOs Follow OpenAPI; The Order/AuctionOrder Split Collapsed To One Order Type

- Status: Accepted
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
`parity/openapi/`. The order-shaped response surface is the single `Order` type: the
auction-only fields (`protocolFees`, `preInteractions`, `postInteractions`,
`created`, `executed`, and an auction-side `quote: Quote`) do not appear on
`Order`, have no public producer, and ship no `AuctionOrder` mirror —
`/api/v1/auction` is an upstream liveness probe, not a public data feed.
Solver-competition reads target the orderbook `v2` routes and decode into a
fully typed `SolverCompetitionResponse` (typed addresses, amounts, order UIDs,
transaction hashes, per-solver reference scores, and each solution's touched
orders), with a shared `AuctionPrices` (`BTreeMap<Address, Amount>`) typing the
clearing- and reference-price maps. `SolverCompetitionResponse` is covered by a
producer-pinned round-trip fixture rather than the `openapi-coverage` manifest
(the vendored v2 schema omits a `required:` block, which would force an
all-`Option` shape against the producer's actual required fields).

`cargo parity-openapi-coverage` expands each schema's inventory in
memory from the vendored OpenAPI and validates Rust DTO coverage against
required, nullable, and default semantics. Hand-written DTO snippets are not
authoritative; the vendored spec is.

## Why

The orderbook backend can add optional response fields faster than the
Rust SDK releases. Open response DTOs keep existing SDK builds
deserializing those additions, while inventory-backed typed fields make
the modeled surface auditable. Splitting `Order` and `AuctionOrder`
prevents auction-only fields from leaking onto ordinary order records and
keeps each Rust type faithful to its upstream schema.

## Must Remain True

- Every public response DTO listed in `parity/openapi/coverage.yaml` has
  a Rust mirror that passes `openapi-coverage`.
- `OrderParameters` (the quote response payload) is covered by the
  `cow_sdk_orderbook::QuoteData` mirror, so the `OrderQuoteResponse` `quote`
  field is validated for field-level fidelity rather than as an opaque object
  (see [ADR 0058](0058-typed-quote-request-response-surface.md)).
- `Order` is exercised by a recorded fixture that passes
  `openapi-coverage`.
- `Order` does not carry auction-only fields (`protocolFees`,
  `preInteractions`, `postInteractions`, `created`, `executed`, or an
  auction-side `quote`); those fields belong to the unmirrored auction schema,
  which has no public producer.
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
