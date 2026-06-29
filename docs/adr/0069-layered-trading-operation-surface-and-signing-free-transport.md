---
type: Decision Record
id: ADR-0069
title: "ADR 0069: Layered Trading Operation Surface With Signing-Free Transport Crates"
description: "High-level trading operations are offered at complementary layers, and the fluent order-lifecycle builder lives in the orchestration crate rather than in a transport crate."
status: Accepted
date: 2026-06-14
authors: ["0xSymbiotic"]
tags: [trading, orderbook, package-boundary, public-surface]
related: [ADR-0002, ADR-0013, ADR-0001]
timestamp: 2026-06-14T00:00:00Z
---

# ADR 0069: Layered Trading Operation Surface With Signing-Free Transport Crates

## Decision

High-level trading operations are offered at complementary layers, and the fluent
order-lifecycle builder lives in the orchestration crate rather than in a transport
crate.

- `cow-sdk-trading` exposes each operation as a stateless free function. The bound
  `Trading` client exposes a curated subset of those operations as methods that resolve
  stored chain, app-code, and orderbook context and then delegate to the free function.
  No method re-implements operation logic.
- The fluent builders (`Trading::swap()` and `Trading::limit()`) are the guided entries
  for the order-placement operations. Each assembles its order — `TradeParams` for a
  market swap, `LimitTradeParams` for a limit order — through named, non-transposable
  token and amount setters and delegates to the bound-client methods. Operations whose
  constructor carries no same-typed transposable pair (cancellation, pre-sign, allowance,
  approval) have no fluent builder.
- `cow-sdk-orderbook` and `cow-sdk-subgraph` stay typed transport clients: a typestate
  construction builder, one method per endpoint, an injection trait seam
  (`OrderbookClient`), and request DTOs. They host no order-lifecycle builder and depend
  on no signing crate, so a consumer can use the typed transport without compiling the
  signing stack.
- Each operation is reachable by one public import path.

## Why

The dedicated orchestration crate (ADR 0002) must stay disciplined about scope. Placing an
order signs, generates app-data, and resolves eth-flow contracts, so a swap or limit
builder transitively needs the signing, app-data, and contracts crates. Hosting such a
builder on the orderbook client would force the transport crate to depend on signing and
lose its value as a lightweight, signing-free client usable on its own — for a backend that
signs elsewhere, a read-only tool, or a size-sensitive browser bundle. Keeping the lifecycle
in `cow-sdk-trading` keeps the transport crates clean and gives consumers a clear rule for
which entry point to reach for.

The layered entries are not redundancy. The free functions serve composition and
integrations that hold no bound client; the bound-client methods serve the
construct-once-call-many path; the fluent swap and limit builders make token and amount
transposition a compile-time impossibility for the order-placement operations. Each higher
layer is a thin delegation to the one below, so there is one implementation per operation,
reached through whichever entry fits the caller.

## Must Remain True

- Public surface: `cow-sdk-trading` keeps the free-function, bound-method, and
  fluent-builder entries; the method layer stays a curated subset and never a full mirror;
  fluent builders cover the order-placement operations (swap and limit) and are not added
  for operations without a same-typed transposable pair; each operation has one public
  import path.
- Runtime and support: `cow-sdk-orderbook` and `cow-sdk-subgraph` depend on no signing
  crate and host no order-lifecycle builder; order-lifecycle orchestration stays in
  `cow-sdk-trading`.
- Validation and review: bound-client methods delegate to the free functions without
  duplicating logic, and the swap and limit builders delegate to the bound-client methods.
- Cost: the trading crate carries three complementary entry layers, which is more surface
  to document than a single path; the discipline rules above keep that surface legible
  rather than redundant.

## Alternatives Rejected

- Bound-client path only (delete the free functions): loses composition and integration
  callers that hold no client, and diverges from the established standalone-function
  surface the upstream SDK ships.
- Fluent-only path: orphans every non-swap operation or forces a separate fluent builder
  per operation, and binds the transport crate to the signing stack.
- Swap builder on the orderbook client: makes `cow-sdk-orderbook` depend on the signing
  crate, collapsing the transport/orchestration boundary ADR 0002 establishes and
  removing the signing-free transport tier.
- A dedicated swap crate: it would need every dependency `cow-sdk-trading` already has,
  so it is the orchestration crate under another name with an extra crate seam to
  maintain.

## Links

- [Architecture](../guides/architecture.md)
- [ADR 0002](0002-dedicated-trading-orchestration-crate.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
