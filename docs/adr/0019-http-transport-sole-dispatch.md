# ADR 0019: HTTP Transport Is The Sole Live-Dispatch Surface On The Orderbook And Subgraph Clients

- Status: Accepted
- Date: 2026-04-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: transport, orderbook, subgraph, wasm, async, error-typing
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)

## Decision

`HttpTransport` in `cow-sdk-core` is the sole live-dispatch
surface on `OrderBookApi` and `SubgraphApi`. Both public clients
hold exactly one HTTP surface — `Arc<dyn HttpTransport + Send +
Sync>` — and every REST or GraphQL call they emit dispatches
through `self.transport.<get|post|put|delete>(...)`. The trait
carries per-call headers and an optional per-call timeout, and
`TransportError::HttpStatus { status, body }` carries non-2xx
responses through the typed error channel instead of through an
`Ok(String)` success path. Native builders default the transport
to `ReqwestTransport`; the browser target continues to require a
caller-supplied `FetchTransport` from `cow-sdk-transport-wasm`.
Rate limiting, retry and backoff, and typed error classification
stay at the orderbook and subgraph layers as orchestration around
the transport call; the transport itself stays bytes-in /
bytes-out.

## Why

ADR 0013 committed the crate family to transport injection as the
production seam, and consumers reach for `Arc<dyn HttpTransport>`
to compose recording, mocking, authentication, retry, and routing
layers around the live clients. Holding the trait object alongside
a parallel `reqwest::Client` on the client struct produced a
facade: the accessor exposed the injected transport while the
actual dispatch path bypassed it, so custom transports observed
nothing and the browser target could not rely on the injected
`FetchTransport` for real request delivery. Collapsing the dual
state onto a single transport surface forces the implementation to
match the contract ADR 0013 already commits to, makes recording
transports first-class reviewers of every live request, and keeps
the browser story coherent because the same transport seam serves
native and wasm targets. Extending the trait with per-call headers
and per-call timeouts folds the request-shape controls that
previously lived on `reqwest::RequestBuilder` onto the seam so the
trait remains expressive enough for the orderbook and subgraph
orchestration layers to preserve retry, rate-limiting, and
typed-error classification unchanged.

## Must Remain True

- Public surface: `OrderBookApi` and `SubgraphApi` hold only
  `transport: Arc<dyn HttpTransport + Send + Sync>` as their HTTP
  surface. No parallel `reqwest::Client` field exists on either
  struct. The `HttpTransport` trait signature carries `get`,
  `post`, `put`, and `delete` methods with
  `(url: &str, body: Option<&str>, headers: &[(String, String)],
  timeout: Option<Duration>) -> Result<String, TransportError>`
  (the `body` slot is absent on `get` and present on the other
  three methods in the reviewed signature). The
  `TransportError::HttpStatus { status, body }` variant ships on
  the `#[non_exhaustive]` error enum. Non-2xx responses surface
  through that variant on both `ReqwestTransport` and
  `FetchTransport`; the numeric status code and raw response body
  are preserved through the typed channel.
- Runtime and support: the orderbook and subgraph request
  pipelines preserve their existing orchestration — rate-limit
  acquire, backoff wrapper, user-agent and `Content-Type`
  headers, and typed-error classification — around the transport
  call. The single network-call line changed from
  `reqwest::RequestBuilder::send` to
  `self.transport.<method>(...)`, and every downstream typed
  error (`OrderbookError::Transport`,
  `SubgraphError::HttpStatus`, `AppDataError::Transport`) reads
  from the same `TransportError` partition
  (`Timeout`, `Connect`, `Redirect`, `Decode`, `Body`, `Builder`,
  `Request`, `Status`, `Other`) plus the new `HttpStatus` and
  the pre-existing `Configuration` variants. Native builders
  default the transport to `ReqwestTransport` constructed from
  the configured `HttpClientPolicy`; the
  `.client(reqwest::Client)` shorthand remains available on
  native targets and wraps the caller-supplied client into a
  `ReqwestTransport` so every live request still flows through
  the injected seam. Wasm builders continue to require an
  explicit `.transport(...)` with `FetchTransport` from
  `cow-sdk-transport-wasm`.
- Validation and review: the recording-transport regression
  modules at `crates/orderbook/tests/api_contract.rs` and
  `crates/subgraph/tests/api_contract.rs` inject an
  `HttpTransport` double that records every call and assert
  every method path (GET, POST, PUT, DELETE) is observed with
  the expected URL, body, headers, and timeout. The rate-limit
  and backoff coverage asserts the orchestration layer still
  fires around the transport call. The subgraph suite also
  asserts `SubgraphError::HttpStatus`, `SubgraphError::GraphQl`,
  and `SubgraphError::MissingData` paths surface correctly from
  the injected double. The builder contract tests at
  `crates/orderbook/tests/builder_contract.rs` and
  `crates/subgraph/tests/builder_contract.rs` confirm the
  injected pointer is the exact handle the API struct uses for
  live dispatch.
- Cost: four trait methods (unchanged count; `put` is the
  addition for app-data uploads), an additive
  `TransportError::HttpStatus` variant, a per-call header slice
  and optional timeout threaded onto each method, and the
  removal of the parallel `reqwest::Client` field on both
  clients. The default native build path remains a single
  `ReqwestTransport` construction behind the `HttpClientPolicy`
  seam, and the browser target retains its explicit transport
  requirement.

## Alternatives Rejected

- Keep the dual state and wire `self.transport` into dispatch
  alongside `self.client`: two HTTP surfaces remain reviewable
  in parallel, every future change must verify both stay
  consistent, and the `transport` accessor still does not prove
  the live path flows through the injected handle. The
  sole-dispatch rule makes the contract self-enforcing.
- Retire the `HttpTransport` seam and keep only
  `reqwest::Client` on the struct: collapses the dual state the
  other direction but breaks the ADR 0013 commitment and leaves
  the wasm target without a viable transport because
  `reqwest::Client` does not compile on `wasm32`.
- Move rate-limit, retry, and typed-error classification inside
  the transport trait so every adapter carries its own
  orchestration: the trait stops being a minimal bytes-in /
  bytes-out surface and every custom transport inherits
  orchestration it does not need. Keeping the orchestration at
  the orderbook and subgraph layer preserves the minimal trait
  shape ADR 0013 committed to.
- Skip the `HttpStatus` error variant and keep non-2xx responses
  in the `Ok(String)` success path: smaller type, but forces
  every downstream caller to rediscover the status-code channel
  and to pattern-match the response body as the single source of
  truth for failure classification.

## Links

- [Architecture](../architecture.md)
- [Transport](../transport.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)

**Proven by:**

- [HTTP Transport Contract Audit](../audit/http-transport-contract-audit.md)
