# Cooperative Cancellation Contract Audit

Status: Current
Last reviewed: 2026-05-01
Owning surface: Cross-cutting cooperative cancellation across `cow-sdk-core`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, and `cow-sdk-trading`
Refresh trigger: Changes to the `Cancellable` combinator, to the `CancellationToken` re-export, to the canonical long-running public methods on the three client surfaces, or to the `From<Cancelled>` bridges on the typed error aggregates
Related docs:
- [ADR 0005](../adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)
- [ADR 0010](../adr/0010-runtime-neutral-async-and-transport-posture.md)
- [Architecture](../architecture.md)
- [Observability](../observability.md)

## Scope

This audit covers:

- the shared `CancellationToken` re-export on `cow-sdk-core`
- the `Cancellable` extension trait and its `WithCancellation<'t, F>`
  wrapper on `cow-sdk-core`
- the canonical long-running public methods on `OrderBookApi`,
  `SubgraphApi`, and `TradingSdk`, each composed with the combinator at
  the call site
- typed `Cancelled` variants on `CoreError`, `OrderbookError`,
  `SubgraphError`, `TradingError`, `SigningError`, `BrowserWalletError`,
  and the facade `SdkError`, plus the `From<Cancelled>` bridges that lift
  the marker across every public error boundary
- the biased cancellation poll inside the combinator that drops in-flight
  request futures promptly when the caller cancels

It does not cover browser-wallet session cancellation, unrelated transport
policy, or future capability crates outside the published surface.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Shared token import | One typed cancellation token re-export across every public crate | Conforms |
| Canonical public methods | Every long-running public operation on `OrderBookApi`, `SubgraphApi`, and `TradingSdk` is exposed as one canonical async method; cancellation composes through `Cancellable::cancel_with(&token)` at the call site | Conforms |
| Typed `Cancelled` variants | Every affected error aggregate surfaces cancellation as a discrete typed variant, and each carries a `From<Cancelled>` bridge so the marker propagates with `?` across every public boundary | Conforms |
| Combinator poll | The combinator polls the token in a biased branch, drops the inner future promptly on cancellation, and routes the marker through the inner result's `From<Cancelled>` implementation | Conforms |

## Current Contract

### Shared Token Import

`cow-sdk-core` re-exports `tokio_util::sync::CancellationToken` as
`cow_sdk_core::CancellationToken`. Every downstream crate routes
cancellation through that one typed import so consumers do not mix
independent tokens across crate boundaries.

### `Cancellable` Combinator

`cow-sdk-core` exposes the `Cancellable` extension trait, implemented for
every `Future`, plus the `WithCancellation<'t, F>` future wrapper. Callers
compose cancellation by wrapping any returned future through
`cow_sdk_core::Cancellable::cancel_with(&token)` at the call site. The
wrapper polls the borrowed `CancellationToken` in a biased branch before
each inner poll; when the token fires, the wrapper drops the inner future
promptly and resolves to the typed `Cancelled` variant through the inner
result's `From<Cancelled>` implementation.

### Canonical Public Methods

Every long-running public operation on `OrderBookApi`, `SubgraphApi`, and
`TradingSdk` is exposed as one canonical async method that performs its
request directly and carries the observability instrumentation for the
operation. Callers that need cooperative cancellation wrap that future
through the combinator at the call site.

### Typed `Cancelled` Variants And `From<Cancelled>` Bridges

`CoreError`, `OrderbookError`, `SubgraphError`, `TradingError`,
`SigningError`, `BrowserWalletError`, and the facade `SdkError` each
expose a typed `Cancelled` variant and an
`impl From<cow_sdk_core::Cancelled>` that lifts the marker into that
variant. Operation code therefore propagates cancellation with `?` across
every public error boundary without pulling the raw `tokio-util` future
type into downstream signatures. Facade classification through
`SdkError::class()` routes every reachable `Cancelled` variant to
`ErrorClass::Cancelled`.

### Combinator Poll And Drop Semantics

The `WithCancellation<'t, F>` wrapper drives a biased poll against the
borrowed token's cancellation future. When the token fires, the wrapper
returns `Poll::Ready(Err(E::from(Cancelled)))` and the caller's `.await`
drops the inner future, so the underlying socket releases promptly rather
than waiting for the request deadline. Cancellation is cooperative: the
caller owns the token and can clone it to propagate shutdown across
multiple SDK instances.

## Evidence

Primary implementation points:

- `crates/core/src/cancellation.rs`
- `crates/core/src/lib.rs`
- `crates/core/src/errors.rs`
- `crates/orderbook/src/api.rs`
- `crates/orderbook/src/error.rs`
- `crates/subgraph/src/api.rs`
- `crates/subgraph/src/error.rs`
- `crates/trading/src/sdk.rs`
- `crates/trading/src/post.rs`
- `crates/trading/src/error.rs`
- `crates/signing/src/errors.rs`
- `crates/browser-wallet/src/error.rs`
- `crates/sdk/src/lib.rs`

Primary regression coverage:

- `crates/core/tests/cancellation_contract.rs`
- `crates/core/tests/cancellation_coverage_validator.rs`
- `crates/orderbook/tests/api_contract.rs`
- `crates/orderbook/tests/cancellation_composition_contract.rs`
- `crates/orderbook/tests/request_contract.rs::retry_after_backoff_wait_can_be_cancelled_before_next_attempt`
- `crates/subgraph/tests/api_contract.rs`
- `crates/subgraph/tests/cancellation_composition_contract.rs`
- `crates/trading/tests/sdk_contract.rs`
- `crates/trading/tests/cancellation_composition_contract.rs`

Validation surface:

```text
cargo fmt --all --check
cargo test -p cow-sdk-core --test cancellation_coverage_validator
cargo test -p cow-sdk-orderbook --test cancellation_composition_contract
cargo test -p cow-sdk-orderbook --test request_contract
cargo test -p cow-sdk-subgraph --test cancellation_composition_contract
cargo test -p cow-sdk-trading --test cancellation_composition_contract
cargo test -p cow-sdk-core
cargo test -p cow-sdk-orderbook
cargo test -p cow-sdk-subgraph
cargo test -p cow-sdk-trading
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo build --target wasm32-unknown-unknown -p cow-sdk
```
