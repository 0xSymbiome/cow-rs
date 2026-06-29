---
type: Decision Record
id: ADR-0002
title: "ADR 0002: Dedicated Trading Orchestration Crate"
description: "Place user-facing quote-to-order workflows in cow-sdk-trading."
status: Accepted
date: 2026-04-09
authors: ["0xSymbiotic"]
tags: [trading, orchestration, package-boundary]
related: [ADR-0001]
timestamp: 2026-04-09T00:00:00Z
---

# ADR 0002: Dedicated Trading Orchestration Crate

## Decision

Place user-facing quote-to-order workflows in `cow-sdk-trading`.

## Why

Quote, sign, submit, cancel, allowance, approval, and slippage handling span
orderbook transport, signing, contracts, and app-data. That workflow needs one
stable home that is not the transport layer and not the root facade.

## Must Remain True

- Public surface: quote-to-order workflows live in `cow-sdk-trading` instead of
  being split across transport crates or hidden in `cow-sdk`.
- Runtime and support: high-level async trading flows can evolve without
  changing transport, hashing, or signing crate boundaries.
- Validation and review: precedence, approval, cancellation, and slippage behavior can be
  tested and documented at one workflow boundary.
- Cost: `cow-sdk-trading` becomes the main integration surface and must stay
  disciplined about scope.

## Alternatives Rejected

- Put orchestration in `cow-sdk-orderbook`: mixes transport concerns with
  workflow policy and precedence.
- Put orchestration in `cow-sdk`: makes the facade own business logic instead
  of exposing owned leaf crates.

## Links

- [Architecture](../guides/architecture.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)

**Proven by:**

- [Trading Order Integrity Audit](../audit/trading-order-integrity-audit.md)
