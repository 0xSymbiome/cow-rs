# Typestate Builder Contract Audit

Status: Current
Last reviewed: 2026-04-25
Owning surface: `cow-sdk-orderbook::OrderBookApiBuilder` and `cow-sdk-subgraph::SubgraphApiBuilder` construction seams
Refresh trigger: Type-parameter or marker visibility changes on either builder, a change to the set of required inputs (chain, environment or API key, transport), a change to the native default-transport convenience impl, a change to the wasm32 transport-required invariant, or a new `trybuild` witness replacing the current compile-fail coverage
Related docs:
- [ADR 0013](../adr/0013-http-transport-injection-and-typestate-builders.md)
- [Transport](../transport.md)
- [Architecture](../architecture.md)

## Scope

This audit covers:

- the three-marker `OrderBookApiBuilder` typestate
  (`ChainState`, `EnvironmentState`, `TransportState`) and the single
  `.build()` path
- the three-marker `SubgraphApiBuilder` typestate
  (`ChainState`, `ApiKeyState`, `TransportState`) and the single
  `.build()` path
- the native default-transport convenience on both builders and its
  `#[cfg(not(target_arch = "wasm32"))]` gate
- the sealed marker structs that prevent direct external construction of
  typestate witnesses
- the wasm32 transport-required invariant proven by a `trybuild`
  compile-fail witness
- the retirement of the legacy free-function constructors on
  `OrderBookApi` and `SubgraphApi`

It does not cover transport-policy retry, rate-limit, or user-agent
layering (the policy surface sits above the builder and is covered by
a separate contract), and it does not cover the `TradingSdkBuilder`
typestate (covered by the trading-sdk runtime prerequisites audit).

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| OrderBookApi construction | `OrderBookApi::builder()` is the only production construction path; every required input is encoded as a compile-time marker | Conforms |
| SubgraphApi construction | `SubgraphApi::builder()` is the only production construction path; every required input is encoded as a compile-time marker | Conforms |
| Marker sealing | Public marker types use private tuple fields, so external callers cannot construct typestate witnesses directly | Conforms |
| Native convenience | Both builders carry a default-transport `.build()` impl gated on `#[cfg(not(target_arch = "wasm32"))]` that installs a `ReqwestTransport` | Conforms |
| wasm32 invariant | `trybuild` compile-fail coverage asserts `.build()` without `.transport(...)` does not compile on `wasm32` | Conforms |

## Current Contract

### OrderBookApi Construction

`OrderBookApiBuilder<ChainState, EnvironmentState, TransportState>`
lives at `crates/orderbook/src/builder.rs`. Each marker transitions
from `…Unset` to `…Set` through the corresponding fluent setter
(`.chain(...)`, `.environment(...)`, `.transport(...)`). The `.build()`
method is implemented only on the fully-set state; attempting to
call it before every marker is set is a compile error. The fluent
layer additionally exposes optional setters for transport policy,
shared `reqwest::Client` reuse on native targets, and per-chain base-URL
overrides.

The public marker types are tuple structs with private unit fields. The
type names remain available in builder type signatures and diagnostics,
but callers outside the defining module cannot construct `Marker(())`
values directly.

### SubgraphApi Construction

`SubgraphApiBuilder<ChainState, ApiKeyState, TransportState>` lives at
`crates/subgraph/src/builder.rs` and follows the same typestate shape
with `ApiKey` in place of `Environment`. The partner API key is
wrapped in the `Redacted<T>` newtype at the setter boundary so debug,
display, and serialized output of the configuration never emit the
raw key.

The subgraph markers use the same private-field tuple shape, keeping
external construction closed while preserving the public type names used
by the builder state machine.

### Native Convenience And wasm32 Invariant

On non-`wasm32` targets, a convenience `.build()` impl is defined on the
`(ChainSet, EnvironmentSet | ApiKeySet, TransportUnset)` state and
installs a default `ReqwestTransport`. On `wasm32` this convenience impl
is absent, so a caller must invoke `.transport(...)` explicitly to
reach `.build()`. The `trybuild` UI harness at
`crates/subgraph/tests/ui/builder_wasm32_missing_transport.rs` captures
the expected compile error and its stderr fixture.

### Legacy Constructor Retirement

The legacy free-function constructors on `OrderBookApi` (`new`,
`new_with_transport_policy`, `from_shared_client`,
`from_shared_client_with_transport_policy`, `new_with_base_url`) and
on `SubgraphApi` (`new`, `with_config`, `with_config_and_transport_policy`,
`from_shared_client`, `from_shared_client_with_config`,
`from_shared_client_with_transport_policy`) have been retired. No
free-function public constructor remains on either type; every caller
in the trading surface, examples workspace, and e2e suite constructs
through the typestate builder.

## Evidence

Primary implementation points:

- `crates/orderbook/src/builder.rs`
- `crates/orderbook/src/api.rs`
- `crates/subgraph/src/builder.rs`
- `crates/subgraph/src/api.rs`

Primary regression coverage:

- `crates/orderbook/tests/builder_contract.rs`
- `crates/subgraph/tests/builder_contract.rs`
- `crates/subgraph/tests/ui/builder_wasm32_missing_transport.rs`

Validation surface:

```text
cargo test -p cow-sdk-orderbook --all-features
cargo test -p cow-sdk-subgraph --all-features
cargo check --workspace --all-features --target wasm32-unknown-unknown
cargo clippy -p cow-sdk-orderbook -p cow-sdk-subgraph --all-targets --all-features -- -D warnings
```
