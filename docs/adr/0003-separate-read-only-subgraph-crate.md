# ADR 0003: Separate Read-Only Subgraph Crate

- Status: Accepted
- Date: 2026-04-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Last reviewed: 2026-06-07
- Tags: subgraph, analytics, read-only, facade, feature
- Related: [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md), [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0060](0060-uniform-error-classification.md)

## Decision

Own subgraph access in a separate, read-only `cow-sdk-subgraph` crate, and also
re-export it through the `cow-sdk` facade behind an off-by-default `subgraph`
feature (`cow_sdk::subgraph`), alongside the other optional capability features
such as `cow-shed`.

## Why

Subgraph access serves analytics and reporting use cases over a GraphQL
endpoint. That is operationally distinct from order submission, orderbook REST
transport, and wallet-backed trading flows, so the behavior lives in its own
crate rather than inside a trading-path crate.

Subgraph access is still a first-class SDK capability, so the facade makes it
reachable as an explicit opt-in. The feature is off by default, so the default
`cow-sdk` surface and dependency closure stay trading-first; enabling it lifts
both the `cow_sdk::subgraph` module and the feature-gated `CowError::Subgraph`
classification variant. This is the same additive-feature pattern the facade
already uses for its other optional leaves, so subgraph is consistent with them
rather than a special case.

## Must Remain True

- Public surface: the `subgraph` feature is off by default (`default = []`); a
  consumer that does not enable it pays no subgraph dependency and sees no
  subgraph surface.
- Standalone use: `cow-sdk-subgraph` stays usable directly; the facade
  re-export is additive convenience, not the only path.
- Runtime and support: GraphQL analytics concerns stay separate from trading
  and orderbook transport.
- Error family: when the feature is enabled, `SubgraphError` joins the uniform
  classification family through `CowError::Subgraph` and
  `SubgraphError::class()` ([ADR 0060](0060-uniform-error-classification.md)).
- Validation and review: query documents, typed results, and read-only
  guarantees can be proven without coupling analytics behavior to order
  submission flows.

## Alternatives Rejected

- Re-export subgraph helpers from `cow-sdk` by default (always on): widens the
  trading-first default surface and dependency closure for every consumer,
  including those that never query the subgraph.
- Keep subgraph permanently unreachable from the facade: makes subgraph the only
  optional leaf the facade refuses to surface even as an opt-in, which is
  inconsistent with the other capability features and adds enforcement machinery
  with no corresponding benefit once the feature is off by default.
- Fold GraphQL access into `cow-sdk-orderbook`: mixes read-only analytics with
  orderbook REST transport responsibilities.
- Fold subgraph into `cow-sdk-core`: places network I/O in a pure-transform
  crate, against the explicit runtime-boundary principle.

## Links

- [Architecture](../architecture.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
