# HTTP Transport Contract Audit

Status: Current  
Last reviewed: 2026-04-21  
Owning surface: `cow-sdk-core::HttpTransport` trait and the `ReqwestTransport` (native) and `FetchTransport` (browser) default adapters  
Refresh trigger: Trait signature, method set, or dyn-compatibility posture changes on `HttpTransport`; changes to `TransportError` or `TransportErrorClass`; changes to the URL-stripping contract on either default adapter; a new shipped adapter crate that adopts the trait  
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

It does not cover transport-policy retry, rate-limit, or user-agent
layering built on top of the trait by `cow-sdk-orderbook` and
`cow-sdk-subgraph`, and it does not cover the `AsyncProvider` chain-RPC
seam (a separate runtime contract).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Trait seam | `HttpTransport` is the sole production HTTP injection point and is dyn-compatible through `async-trait` | Conforms |
| Typed failures | Every failure routes through `TransportError::Transport { class, detail }` or `TransportError::Configuration { message }` | Conforms |
| URL redaction | Both defaults strip URLs before wrapping so credential-bearing query strings never surface through `Debug` or `Display` | Conforms |
| Adapter parity | The native and browser adapters report the same `TransportErrorClass` for the same failure class on matching fixtures | Conforms |

## Current Contract

### Trait Seam

The trait lives at `cow_sdk_core::HttpTransport` and declares three async
methods (`get`, `post`, `delete`) that return `Result<String,
TransportError>`. The trait is `#[async_trait(?Send)]` so
`Arc<dyn HttpTransport>` composes cleanly across native and browser
callers; implementations additionally carry `std::fmt::Debug` for
derived `Debug` rendering on consumer-facing clients.

### Typed Failure Surface

`TransportError::Transport { class, detail }` pairs a categorical
`TransportErrorClass` tag (`Timeout`, `Connect`, `Redirect`, `Decode`,
`Body`, `Builder`, `Request`, `Status`, `Other`) with a redacted
detail string. `TransportError::Configuration { message }` captures
builder-time failures that prevent a request from dispatching.
Downstream error aggregates (`OrderbookError::Transport`,
`SubgraphError::Transport`, `AppDataError::Transport`) carry the same
partition.

### URL Redaction

`ReqwestTransport` invokes `reqwest::Error::without_url` before
wrapping every failure, so the URL never appears in the typed error.
Base URLs are held in the `Redacted<T>` newtype in
`cow-sdk-core::redaction` so debug, display, and serialized outputs of
the configuration never emit the raw URL either.
`FetchTransport` does not embed the URL in its detail string for the
same reason.

### Adapter Parity

`ReqwestTransport` and `FetchTransport` share a fixture-driven parity
contract that exercises `Timeout`, `Connect`, `Decode`, `Body`, and
`Status` outcomes against matching synthetic errors and asserts both
adapters map to the same `TransportErrorClass`. The `Redirect` variant
is documented as unreachable in the browser adapter (default `fetch`
auto-follows redirects), so parity there is empty by design.

## Evidence

Primary implementation points:

- `crates/core/src/transport/mod.rs`
- `crates/core/src/transport/http.rs`
- `crates/core/src/transport/reqwest.rs`
- `crates/transport-wasm/src/fetch.rs`
- `crates/transport-wasm/src/lib.rs`

Primary regression coverage:

- `crates/core/tests/transport_contract.rs`
- `crates/transport-wasm/tests/parity_contract.rs`

Validation surface:

```text
cargo test -p cow-sdk-core --all-features
cargo test -p cow-sdk-transport-wasm --target wasm32-unknown-unknown
cargo clippy -p cow-sdk-core --all-targets --all-features -- -D warnings
cargo check -p cow-sdk-transport-wasm --target wasm32-unknown-unknown
```
