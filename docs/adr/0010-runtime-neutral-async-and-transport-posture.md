# ADR 0010: Runtime-Neutral Async And Transport Posture

- Status: Accepted (amended)
- Date: 2026-04-17
- Last reviewed: 2026-06-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: async, cancellation, transport, observability, error-model
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md), [ADR 0013](0013-http-transport-injection-and-typestate-builders.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0040](0040-wallet-provider-callback-boundary-for-js-consumers.md)
- Superseded in part by: [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)

## Decision

The public async surface stays runtime-neutral. Long-running operations accept
cancellation through `cow_sdk_core::Cancellable::cancel_with(&token)`, the
`HttpTransport` trait remains the production HTTP seam, and `tracing`
instrumentation stays opt-in.

The runtime-neutral transport posture supports three
`cow_sdk_core::HttpTransport` implementations: `ReqwestTransport` for native
targets, target-gated inside `cow-sdk-core`; `cow_sdk_transport_wasm::FetchTransport`
for browser `fetch`; and `cow_sdk_wasm::exports::JsCallbackHttpTransport` for
runtime-neutral JS consumers such as Node, Workers, and Deno. reqwest stays in
`cow-sdk-core`, target-gated; the workspace does not extract a separate
native-reqwest transport crate.

The JS callback transport enforces SDK-owned request timeout with
`globalThis.AbortController`. Its `TimerGuard` owns both the opaque timer
handle and the `Closure<dyn FnMut()>`, so cleanup happens on every return
path. The same cancellation and transport contract extends to
`cow_sdk_app_data::IpfsFetchTransport`: the trait is async and uses the
dual-gate `async_trait(?Send)` on wasm32 and `async_trait` on native targets.

Wire-format envelopes for the wasm surface use a string `schemaVersion` field
such as `"1"`, not a numeric one. The Rust-side `#[non_exhaustive]`
`SchemaVersion` enum serializes and deserializes through custom impls that emit
and parse strings, avoiding JSON numeric-precision risks across future schema
evolutions.

## Why

Consumers embed the SDK inside bots, analytics systems, browser apps, and
JavaScript runtimes that already own their event loop, telemetry subscriber,
and error routing. A fixed runtime, implicit background tasks, leaked
credential-bearing URLs, or one hardcoded HTTP client would make the SDK harder
to compose and review.

## Must Remain True

- Public surface: each long-running public method uses the canonical
  cancellation combinator and returns the crate-level `Cancelled` variant on
  cancellation.
- Runtime and support: library code does not call `tokio::spawn`, does not
  require `rt-multi-thread`, and does not use `#[tokio::main]`.
- Validation and review: reqwest error conversions classify through upstream
  predicates and call `without_url()` before wrapping; `tracing` fields never
  carry secrets.
- Cost: the shared `CancellationToken`, `Cancellable` combinator, target-gated
  transport adapters, and string schema versioning add small surface area to
  preserve runtime neutrality.

## Alternatives Rejected

- Spawn tasks eagerly and broadcast shutdown internally: forces a runtime
  contract on library consumers.
- Expose `reqwest::Client` as a required constructor argument: breaks the
  default path and does not serve wasm consumers.
- Encode schema versions as JSON numbers: smaller, but less stable across
  JavaScript number handling and future version shapes.

## Links

- [Architecture](../architecture.md)
- [Transport](../transport.md)
- [Observability](../observability.md)
- [Performance](../performance.md)
- [Verification Guide](../verification.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)
- See also: ADR 0024, ADR 0029, ADR 0030, ADR 0039, ADR 0040, and ADR 0041.

**Proven by:**

- [Cooperative Cancellation Contract Audit](../audit/cooperative-cancellation-contract-audit.md)
- [Credential Surface Contract Hygiene Audit](../audit/credential-surface-contract-hygiene-audit.md)

## Amendment 2026-06-11: seam-owning crates re-export `async_trait`

`cow-sdk-trading` and `cow-sdk-signing` — the crates owning the `Arc<dyn …>`
seam traits (`SlippageSuggester`, `EthFlowOrderExistsChecker`, `Eip1271Signer`)
— re-export `async_trait::async_trait`, so implementors add no direct
`async-trait` dependency. Native-only implementors use the plain
`#[async_trait]` attribute; the dual-gate `cfg_attr` pair recorded above is
required only for code that compiles for both native and wasm32 targets.
