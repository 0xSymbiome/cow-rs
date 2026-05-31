# Typestate Builder Contract Audit

Status: Current
Last reviewed: 2026-05-31
Owning surface: `cow-sdk-orderbook::OrderbookApiBuilder`, `cow-sdk-subgraph::SubgraphApiBuilder`, and `cow-sdk-trading::TradingBuilder` construction seams
Refresh trigger: ADR 0038 review confirmed no builder-shape change; future type-parameter or marker visibility changes on any covered builder, a change to the set of required inputs (chain, environment, API key, appCode, or transport), a change to host-policy validation, a change to the native default-transport convenience impl, a change to the wasm32 transport-required or injected-orderbook invariant, or a new `trybuild` witness replacing the current compile-fail coverage
Related docs:
- [ADR 0011](../adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md)
- [ADR 0013](../adr/0013-http-transport-injection-and-typestate-builders.md)
- [ADR 0028](../adr/0028-account-abstraction-integration-plan.md)
- [Alloy Umbrella Adapter Audit](alloy-umbrella-adapter-audit.md)
- [Transport](../transport.md)
- [Architecture](../architecture.md)

## Scope

This audit covers:

- the three-marker `OrderbookApiBuilder` typestate
  (`ChainState`, `EnvironmentState`, `TransportState`) and the single
  `.build()` path
- the three-marker `SubgraphApiBuilder` typestate
  (`ChainState`, `ApiKeyState`, `TransportState`) and the single
  `.build()` path
- the native default-transport convenience on both builders and its
  `#[cfg(not(target_arch = "wasm32"))]` gate
- external host-policy validation for explicit endpoint overrides
- the two-marker `TradingBuilder` typestate
  (`ChainIdState`, `AppCodeState`), validated `AppCode` attribution,
  the distinct `Trading`/`TradingHelpers` terminal types, and the
  documented `wasm32` injected-orderbook runtime terminal
- the native Alloy provider, signer, and umbrella builders that expose
  terminal construction only after their sealed transport, key-source, and
  chain marker states are satisfied
- the sealed, data-carrying marker structs that prevent direct external
  construction of typestate witnesses, proven by a `trybuild` compile-fail
  witness
- the wasm32 transport-required invariant proven by a `trybuild`
  compile-fail witness
- the retirement of the legacy free-function constructors on
  `OrderbookApi` and `SubgraphApi`

It does not cover transport-policy retry, rate-limit, or user-agent
layering (the policy surface sits above the builder and is covered by
a separate contract). Method-specific trading prerequisites are covered by
the trading-sdk runtime prerequisites audit.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| OrderbookApi construction | `OrderbookApi::builder()` is the only production construction path; every required input is encoded as a compile-time marker | Conforms |
| SubgraphApi construction | `SubgraphApi::builder()` is the only production construction path; every required input is encoded as a compile-time marker | Conforms |
| Marker sealing | Public marker types use private tuple fields — the `…Set` markers carry their supplied value in that private field — so external callers cannot construct typestate witnesses directly | Conforms |
| Native convenience | Both builders carry a default-transport `.build()` impl gated on `#[cfg(not(target_arch = "wasm32"))]` that installs a `ReqwestTransport` | Conforms |
| Panic-free terminals | Build terminals read each input from the data-carrying marker and return typed errors; no typestate-guard `expect`/`panic!` remains | Conforms |
| Host policy | Explicit orderbook and subgraph endpoint overrides are validated at build time and fail through typed host-policy errors | Conforms |
| wasm32 invariant | `trybuild` compile-fail coverage asserts `.build()` without `.transport(...)` does not compile on `wasm32` | Conforms |
| Trading SDK construction | `build_ready` requires chain id plus validated `AppCode`, `build_helper_only` requires chain id only, and the terminals return distinct SDK types | Conforms |
| Trading wasm32 posture | `build_ready` documents and enforces the injected orderbook-client requirement at the runtime terminal on `wasm32` | Conforms |
| Native Alloy builders | Provider, signer, and umbrella construction terminals are reachable only after required transport, key-source, and chain marker axes are set | Conforms |

## Current Contract

### OrderbookApi Construction

`OrderbookApiBuilder<ChainState, EnvironmentState, TransportState>`
lives at `crates/orderbook/src/builder.rs`. Each marker transitions
from `…Unset` to `…Set` through the corresponding fluent setter
(`.chain(...)`, `.environment(...)`, `.transport(...)`). The `.build()`
method is implemented only on the fully-set state; attempting to
call it before every marker is set is a compile error. The fluent
layer additionally exposes optional setters for transport policy,
external host policy, shared `reqwest::Client` reuse on native targets,
and per-chain base-URL overrides. The `.base_url(...)` convenience —
which reuses the environment already carried by the `EnvSet` marker — is
implemented only on the `EnvSet` state, so calling it before
`.environment(...)` is a compile error rather than a runtime panic. The
`.build()` method returns a `Result` so explicit endpoint overrides can
fail closed before a client is constructed.

The public marker types are tuple structs with private fields. The
`…Set` markers carry the supplied value — chain id, environment, or the
`Arc<dyn HttpTransport + Send + Sync>` — in that private field, so the
build terminal reads each input directly from the type-level witness
instead of unwrapping an `Option`; the `…Unset` markers carry a private
unit field. The type names remain available in builder type signatures
and diagnostics, but callers outside the defining module cannot construct
either form directly, and the build terminals therefore contain no
typestate-guard panic.

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

### Host-Policy Validation

Both builders default to canonical service hosts and validate explicit
endpoint overrides before returning a client. Callers that need private
mirrors, open routing, or local loopback fixtures must opt in with
`ExternalHostPolicy`. Rejections flow through sanitized
`HostPolicyError` variants rather than panicking or constructing a client
pointed at an unreviewed service host.

### Native Convenience And wasm32 Invariant

On non-`wasm32` targets, a convenience `.build()` impl is defined on the
`(ChainSet, EnvironmentSet | ApiKeySet, TransportUnset)` state and
installs a default `ReqwestTransport`. Constructing that default
transport is fallible: a user-agent that cannot be encoded as an HTTP
header value returns a typed error (`OrderbookError::Transport` for the
orderbook builder, `SubgraphError::TransportConfiguration` for the
subgraph builder) rather than panicking. On `wasm32` this convenience impl
is absent, so a caller must invoke `.transport(...)` explicitly to
reach `.build()`. The `trybuild` UI harness at
`crates/subgraph/tests/ui/builder_wasm32_missing_transport.rs` captures
the expected compile error and its stderr fixture.

### Trading SDK Construction

`TradingBuilder<ChainIdState, AppCodeState>` lives at
`crates/trading/src/sdk/builder.rs`. The fluent chain-id and app-code setters move
the builder from unset to set marker states. `build_ready()` is implemented
only on `(ChainIdSet, AppCodeSet)` and returns `Trading`; `build_helper_only()`
is implemented once `ChainIdSet` is present and returns `TradingHelpers`, which
does not expose quote, post, order lookup, or off-chain cancellation methods.

Trading attribution is validated through `AppCode` before a ready SDK is
returned. The validation deliberately rejects only empty strings, NUL bytes,
and ASCII control characters so source-backed examples such as `CoW Swap`,
`cow-rs/wasm-console`, and `COW_BRIDGING_REACT_EXAMPLE` remain accepted.

On `wasm32`, `build_ready()` keeps the documented runtime terminal posture:
callers must inject an orderbook client with
`TradingOptions::with_orderbook_client(...)`, otherwise the terminal returns
`TradingError::MissingInjectedOrderbookClient`. That avoids adding a third
builder marker while keeping the browser runtime requirement explicit in
rustdoc and regression coverage.

### Legacy Constructor Retirement

The legacy free-function constructors on `OrderbookApi` (`new`,
`new_with_transport_policy`, `from_shared_client`,
`from_shared_client_with_transport_policy`, `new_with_base_url`) and
on `SubgraphApi` (`new`, `with_config`, `with_config_and_transport_policy`,
`from_shared_client`, `from_shared_client_with_config`,
`from_shared_client_with_transport_policy`) have been retired. No
free-function public constructor remains on either type; every caller
in the trading surface, examples workspace, and e2e suite constructs
through the typestate builder.

### Native Alloy Builder Construction

`cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, and `cow-sdk-alloy` each
use sealed marker states to make required construction inputs explicit. The
umbrella `AlloyClientBuilder` combines the native HTTP endpoint, private key,
and chain id into a single terminal `.build()` path. Compile-fail witnesses
cover incomplete states on the public client and signer handle surfaces, while
runtime tests assert chain coherence and non-broadcasting synchronous signing
posture.

## Evidence

Primary implementation points:

- `crates/orderbook/src/builder.rs`
- `crates/orderbook/src/api.rs`
- `crates/subgraph/src/builder.rs`
- `crates/subgraph/src/api.rs`
- `crates/trading/src/sdk/builder.rs`
- `crates/core/src/types/app_code.rs`
- `crates/alloy-provider/src/builder.rs`
- `crates/alloy-signer/src/builder.rs`
- `crates/alloy/src/builder.rs`

Primary regression coverage:

- `crates/orderbook/tests/builder_contract.rs`
- `crates/orderbook/tests/host_policy_contract.rs`
- `crates/subgraph/tests/builder_contract.rs`
- `crates/subgraph/tests/host_policy_contract.rs`
- `crates/subgraph/tests/ui/builder_wasm32_missing_transport.rs`
- `crates/contracts/tests/ui/typestate_marker_sealing.rs`
- `crates/trading/tests/sdk_contract.rs`
- `crates/trading/tests/app_code_contract.rs`
- `crates/trading/tests/ui.rs`
- `crates/alloy/tests/compile_fail.rs`
- `crates/alloy/tests/chain_coherence.rs`
- `crates/alloy/tests/no_broadcast_for_sign_transaction.rs`

Validation surface:

```text
cargo test -p cow-sdk-orderbook --all-features
cargo test -p cow-sdk-subgraph --all-features
cargo test -p cow-sdk-trading
cargo test -p cow-sdk-alloy --all-features
cargo check --workspace --all-features --target wasm32-unknown-unknown
cargo clippy -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-trading -p cow-sdk-alloy --all-targets --all-features -- -D warnings
```
