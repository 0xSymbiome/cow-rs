---
type: Decision Record
id: ADR-0010
title: "ADR 0010: Runtime-Neutral Async And Transport Posture"
description: "The public async surface stays runtime-neutral."
status: Accepted
date: 2026-04-17
last_reviewed: 2026-06-15
authors: ["0xSymbiotic"]
tags: [async, cancellation, transport, observability, error-model]
related: [ADR-0005, ADR-0006, ADR-0013, ADR-0039, ADR-0040]
timestamp: 2026-06-15T00:00:00Z
---

# ADR 0010: Runtime-Neutral Async And Transport Posture

## Decision

The public async surface stays runtime-neutral. Long-running operations accept
cancellation through `cow_sdk_core::Cancellable::cancel_with(&token)`, the
`HttpTransport` trait remains the production HTTP seam, and `tracing`
instrumentation stays opt-in.

The runtime-neutral transport posture supports three
`cow_sdk_core::HttpTransport` implementations, two of which ship inside
`cow-sdk-core` under target cfgs: `ReqwestTransport` for native targets and
`FetchTransport` (browser `fetch`) for `wasm32-unknown-unknown`. The third,
`cow_sdk_js::exports::JsCallbackHttpTransport`, serves runtime-neutral JS
consumers such as Node, Workers, and Deno. Both default transports stay in
`cow-sdk-core`, each gated to its target; the workspace does not extract a
separate per-target transport crate. The browser `FetchTransport` is the
`wasm32` sibling of the native `ReqwestTransport`, gated to
`cfg(all(target_arch = "wasm32", target_os = "unknown"))` so WASI builds stay
free of the browser-global dependency stack.

The JS callback transport enforces SDK-owned request timeout with
`globalThis.AbortController`. Its `TimerGuard` owns both the opaque timer
handle and the `Closure<dyn FnMut()>`, so cleanup happens on every return
path. The same cancellation and transport contract extends to
`cow_sdk_app_data::IpfsFetchTransport`: the trait is async and uses the
dual-gate `async_trait(?Send)` on wasm32 and `async_trait` on native targets.

Wire-format envelopes for the wasm surface use a string `schemaVersion` field
such as `"v1"`, not a numeric one. The Rust-side `#[non_exhaustive]`
`SchemaVersion` enum serializes and deserializes as a string via serde `rename`
attributes, avoiding JSON numeric-precision risks across future schema
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
- Async-trait ergonomics: the seam-owning crates (`cow-sdk-trading`,
  `cow-sdk-signing`) re-export `async_trait`, so implementors of their
  `Arc<dyn …>` seam traits add no direct `async-trait` dependency; native-only
  implementors use plain `#[async_trait]`, and the dual-gate `cfg_attr` pair is
  needed only for code compiled for both native and `wasm32`.
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

- [Architecture](../guides/architecture.md)
- [Transport](../guides/transport.md)
- [Observability](../guides/observability.md)
- [Performance](../guides/performance.md)
- [Verification Guide](../guides/verification.md)
- [ADR 0013](0013-http-transport-injection-and-typestate-builders.md)
- See also: ADR 0024, ADR 0030, ADR 0039, ADR 0040, and ADR 0041.

**Proven by:**

- [Credential Redaction Audit](../audit/credential-redaction-audit.md)
