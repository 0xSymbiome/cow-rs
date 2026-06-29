---
type: Decision Record
id: ADR-0013
title: "ADR 0013: HTTP Transport Injection Seam And Typestate Construction For Orderbook And Subgraph"
description: "HTTP dispatch for orderbook and subgraph flows through the single HttpTransport trait in cow-sdk-core."
status: Accepted
date: 2026-04-21
last_reviewed: 2026-06-15
authors: ["0xSymbiotic"]
tags: [transport, typestate, builders, wasm, async]
related: [ADR-0005, ADR-0006, ADR-0010, ADR-0011, ADR-0039]
timestamp: 2026-06-15T00:00:00Z
---

# ADR 0013: HTTP Transport Injection Seam And Typestate Construction For Orderbook And Subgraph

## Decision

HTTP dispatch for orderbook and subgraph flows through the single
`HttpTransport` trait in `cow-sdk-core`. The trait stays dyn-compatible through
`async-trait`, so consumers can hold any implementation behind
`Arc<dyn HttpTransport + Send + Sync>`.

The typestate builders accept the same transport seam for every runtime:
native consumers pass `Arc<ReqwestTransport>`, browser wasm consumers pass
`Arc<FetchTransport>`, and runtime-neutral JS consumers pass
`Arc<JsCallbackHttpTransport>`. Transport policy for timeout, retry,
rate-limit, and jitter is injected through
`cow_sdk_core::transport::policy::TransportPolicy` per ADR 0041. Builders remain
transport-agnostic; new transports land as additive peers without changing the
builder API.

`OrderbookApi` and `SubgraphApi` construct exclusively through their typestate
builders. Marker types use private tuple fields so external crates cannot
construct them; the fully-set markers carry the value they prove is present
(chain id, environment or API key, and transport), so `.build()` reads each
input directly from the marker without unwrapping an `Option` or retaining a
typestate-guard panic, and it is reachable only from the fully-set state.
The builders expose a default-transport `.build()` on the transport-unset
typestate for every target: native targets default to `ReqwestTransport`, and
`wasm32` targets default to the browser `FetchTransport` backed by the realm's
global `fetch`. The policy timeout and response-byte cap apply to either
default; the browser default omits the user-agent because `User-Agent` is a
forbidden request header for `fetch`. Native targets additionally keep
`.client(reqwest::Client)` as a convenience over `ReqwestTransport`, and
explicit `.transport(...)` injection remains the customization seam on every
target.

## Why

A protocol SDK that ties orderbook and subgraph calls to a concrete
`reqwest::Client` forces every consumer to accept that backend. The browser
target cannot carry reqwest's native TLS stack, and JavaScript runtimes need a
callback transport. A single trait seam pulls dispatch one level up, keeps
native ergonomics, and lets transport choice remain caller-owned.

Typestate construction collapses the prior constructor family into one
reviewable path, proving chain, environment or API key, and transport are all
set before a live client exists.

## Must Remain True

- Public surface: `HttpTransport` is the production injection point for every
  REST or GraphQL call issued by orderbook and subgraph clients.
- Sole live dispatch: `OrderbookApi` and `SubgraphApi` each hold exactly one
  `Arc<dyn HttpTransport + Send + Sync>` and no parallel `reqwest::Client`, so
  the injected handle is the live handle. The success channel returns a
  `TransportResponse` (2xx status, response headers, body) while non-2xx
  responses stay on the typed `TransportError::HttpStatus { status, headers,
  body }` channel, so a calling layer never fabricates response metadata on the
  success path.
- Runtime and support: `TransportError` remains typed and URL-redacted on both
  native and browser adapters; retry and rate-limit orchestration stays above
  the transport trait.
- Validation and review: cross-adapter parity tests, builder contract tests,
  and trybuild compile-fail fixtures prove dispatch parity and marker sealing.
- Cost: the trait, transport leaf, policy object, and typestate builders add
  modest construction verbosity in exchange for one enforceable path.

## Alternatives Rejected

- Keep direct `reqwest::Client` constructors: familiar, but keeps parallel
  construction paths and excludes wasm transports.
- Expose one free-function constructor: shorter, but loses compile-time
  coverage for missing required inputs.
- Extract a separate per-target transport crate: cleaner boundary on paper,
  but the default transports already live in `cow-sdk-core` gated by target
  cfgs (per ADR 0010), so native builds carry no wasm-bindgen deps and an
  extra crate buys nothing.

## Links

- [Architecture](../guides/architecture.md)
- [Transport](../guides/transport.md)
- [Performance](../guides/performance.md)
- [Verification Guide](../guides/verification.md)
- See also: ADR 0030, ADR 0039, and ADR 0041.

**Proven by:**

- [HTTP Transport Contract Audit](../audit/http-transport-contract-audit.md)
- `crates/orderbook/tests/api_contract.rs`
- `crates/subgraph/tests/api_contract.rs`
