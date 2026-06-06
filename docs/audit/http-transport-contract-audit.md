# HTTP Transport Contract Audit

Status: Current
Last reviewed: 2026-06-06
Owning surface: `cow-sdk-core::HttpTransport` trait and the `ReqwestTransport` (native) and `FetchTransport` (browser) default adapters, including the sole-dispatch contract that binds every live REST or GraphQL call from `cow-sdk-orderbook` and `cow-sdk-subgraph` to the injected transport
Refresh trigger: Trait signature, method set, or dyn-compatibility posture changes on `HttpTransport`; changes to `TransportError` or `TransportErrorClass`; changes to the `TransportError::HttpStatus` shape; changes to the URL-stripping contract on either default adapter; any change to the shared `run_with_retry` driver's backoff schedule, jitter policy, retry tracing events, `Retry-After` honor contract, the `Retry-After` IMF-fixdate civil-day arithmetic, or the `system_now` wall clock; a new shipped adapter crate that adopts the trait; any change that lets a live REST or GraphQL call from `OrderbookApi` or `SubgraphApi` bypass `self.transport`
Related docs:
- [ADR 0013](../adr/0013-http-transport-injection-and-typestate-builders.md)
- [ADR 0019](../adr/0019-http-transport-sole-dispatch.md)
- [Transport](../transport.md)
- [Architecture](../architecture.md)
- [Credential Surface Contract Hygiene Audit](credential-surface-contract-hygiene-audit.md)
- [Bounded Response Reads Audit](bounded-response-reads-audit.md)
- [ADR 0055](../adr/0055-bounded-response-reads.md)

## Scope

This audit covers:

- the `HttpTransport` trait definition in `cow-sdk-core` and its
  dyn-compatibility posture
- the `ReqwestTransport` native default adapter and its URL-stripping
  contract on `reqwest::Error`
- the `FetchTransport` browser default adapter shipped from
  `cow-sdk-transport-wasm`
- the typed `TransportError` enum and the `TransportErrorClass` partition
  every adapter is expected to populate
- the shared `run_with_retry` driver's use of transport-surfaced
  `Retry-After` headers on `429` and `503` responses, its jittered backoff
  policy, its browser-safe wall clock, and its retry tracing event shape, as
  consumed by the orderbook, subgraph, and IPFS clients
- the sole-dispatch invariant that every live REST or GraphQL call from
  `OrderbookApi` or `SubgraphApi` flows through `self.transport` rather
  than a parallel HTTP client held inside those structs

It does not cover user-agent layering, the retry policy primitives and the
`run_with_retry` outcome contract themselves (see
`transport-policy-coverage-audit.md`), or the `Provider` chain-RPC seam (a
separate runtime contract).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Trait seam | `HttpTransport` is the sole production HTTP injection point and is dyn-compatible through `async-trait` with target-aware `Send` bounds | Conforms |
| Per-call controls | Every trait method carries per-call headers and an optional per-call timeout; adapters merge with constructor defaults and apply the deadline when supplied | Conforms |
| Typed failures | Every failure routes through `TransportError::Transport { class, detail }`, `TransportError::Configuration { message }`, or `TransportError::HttpStatus { status, headers, body }` | Conforms |
| Canonical HTTP and URL types | Orderbook and subgraph request code reaches `http` and `url` directly where the native client previously re-exported those types | Conforms |
| URL redaction | Both defaults strip URLs before wrapping so credential-bearing query strings never surface through `Debug` or `Display` | Conforms |
| Adapter parity | The native and browser adapters report the same `TransportErrorClass` for the same failure class on matching fixtures, and both surface non-2xx responses through `TransportError::HttpStatus` with the numeric status code preserved | Conforms |
| Retry cooldowns | The shared `run_with_retry` driver honors `Retry-After` on `429` and `503` for the orderbook, subgraph, and IPFS clients, waiting for the larger of the jittered local backoff and the server cooldown, evaluated against the browser-safe `system_now` wall clock | Conforms |
| Retry observability | The shared driver emits retry events that expose attempt index, backoff duration, and either response status or transport error class; the orderbook request methods record attempts and response status on the current span | Conforms |
| Write-retry idempotency | The driver replays writes (`POST`/`PUT`/`DELETE`) as well as reads on a retryable failure; this is safe because every CoW write endpoint is idempotent on the server (order creation by UID, cancellation by order state, app-data by hash), so a replay cannot create a duplicate side effect | Conforms |
| Sole-dispatch invariant | `OrderbookApi` and `SubgraphApi` hold only an `Arc<dyn HttpTransport + Send + Sync>` as their HTTP surface; every live REST and GraphQL call dispatches through that handle, and injected transports observe every request | Conforms |

## Current Contract

### Trait Seam

The trait lives at `cow_sdk_core::HttpTransport` and declares four async
methods (`get`, `post`, `put`, `delete`) that return `Result<String,
TransportError>`. Each method accepts per-call headers as a slice of
name/value pairs and an optional per-call timeout alongside the URL and
body. The trait is `#[async_trait]` on native targets (futures are
`Send`) and `#[async_trait(?Send)]` on `wasm32` targets so
`Arc<dyn HttpTransport + Send + Sync>` composes cleanly across native
consumers while the browser adapter stays viable; implementations
additionally carry `std::fmt::Debug` for derived `Debug` rendering on
consumer-facing clients.

### Typed Failure Surface

`TransportError::Transport { class, detail }` pairs a categorical
`TransportErrorClass` tag (`Timeout`, `Connect`, `Redirect`, `Decode`,
`Body`, `Builder`, `Request`, `Status`, `Upgrade`, `Other`) with a redacted
detail string. `TransportError::Configuration { message }` captures
builder-time failures that prevent a request from dispatching.
`TransportError::HttpStatus { status, headers, body }` captures a
non-2xx response so the calling layer receives the numeric status,
response headers, and raw response body through the typed error channel
rather than through an `Ok(String)` success path. Downstream error
aggregates
(`OrderbookError::Transport`, `SubgraphError::Transport`,
`SubgraphError::HttpStatus`, `AppDataError::Transport`) carry the same
partition. `Upgrade` is reserved for future HTTP protocol-upgrade
classification and is not produced by any current adapter.

### URL Redaction

`ReqwestTransport` invokes `reqwest::Error::without_url` before
wrapping every failure, so the URL never appears in the typed error.
Base URLs are held in the `Redacted<T>` newtype in
`cow-sdk-core::redaction` so debug, display, and serialized outputs of
the configuration never emit the raw URL either.
`FetchTransport` does not embed the URL in its detail string for the
same reason.

### Per-Call Controls

Adapters merge caller-supplied headers with any constructor-configured
defaults before dispatch. An `Option<Duration>` timeout on each call
overrides the transport's default request deadline; on the native
adapter the per-call timeout binds through `RequestBuilder::timeout`
on the underlying `reqwest` request, and on the browser adapter the
timeout wires an `AbortController` into the in-flight `fetch`
invocation.

### Adapter Parity

`ReqwestTransport` and `FetchTransport` share a fixture-driven parity
contract that exercises `Connect`, `Timeout`, and `Body` partitions
against matching synthetic errors and asserts both adapters map to the
same `TransportErrorClass`. Non-2xx responses surface through the
typed `TransportError::HttpStatus` variant on both runtimes with the
numeric status code, surfaced response headers, and raw response body
preserved. The `Redirect` variant is documented as unreachable in the
browser adapter (default `fetch` auto-follows redirects), so parity
there is empty by design.

### Retry Cooldowns

The shared `cow-sdk-transport-policy::run_with_retry` driver is the single
retry loop for the orderbook, subgraph, and IPFS clients. It reads
`Retry-After` from `TransportError::HttpStatus.headers` on
`429 Too Many Requests` and `503 Service Unavailable` responses. The parser
accepts both delta-seconds and HTTP-date forms, parse failures fall back to
the local exponential backoff schedule, and successful parses hold the retry
loop for the larger of the jittered local backoff and the server-supplied
cooldown. The HTTP-date branch performs its civil-day arithmetic in `i64` so
an attacker-controlled year value cannot panic the retry loop through integer
overflow; out-of-range timestamps fall back to local backoff through the
documented `checked_mul` guard. The "now" reference is the target-neutral
`system_now` wall clock, so an HTTP-date `Retry-After` evaluates against the
current time on both native and `wasm32` targets without the standard clock's
wasm abort. `RetryPolicy::with_jitter` accepts an explicit `JitterStrategy`;
`JitterStrategy::none` keeps deterministic wait schedules available for tests
and controlled callers. The driver honors the `max_attempts` limit, returns
immediately on a non-retryable transport class instead of re-dispatching, and
each client's attempt closure keeps applying its per-call timeout contract.

When the `tracing` feature is enabled, the shared driver emits a `debug`
event for every retry decision with `attempt_index`, `backoff_ms`, and
either `status` or `transport_error_class`; an exhausted retryable signal
emits the same field shape at `warn` level. The orderbook request methods
additionally record `attempts` and `status` on the current request span, and
the quote/order methods populate the documented `quote_id` field where the
value is available.

### Write-Retry Idempotency

The shared driver is method-agnostic: it replays a failed write
(`POST /orders`, `DELETE /orders`, `PUT` app-data) on the same retryable
transport classes and statuses as a read, matching the upstream
`@cowprotocol/cow-sdk` retry policy. This is safe because the CoW Protocol
write endpoints are idempotent on the server. Order creation is
content-addressed by order UID, so a replay is rejected as a duplicate
(`DuplicatedOrder`, a non-retryable `400`) rather than stored twice;
cancellation is keyed by order state, so a replay of an already-cancelled
order is a no-op (`AlreadyCancelled`, non-retryable `400`); app-data
registration is content-addressed by hash, so a replay matches the existing
entry. A quote carries no durable state. A replayed write therefore cannot
create a duplicate side effect. The one residual is benign and recoverable:
when a write commits on the server but its response is lost in transit, the
replay can surface the duplicate/already-cancelled rejection for an operation
that succeeded; callers confirm the committed state with an order lookup
(`GET /orders/{uid}`).

### Canonical Type Imports

`cow-sdk-orderbook` imports header and status types from `http` and parses
query strings with `url::Url`. `cow-sdk-subgraph` also parses explicit base
URLs with `url::Url`. These paths match the concrete crates re-exported by
the native client while keeping the browser target free of that client on the
orderbook, subgraph, and trading leaves.

### Sole-Dispatch Invariant

`OrderbookApi` and `SubgraphApi` hold only an
`Arc<dyn HttpTransport + Send + Sync>` as their HTTP surface. Every
public method dispatches through `self.transport.<get|post|put|delete>(...)`;
there is no parallel `reqwest::Client` field on either struct. The
orderbook's request pipeline runs the transport call inside the shared
`run_with_retry` driver, which provides the rate-limit gate, backoff, and
`Retry-After` cooldown, while the orderbook closure keeps the typed-error
classification around the single network-call line that previously invoked
`reqwest::RequestBuilder::send`. The subgraph's `run_query_with_config`
serializes the GraphQL envelope into a string, builds the
`Content-Type: application/json` header, and dispatches
`self.transport.post(api, body, headers, timeout).await` through the same
shared driver, mapping `TransportError::HttpStatus` straight into
`SubgraphError::HttpStatus`. Injected transports — including the
browser-native `FetchTransport` — therefore observe every live request.

## Evidence

Primary implementation points:

- `crates/core/src/transport/mod.rs`
- `crates/core/src/transport/http.rs`
- `crates/core/src/transport/error.rs`
- `crates/core/src/transport/reqwest.rs`
- `crates/transport-wasm/src/fetch.rs`
- `crates/transport-wasm/src/lib.rs`
- `crates/transport-policy/src/runner.rs`
- `crates/transport-policy/src/time.rs`
- `crates/orderbook/src/api.rs`
- `crates/orderbook/src/request.rs`
- `crates/orderbook/src/builder.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/builder.rs`
- `crates/wasm/src/exports/ipfs.rs`

Primary regression coverage:

- `crates/core/tests/transport_contract.rs`
- `crates/transport-wasm/tests/parity_contract.rs`
- `crates/transport-policy/tests/retry_after_contract.rs`
- `crates/transport-policy/tests/classify_contract.rs::network_error_kind_mapping_round_trip_is_total`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_031_retry_after_helper_is_case_insensitive`
- `crates/transport-policy/src/runner.rs` (`tests::non_retryable_transport_returns_without_redispatch`, `tests::http_date_retry_after_uses_the_injected_clock`, `tests::persistent_transport_error_exhausts_attempts`)
- `crates/wasm/tests/wasm_retry_runner_contract.rs::retryable_status_drives_backoff_without_panicking`
- `crates/transport-policy/tests/policy_contract.rs::prop_tpp_004_decorrelated_jitter_is_bounded_by_max_delay`
- `crates/orderbook/tests/request_contract.rs::tracing_contract::execute_with_emits_retry_events_with_status_and_transport_error_fields`
- `crates/orderbook/tests/request_contract.rs::tracing_contract::send_order_span_records_quote_id_attempts_and_status`
- `crates/orderbook/tests/api_contract.rs::service_unavailable_retry_after_header_delays_retry_for_at_least_server_cooldown`
- `crates/orderbook/tests/api_contract.rs::recording_transport::orderbook_get_order_dispatches_through_injected_transport`
- `crates/orderbook/tests/api_contract.rs::recording_transport::orderbook_send_order_dispatches_through_injected_transport`
- `crates/orderbook/tests/api_contract.rs::recording_transport::orderbook_delete_cancellation_dispatches_through_injected_transport`
- `crates/orderbook/tests/api_contract.rs::recording_transport::orderbook_rate_limit_and_backoff_still_apply_through_injected_transport`
- `crates/orderbook/tests/api_contract.rs::recording_transport::orderbook_non_2xx_surfaces_as_http_status_error_through_injected_transport`
- `crates/orderbook/tests/builder_contract.rs::injected_transport_observes_every_live_request_from_the_built_client`
- `crates/subgraph/tests/api_contract.rs::recording_transport::subgraph_run_query_dispatches_through_injected_transport`
- `crates/subgraph/tests/builder_contract.rs::injected_transport_observes_every_live_request_from_the_built_client`

Validation surface:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p cow-sdk-core --test transport_contract
cargo test -p cow-sdk-orderbook --test api_contract
cargo test -p cow-sdk-orderbook --test request_contract
cargo test -p cow-sdk-orderbook --features tracing --test request_contract
cargo test -p cow-sdk-subgraph --test api_contract
cargo check --workspace --all-features --target wasm32-unknown-unknown
wasm-pack test --headless --firefox crates/transport-wasm --all-features
```
