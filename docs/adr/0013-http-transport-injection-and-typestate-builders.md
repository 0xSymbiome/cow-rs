# ADR 0013: HTTP Transport Injection Seam And Typestate Construction For Orderbook And Subgraph

- Status: Accepted (amended)
- Date: 2026-04-21
- Last reviewed: 2026-04-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: transport, typestate, builders, wasm, async
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md)

## Decision

HTTP dispatch for the orderbook and subgraph surfaces flows through a
single typed `HttpTransport` trait in `cow-sdk-core`. The trait is
`dyn`-compatible through `async-trait` so consumers can hold transports
behind `Arc<dyn HttpTransport>`. The native default is `ReqwestTransport`
in `cow-sdk-core`; the browser default is `FetchTransport` shipped from
the dedicated `cow-sdk-transport-wasm` leaf crate. Construction of
`OrderBookApi` and `SubgraphApi` is exclusively through the
`OrderBookApiBuilder` and `SubgraphApiBuilder` typestate builders; the
legacy shared-client free-function constructors are retired.

## Why

A protocol SDK that ties orderbook and subgraph calls to a concrete
`reqwest::Client` forces every consumer — bot, analytics pipeline,
browser app, test harness — to accept that backend regardless of whether
it is a fit. The browser target in particular cannot carry `reqwest`'s
native TLS stack, so pinning the transport there was always an
artificial coupling. A single trait seam in `cow-sdk-core` pulls the
dispatch boundary one level up, keeps the default ergonomics unchanged
for native callers, and makes the browser adapter a peer rather than a
workaround. Replacing the five-plus free-function constructors on
`OrderBookApi` (and the six on `SubgraphApi`) with one typestate builder
per crate collapses the construction surface to one reviewable path,
encodes the three required inputs (chain, environment or API key,
transport) as compile-time markers, and removes an entire class of
silent mismatch between the client's chain, its environment, and its
installed transport.

## Must Remain True

- Public surface: `HttpTransport` in `cow-sdk-core` is the production
  injection point for every REST or GraphQL call the orderbook and
  subgraph clients issue. Implementations carry `Debug` and declare the
  `get`, `post`, and `delete` methods; the trait is `#[async_trait(?Send)]`
  so `Arc<dyn HttpTransport>` composes cleanly across native and browser
  callers. `OrderBookApi::builder()` returns
  `OrderBookApiBuilder<ChainIdUnset, EnvironmentUnset, TransportUnset>`
  and `SubgraphApi::builder()` returns the analogous three-marker
  builder; `.build()` is reachable only from the fully-set state on
  both, and the wasm32 default-transport convenience impl is gated on
  `#[cfg(not(target_arch = "wasm32"))]` so browser consumers must
  supply `FetchTransport` from `cow-sdk-transport-wasm` explicitly.
  `.client(reqwest_client)` remains available on native targets as a
  convenience over `ReqwestTransport` for multi-chain connection-pool
  reuse.
- Typestate marker types use private tuple fields so external crates
  cannot construct them. A `trybuild` `compile_fail` fixture under
  `crates/<crate>/tests/ui/` proves the sealing for every typed client.
- Runtime and support: `TransportError` is a typed enum with a
  `Transport { class: TransportErrorClass, detail: String }` variant
  partitioned across `Timeout`, `Connect`, `Redirect`, `Decode`, `Body`,
  `Builder`, `Request`, `Status`, and `Other`, plus a `Configuration`
  variant for builder-time failures. Native and browser adapters strip
  the URL (through `reqwest::Error::without_url` on the native side and
  through explicit URL omission on the browser side) before wrapping,
  so credential-bearing query strings never surface through `Display`
  or `Debug`. The transport-policy layer on the orderbook and subgraph
  clients (retry, rate-limit, user-agent) sits above the trait and is
  unchanged.
- Validation and review: a cross-adapter parity contract test exercises
  `ReqwestTransport` and `FetchTransport` against matching fixtures and
  asserts both adapters surface the same `TransportErrorClass` for the
  same failure class. A `trybuild` UI witness asserts that calling
  `.build()` on a `SubgraphApiBuilder` without `.transport(...)` on a
  `wasm32` target fails to compile; the analogous native happy-path
  test covers the `ReqwestTransport` default. Every caller in the
  trading surface, the examples workspace, and the browser-wallet
  console uses one of the two builders — no free-function constructors
  remain.
- Cost: one new trait in `cow-sdk-core`, one new leaf crate
  (`cow-sdk-transport-wasm`) for the browser transport, and one new
  builder per public client. The `async-trait` macro adds a small amount
  of generated code per method. The construction path becomes slightly
  more verbose (three setters plus `.build()`) in exchange for the
  compile-time guarantee that the three required inputs are present.

## Alternatives Rejected

- Keep the direct `reqwest::Client` constructor family on each public
  client: familiar, but forces every wasm consumer to own the browser
  transport integration and keeps five-plus free-function constructors
  on `OrderBookApi` alone as parallel construction paths that drift over
  time.
- Expose a single `OrderBookApi::new(chain, env, transport)` free
  function and drop the builder altogether: shorter, but removes the
  compile-time coverage for "forgot to set chain" that the typestate
  markers provide and loses the fluent extension points (policy,
  shared client, base-URL override) that callers actually use.
- Keep the HTTP seam as a plain `async fn in trait` and rely on
  specialized generics instead of `dyn` compatibility: workable for
  a single callsite, but makes `Arc<dyn HttpTransport>` composition
  (the shape capability crates and the transport-wasm adapter both
  reach for) either impossible or `Box<dyn ...>`-heavy.
- Ship the browser transport inside `cow-sdk-core` behind a cfg flag:
  smaller surface area, but pins every native consumer to a
  wasm-bindgen dependency graph they never run, and makes the browser
  transport a second-class inhabitant of the core crate.

## Links

- [Architecture](../architecture.md)
- [Transport](../transport.md)
- [Performance](../performance.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md)
- [ADR 0011](0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [ADR 0019](0019-http-transport-sole-dispatch.md)

**Proven by:**

- [ADR 0019](0019-http-transport-sole-dispatch.md)
- [HTTP Transport Contract Audit](../audit/http-transport-contract-audit.md)
- [Typestate Builder Contract Audit](../audit/typestate-builder-contract-audit.md)
- `crates/orderbook/tests/api_contract.rs`
- `crates/subgraph/tests/api_contract.rs`
