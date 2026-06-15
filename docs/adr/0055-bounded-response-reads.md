# ADR 0055: HTTP Response Reads Are Bounded By A Configurable Per-Client Byte Limit

- Status: Accepted
- Date: 2026-05-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: transport, security, orderbook, subgraph, ipfs, wasm, error-typing
- Related: [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0019](0019-http-transport-sole-dispatch.md), [ADR 0025](0025-workspace-url-redaction-convention.md), [ADR 0033](0033-minimum-viable-panic-surface.md), [ADR 0040](0040-wallet-provider-callback-boundary-for-js-consumers.md), [ADR 0041](0041-transport-policy-l3-layering.md), [ADR 0046](0046-transport-policy-js-exposure.md)

## Decision

Every HTTP response body that the SDK buffers is bounded by a configurable
maximum size, in decoded bytes, carried on `HttpClientPolicy` as
`max_response_bytes`:

- The native `ReqwestTransport` reads the body as a stream of chunks and
  refuses to buffer past the limit, so an over-large body — including a
  decompression-amplified one — is rejected after at most one over-limit
  chunk rather than fully materialized. The bound applies to both the
  success body and the non-2xx error body.
- The browser `FetchTransport` and the runtime-neutral JS-callback transport
  apply the same limit as a post-receipt bound: the surrounding JS layer
  materializes the body, and the SDK refuses to process a body that exceeds
  the limit.
- Refusal surfaces as `TransportError::Transport { class:
  TransportErrorClass::ResponseTooLarge, .. }`. The classification is
  deterministic for a given response, so it is never retried.
- Per-client defaults differ by trust: the orderbook and trading clients use
  the generous workspace default, the untrusted subgraph gateway uses a
  tighter default, and the IPFS app-data read uses a bound sized to the
  protocol app-data document limit.
- Signature hex fields are length-bounded before the hex decoder allocates,
  using a generous bound equal to the orderbook request-body limit so a valid
  signature is never rejected while non-transport input cannot drive an
  unbounded decode allocation.

## Why

Responses from untrusted third-party infrastructure — the subgraph gateway,
JSON-RPC providers, and IPFS gateways — are the SDK's primary untrusted input.
Reading an unbounded body into memory lets a hostile, misbehaving, or
intermediary-tampered source exhaust process memory, and transparent
decompression lets a small compressed body amplify into a very large buffer.
Bounding the read at the point the SDK owns the loop converts that failure mode
into a typed, non-retryable rejection. Keeping the bound on `HttpClientPolicy`
makes it instance-scoped and per-client tunable rather than a single global
constant, consistent with the policy-contract rule.

## Must Remain True

- Public surface: `HttpClientPolicy`, `ReqwestTransportConfig`, and
  `FetchTransportConfig` expose `max_response_bytes` getters and builder
  setters; the transport policy sets per-client defaults; a new
  `TransportErrorClass::ResponseTooLarge` flows through the existing
  `Transport { class, detail }` channel and through every downstream error
  surface with its classification intact.
- Runtime and support: the bound is on decoded bytes, the only sound bound
  when transparent decompression is active; the JS-owned and RPC-stack-owned
  read loops are bounded by a post-receipt check and the request timeout
  rather than a streamed cap, with the residual documented in the security
  policy.
- Validation and review: regression tests prove rejection of an over-limit
  body, a decompression bomb (bounding decoded bytes), an over-limit error
  body, the exact-limit boundary, lenient decoding of non-UTF-8 bodies, the
  non-retryable classification, the per-client default values, and the
  signature pre-decode bound.
- Cost: the streamed read holds the accumulator plus one in-flight chunk
  instead of using a single buffered read; the limit is one additional policy
  field.

## Alternatives Rejected

- A `Content-Length` pre-check: the header is the compressed size and is
  often absent under transparent decompression, so it cannot bound a
  decompression bomb.
- A single global maximum constant: it cannot express the different trust
  posture of the trusted orderbook versus an untrusted gateway and conflicts
  with instance-scoped policy.
- A tight, EIP-1271-specific signature length cap: the protocol imposes no
  signature-length maximum, so a tight cap would reject valid smart-account
  signatures; the bound is set to the request-body limit instead.
- Wrapping the JSON-RPC transport stack to cap its read: it would couple the
  SDK to that stack's internals against the runtime-neutral transport posture;
  removing response decompression on that client plus the request timeout is
  the proportionate mitigation, with the residual documented.

## Links

- [Architecture](../architecture.md)
- [Security Policy](../../SECURITY.md)
- [ADR 0019](0019-http-transport-sole-dispatch.md)
- [ADR 0041](0041-transport-policy-l3-layering.md)

**Proven by:**

- [Bounded Response Reads Audit](../audit/bounded-response-reads-audit.md)
