# ADR 0003: Separate Read-Only Subgraph Crate

- Status: Accepted
- Date: 2026-04-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: subgraph, analytics, read-only
- Related: [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)

## Decision

Expose subgraph functionality through a separate, read-only
`cow-sdk-subgraph` crate.

## Why

Subgraph access serves analytics and reporting use cases. It is operationally
different from order submission, orderbook transport, and wallet-backed
trading flows, and it should not become a hidden dependency of the default SDK
surface.

## Must Remain True

- Public surface: subgraph access stays explicit and opt-in through
  `cow-sdk-subgraph`.
- Runtime and support: GraphQL analytics concerns stay separate from trading,
  orderbook transport, and browser-wallet runtime behavior.
- Validation and review: query documents, typed results, and read-only guarantees can be
  proven without coupling analytics behavior to order submission flows.
- Cost: analytics consumers add one more crate.

## Alternatives Rejected

- Re-export subgraph helpers from `cow-sdk` by default: widens the root facade
  beyond the trading-first surface.
- Fold GraphQL access into `cow-sdk-orderbook`: mixes read-only analytics with
  orderbook transport responsibilities.

## Links

- [Architecture](../architecture.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
