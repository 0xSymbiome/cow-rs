# ADR 0019: HTTP Transport Is The Sole Live-Dispatch Surface On The Orderbook And Subgraph Clients

- Status: Accepted (amended)
- Date: 2026-04-22
- Last reviewed: 2026-05-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: transport, orderbook, subgraph, wasm, async, error-typing
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0013](0013-http-transport-injection-and-typestate-builders.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md)

## Decision

`HttpTransport` in `cow-sdk-core` is the sole live-dispatch surface on
`OrderbookApi` and `SubgraphApi`. Both clients hold exactly one HTTP surface:
`Arc<dyn HttpTransport + Send + Sync>`. Every REST or GraphQL request dispatches
through `self.transport.<get|post|put|delete>(...)`; no parallel
`reqwest::Client` field exists on either client.

The trait carries per-call headers and optional timeout inputs, and
`TransportError::HttpStatus { status, headers, body }` carries non-2xx
responses through the typed error channel. Native builders default to
`ReqwestTransport`, browser wasm callers inject `FetchTransport`, and
runtime-neutral JS callers inject `JsCallbackHttpTransport`.

The sole-dispatch invariant extends to the JS callback transport:
`JsCallbackHttpTransport::send` is the sole Rust dispatch point for
runtime-neutral JS consumers. The callback delegates wire I/O to JavaScript,
but Rust-side dispatch remains a single named method on one trait
implementation per consumer instance.

## Why

ADR 0013 made transport injection the production seam. Keeping an injected
transport beside a parallel native client produced a facade: the accessor
exposed the injected handle while live calls could bypass it. Collapsing the
dual state makes recording, mocking, authentication, retry, and routing
transports first-class reviewers of every live request and keeps native,
browser, and callback transports under one rule.

## Must Remain True

- Public surface: orderbook and subgraph structs hold one transport trait
  object and every live request flows through it.
- Runtime and support: rate-limit acquire, backoff, user-agent and
  `Content-Type` headers, and typed error classification remain orchestration
  around the transport call.
- Validation and review: recording-transport regression tests observe every
  method path with expected URL, body, headers, and timeout; builder contract
  tests prove the injected handle is the live handle.
- Cost: the trait carries four methods, per-call headers, optional timeouts,
  and `HttpStatus`, but eliminates the hidden second dispatch surface.

## Alternatives Rejected

- Keep dual state and wire both surfaces carefully: future changes would still
  need to prove two HTTP paths stay coherent.
- Retire `HttpTransport` and keep only `reqwest::Client`: breaks ADR 0013 and
  leaves wasm targets without a viable transport.
- Put retry and rate-limit inside the transport trait: makes every adapter own
  orchestration it does not need.

## Links

- [Architecture](../architecture.md)
- [Transport](../transport.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)
- See also: ADR 0039.

**Proven by:**

- [HTTP Transport Contract Audit](../audit/http-transport-contract-audit.md)
