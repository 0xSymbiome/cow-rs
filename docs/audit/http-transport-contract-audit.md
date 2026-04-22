# HTTP Transport Contract Audit

Status: Current
Last reviewed: 2026-04-22
Owning surface: `cow-sdk-core::HttpTransport` trait and the `ReqwestTransport` (native) and `FetchTransport` (browser) default adapters, including the sole-dispatch contract that binds every live REST or GraphQL call from `cow-sdk-orderbook` and `cow-sdk-subgraph` to the injected transport
Refresh trigger: Trait signature, method set, or dyn-compatibility posture changes on `HttpTransport`; changes to `TransportError` or `TransportErrorClass`; changes to the URL-stripping contract on either default adapter; a new shipped adapter crate that adopts the trait; any change that lets a live REST or GraphQL call from `OrderBookApi` or `SubgraphApi` bypass `self.transport`
Related docs:
- [ADR 0013](../adr/0013-http-transport-injection-and-typestate-builders.md)
- [Transport](../transport.md)
- [Architecture](../architecture.md)
- [Credential Surface Contract Hygiene Audit](credential-surface-contract-hygiene-audit.md)

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
- the sole-dispatch invariant that every live REST or GraphQL call from
  `OrderBookApi` or `SubgraphApi` flows through `self.transport` rather
  than a parallel HTTP client held inside those structs

It does not cover transport-policy retry, rate-limit, or user-agent
layering built on top of the trait by `cow-sdk-orderbook` and
`cow-sdk-subgraph`, and it does not cover the `AsyncProvider` chain-RPC
seam (a separate runtime contract).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Trait seam | `HttpTransport` is the sole production HTTP injection point and is dyn-compatible through `async-trait` with target-aware `Send` bounds | Conforms |
| Per-call controls | Every trait method carries per-call headers and an optional per-call timeout; adapters merge with constructor defaults and apply the deadline when supplied | Conforms |
| Typed failures | Every failure routes through `TransportError::Transport { class, detail }`, `TransportError::Configuration { message }`, or `TransportError::HttpStatus { status, body }` | Conforms |
| URL redaction | Both defaults strip URLs before wrapping so credential-bearing query strings never surface through `Debug` or `Display` | Conforms |
| Adapter parity | The native and browser adapters report the same `TransportErrorClass` for the same failure class on matching fixtures, and both surface non-2xx responses through `TransportError::HttpStatus` with the numeric status code preserved | Conforms |
| Sole-dispatch invariant | `OrderBookApi` and `SubgraphApi` hold only an `Arc<dyn HttpTransport + Send + Sync>` as their HTTP surface; every live REST and GraphQL call dispatches through that handle, and injected transports observe every request | Conforms |

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
`Body`, `Builder`, `Request`, `Status`, `Other`) with a redacted
detail string. `TransportError::Configuration { message }` captures
builder-time failures that prevent a request from dispatching.
`TransportError::HttpStatus { status, body }` captures a non-2xx
response so the calling layer receives the numeric status and raw
response body through the typed error channel rather than through an
`Ok(String)` success path. Downstream error aggregates
(`OrderbookError::Transport`, `SubgraphError::Transport`,
`SubgraphError::HttpStatus`, `AppDataError::Transport`) carry the same
partition.

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
numeric status code and raw response body preserved. The `Redirect`
variant is documented as unreachable in the browser adapter (default
`fetch` auto-follows redirects), so parity there is empty by design.

### Sole-Dispatch Invariant

`OrderBookApi` and `SubgraphApi` hold only an
`Arc<dyn HttpTransport + Send + Sync>` as their HTTP surface. Every
public method dispatches through `self.transport.<get|post|put|delete>(...)`;
there is no parallel `reqwest::Client` field on either struct. The
orderbook's request pipeline preserves its rate-limit gate, retry and
backoff wrapper, and typed-error classification around the transport
call, replacing only the single network-call line that previously
invoked `reqwest::RequestBuilder::send`. The subgraph's
`run_query_with_config` serializes the GraphQL envelope into a string,
builds the `Content-Type: application/json` header, and calls
`self.transport.post(&api, &body, &headers, self.client_policy().timeout()).await`,
mapping `TransportError::HttpStatus` straight into
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
- `crates/orderbook/src/api.rs`
- `crates/orderbook/src/request.rs`
- `crates/orderbook/src/builder.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/builder.rs`

Primary regression coverage:

- `crates/core/tests/transport_contract.rs`
- `crates/transport-wasm/tests/parity_contract.rs`
- `crates/orderbook/tests/api_contract.rs::recording_transport::orderbook_get_order_dispatches_through_injected_transport`
- `crates/orderbook/tests/api_contract.rs::recording_transport::orderbook_send_order_dispatches_through_injected_transport`
- `crates/orderbook/tests/api_contract.rs::recording_transport::orderbook_delete_cancellation_dispatches_through_injected_transport`
- `crates/orderbook/tests/api_contract.rs::recording_transport::orderbook_rate_limit_and_backoff_still_apply_through_injected_transport`
- `crates/orderbook/tests/api_contract.rs::recording_transport::orderbook_non_2xx_surfaces_as_http_status_error_through_injected_transport`
- `crates/orderbook/tests/builder_contract.rs::injected_transport_observes_every_live_request_from_the_built_client`
- `crates/subgraph/tests/api_contract.rs::recording_transport::subgraph_run_query_dispatches_through_injected_transport`
- `crates/subgraph/tests/api_contract.rs::recording_transport::subgraph_errors_field_surfaces_as_graphql_error_through_injected_transport`
- `crates/subgraph/tests/api_contract.rs::recording_transport::subgraph_missing_data_surfaces_as_missing_data_error_through_injected_transport`
- `crates/subgraph/tests/api_contract.rs::recording_transport::subgraph_http_status_error_propagates_through_injected_transport`
- `crates/subgraph/tests/builder_contract.rs::injected_transport_observes_every_live_request_from_the_built_client`

Validation surface:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo test -p cow-sdk-orderbook --test api_contract
cargo test -p cow-sdk-subgraph --test api_contract
cargo check --workspace --all-features --target wasm32-unknown-unknown
```
