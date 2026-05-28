# ADR 0041: Share Transport Policy Across HTTP Clients

- Status: Accepted (amended)
- Date: 2026-05-08
- Last reviewed: 2026-05-28
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: transport, retry, layering
- Related: [ADR 0013](0013-http-transport-injection-and-typestate-builders.md), [ADR 0019](0019-http-transport-sole-dispatch.md), [ADR 0046](0046-transport-policy-js-exposure.md)

## Decision

Orderbook and subgraph retry, `Retry-After`, jitter, rate-limit, and
transport-error classification behavior lives in `cow-sdk-transport-policy`.
The crate sits above `cow-sdk-core::HttpTransport` and below typed clients.
`OrderBookApiBuilder` and `SubgraphApiBuilder` accept the shared
`TransportPolicy` through `.transport_policy(...)`.

The TypeScript-callable package exposes the same policy through a typed
`TransportPolicyConfig` on JavaScript client constructors. Omitting the config
preserves Rust defaults; invalid policy values fail during constructor
validation.

## Why

The raw transport trait should stay a dispatch seam. Retry and rate-limit
behavior depends on client semantics, HTTP status handling, and caller policy,
so embedding it in `cow-sdk-core` would make the trait harder to keep stable.
Keeping it separately owned gives orderbook and subgraph one consistent policy
without duplicating backoff code or tying the browser transport crate to native
HTTP details.

The clean migration also avoids parallel public names for the same behavior.
Downstream callers configure one policy type for orderbook and subgraph instead
of separate client-specific policy wrappers.

## Must Remain True

- Public surface: typed Rust clients consume `TransportPolicy`; JavaScript
  clients consume `TransportPolicyConfig`; moved policy types are not
  re-exported from orderbook or subgraph.
- Runtime and support: retryable statuses remain `408`, `425`, `429`, `500`,
  `502`, `503`, and `504`; `Retry-After` is honored for `429` and `503`;
  rate-limit state remains instance-scoped.
- Shared driver: the retry driver loop is owned by `cow-sdk-transport-policy`
  through `run_with_retry`. The orderbook, subgraph, and IPFS clients route
  their retries through that driver instead of hand-rolling a per-client loop,
  so the retry, backoff, `Retry-After`, rate-limit acquisition, and retry
  telemetry behavior is defined once. A non-retryable transport class returns
  immediately rather than re-dispatching.
- Wall clock: retry-delay computation reads the wall clock through the
  target-neutral `cow-sdk-transport-policy::system_now`, never
  `std::time::SystemTime::now()` on a wasm-reachable path, so a retryable
  response cannot abort a browser runtime.
- Validation and review: the transport-policy crate must test default
  orderbook and subgraph policy stability, no-retry behavior, jitter bounds,
  per-host limiter keying, status completeness, classifier totality, the
  `run_with_retry` outcome contract across the success, retry, exhaustion, and
  non-retryable cases, the `system_now` browser-safe clock, and TypeScript
  config translation for wasm clients.

## Alternatives Rejected

- Keep client-local policy types: this retained duplicated retry and
  `Retry-After` logic and made subgraph retry behavior diverge from orderbook.
- Move retry policy into `cow-sdk-core`: this widened the raw transport seam
  with client policy that belongs above dispatch.
- Provide compatibility aliases from orderbook or subgraph: aliases would
  preserve two names for one policy and make the public surface harder to audit.

## Links

- [Transport](../transport.md)
- [Architecture](../architecture.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)
- [ADR 0019](0019-http-transport-sole-dispatch.md)
- [WASM Public API Stability Audit](../audit/wasm-public-api-stability-audit.md)

**Proven by:**

- [Transport Policy Coverage Audit](../audit/transport-policy-coverage-audit.md)
- [HTTP Transport Contract Audit](../audit/http-transport-contract-audit.md)
- [WASM Public API Stability Audit](../audit/wasm-public-api-stability-audit.md)
