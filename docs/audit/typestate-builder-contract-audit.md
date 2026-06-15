# Typestate Builder Contract Audit

Status: Current
Last reviewed: 2026-06-14
Owning surface: `cow-sdk-orderbook::OrderbookApiBuilder`, `cow-sdk-subgraph::SubgraphApiBuilder`, and `cow-sdk-trading::TradingBuilder` construction seams, plus the `cow-sdk-trading::SwapBuilder` swap lifecycle seam
Refresh trigger: future type-parameter or marker visibility changes on any covered builder, a change to the set of required inputs (chain, environment, API key, appCode, transport, or the swap sell-token/buy-token/amount markers), a change to host-policy validation, a change to either per-target default-transport `.build()` impl, a change to the trading target-neutral default orderbook factory, a change to the swap lifecycle terminals, or a new `trybuild`/`compile_fail` witness replacing the current compile-fail coverage
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
- the per-target default-transport `.build()` convenience on both builders
  (native `ReqwestTransport`, `wasm32` `FetchTransport`)
- external host-policy validation for explicit endpoint overrides
- the two-marker `TradingBuilder` typestate
  (`ChainIdState`, `AppCodeState`), validated `AppCode` attribution,
  the `Trading` ready terminal type, and the target-neutral default
  orderbook factory inside `build()`
- the three-marker `SwapBuilder` swap lifecycle typestate
  (sell-token, buy-token, and amount markers), its named token setters,
  and the `execute` / `quote` terminals reachable only once all three are set
- the native Alloy provider, signer, and umbrella builders that expose
  terminal construction only after their sealed transport, key-source, and
  chain marker states are satisfied
- the sealed, data-carrying marker structs that prevent direct external
  construction of typestate witnesses, proven by a `trybuild` compile-fail
  witness
- the per-target default-transport `.build()` terminals, type-checked for
  `wasm32` by compiling both builder crates for that target in CI
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
| Default-transport convenience | Both builders carry a default-transport `.build()` impl per target: native installs `ReqwestTransport`, `wasm32` installs the browser `FetchTransport` | Conforms |
| Panic-free terminals | Build terminals read each input from the data-carrying marker and return typed errors; no typestate-guard `expect`/`panic!` remains | Conforms |
| Host policy | Explicit orderbook and subgraph endpoint overrides are validated at build time and fail through typed host-policy errors | Conforms |
| wasm32 default terminal | the `wasm32` default-transport `.build()` constructs `FetchTransport`; both builder crates are compiled for `wasm32` in CI to type-check that terminal and its `cow-sdk-core` `FetchTransport` edge | Conforms |
| Trading SDK construction | `build` requires chain id plus validated `AppCode` and returns the ready `Trading` client | Conforms |
| Trading default terminal | `build` is target-neutral: the lazy default orderbook factory rides the orderbook builder's per-target default transport on native and `wasm32` alike | Conforms |
| Swap lifecycle builder | `Trading::swap` requires sell token, buy token, and amount through named setters before `execute`/`quote` compile; the lifecycle delegates to the existing post entries and adds no protocol logic | Conforms |
| Native Alloy builders | Provider, signer, and umbrella construction terminals are reachable only after required transport, key-source, and chain marker axes are set | Conforms |

## Current Contract

### OrderbookApi Construction

`OrderbookApiBuilder<ChainState, EnvironmentState, TransportState>`
lives at `crates/orderbook/src/builder.rs`. Each marker transitions
from `…Unset` to `…Set` through the corresponding fluent setter
(`.chain(...)`, `.env(...)`, `.transport(...)`). The `.build()`
method is implemented only on the fully-set state; attempting to
call it before every marker is set is a compile error. Three `trybuild`
compile-fail witnesses under `crates/orderbook/tests/ui/` pin this rejection:
reaching `.build()` without a chain id, without an environment, and on an
empty builder each fail with the `no method named build` diagnostic. The fluent
layer additionally exposes optional setters for transport policy,
external host policy, shared `reqwest::Client` reuse on native targets,
and per-chain base-URL overrides. Every construction-builder setter is
named by its bare configuration noun — `chain`, `environment`,
`transport`, `transport_policy`, `api_key`, `external_host_policy`, and
`base_urls` — with no `with_` prefix, matching the standard-library
builder convention (`Command::arg`, `OpenOptions::read`). The `with_`
prefix is reserved for the owned-value setters of parameter and
configuration types whose bare noun is already an accessor. The `.base_url(...)` convenience —
which reuses the environment already carried by the `EnvSet` marker — is
implemented only on the `EnvSet` state, so calling it before
`.env(...)` is a compile error rather than a runtime panic. The
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

Three `trybuild` compile-fail witnesses under `crates/subgraph/tests/ui/` pin
the build rejection: reaching `.build()` without a chain id, without an API key,
and on an empty builder each fail to compile.

### Host-Policy Validation

Both builders default to canonical service hosts and validate explicit
endpoint overrides before returning a client. Callers that need private
mirrors, open routing, or local loopback fixtures must opt in with
`ExternalHostPolicy`. Rejections flow through sanitized
`HostPolicyError` variants rather than panicking or constructing a client
pointed at an unreviewed service host.

### Per-Target Default-Transport Convenience

A convenience `.build()` impl is defined on the
`(ChainSet, EnvironmentSet | ApiKeySet, TransportUnset)` state for every
target. On non-`wasm32` targets it installs a default `ReqwestTransport`;
constructing that default is fallible because a user-agent that cannot be
encoded as an HTTP header value returns a typed error
(`OrderbookError::Transport` for the orderbook builder,
`SubgraphError::TransportConfiguration` for the subgraph builder) rather
than panicking. On `wasm32` the same terminal installs the browser
`FetchTransport` from `cow-sdk-core`, acquired from the realm's
global `fetch`; the policy timeout and response-byte cap apply to either
default, and the browser default omits the user-agent because `User-Agent`
is a forbidden request header for `fetch`. Compiling `cow-sdk-orderbook` and
`cow-sdk-subgraph` for `wasm32` in CI type-checks the browser terminal and
its target-gated `cow-sdk-core` `FetchTransport` dependency edge. Explicit
`.transport(...)` injection remains available on every target for a custom
backend.

### Trading SDK Construction

`TradingBuilder<ChainIdState, AppCodeState>` lives at
`crates/trading/src/client/builder.rs`. The fluent chain-id and app-code setters move
the builder from unset to set marker states. `build()` is implemented
only on `(ChainIdSet, AppCodeSet)` and returns `Trading`. App-code-less helper
flows (allowance, approval, pre-sign, on-chain cancellation) are the crate's
free functions and need no trading client.

Trading attribution is validated through `AppCode` before a ready SDK is
returned. The validation deliberately rejects only empty strings, NUL bytes,
and ASCII control characters so source-backed examples such as `CoW Swap`,
`cow-rs/wasm-console`, and `COW_BRIDGING_REACT_EXAMPLE` remain accepted.

`build()` is target-neutral: with no injected orderbook client, the default
orderbook factory constructs one lazily through `OrderbookApi::builder()` on
native and `wasm32` alike, because the orderbook builder's default-transport
terminal exists on both targets (ADR 0013). An injected client is accepted by
value through `TradingBuilder::orderbook(...)`, which shares it internally as an
`Arc<dyn OrderbookClient>`; the `Arc`-taking `TradingBuilder::orderbook_shared(...)`
variant remains for an already-shared handle.

### Trading Swap Lifecycle Builder

`SwapBuilder<SellToken, BuyToken, AmountState>` lives at
`crates/trading/src/client/swap.rs` and is opened by `Trading::swap()`. The sell
token, buy token, and amount are tracked as the sealed `Set` / `Unset` markers;
each has its own named setter (`sell_token`, `buy_token`, `sell_amount`,
`buy_amount`), so two same-typed token addresses cannot be transposed at the
call boundary. The `execute` and `quote` terminals are implemented only on
`SwapBuilder<Set, Set, Set>`; reaching them before all three required fields
are set is a compile error, pinned by a `compile_fail` doctest on `execute`.
`execute(&signer)` performs the one-call quote-sign-post path and `quote(&signer)`
returns a `QuotedSwap` whose `results()` exposes the quote for inspection before
`submit(&signer)`. The owner is resolved from the signer when no explicit
`owner` is set, so an explicit `owner` is optional and the builder tracks the
three required markers (sell token, buy token, amount). The terminals are
asynchronous, following the crate's runtime-neutral async `Signer` boundary; the
builder delegates to `quote_results`, `post_swap_order`, and
`post_swap_order_from_quote` and adds no protocol logic. The `Set` / `Unset`
markers use private tuple fields, so external callers cannot construct
typestate witnesses directly.

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
umbrella `ClientBuilder` combines the native HTTP endpoint, private key,
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
- `crates/trading/src/client/builder.rs`
- `crates/trading/src/client/swap.rs`
- `crates/core/src/types/app_code.rs`
- `crates/alloy-provider/src/builder.rs`
- `crates/alloy-signer/src/builder.rs`
- `crates/alloy/src/builder.rs`

Primary regression coverage:

- `crates/orderbook/tests/builder_contract.rs`
- `crates/orderbook/tests/ui/build_without_chain.rs`
- `crates/orderbook/tests/ui/build_without_environment.rs`
- `crates/orderbook/tests/ui/build_on_empty_builder.rs`
- `crates/orderbook/tests/host_policy_contract.rs`
- `crates/subgraph/tests/builder_contract.rs`
- `crates/subgraph/tests/ui/build_without_chain.rs`
- `crates/subgraph/tests/ui/build_without_api_key.rs`
- `crates/subgraph/tests/ui/build_on_empty_builder.rs`
- `crates/subgraph/tests/host_policy_contract.rs`
- `crates/contracts/tests/ui/typestate_marker_sealing.rs`
- `crates/trading/tests/sdk_contract.rs`
- `crates/trading/tests/app_code_contract.rs`
- `crates/trading/tests/swap_lifecycle_contract.rs`
- `crates/trading/tests/ui.rs`
- `cargo test -p cow-sdk-trading --doc` (the `SwapBuilder::execute` compile-fail witness)
- `crates/alloy/tests/compile_fail.rs`
- `crates/alloy/tests/chain_coherence.rs`

Validation surface:

```text
cargo test -p cow-sdk-orderbook --all-features
cargo test -p cow-sdk-subgraph --all-features
cargo test -p cow-sdk-trading
cargo test -p cow-sdk-alloy --all-features
cargo check --workspace --all-features --target wasm32-unknown-unknown
cargo clippy -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-trading -p cow-sdk-alloy --all-targets --all-features -- -D warnings
```
