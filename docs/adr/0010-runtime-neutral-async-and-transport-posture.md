# ADR 0010: Runtime-Neutral Async And Transport Posture

- Status: Accepted
- Date: 2026-04-17
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: async, cancellation, transport, observability, error-model
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)

## Decision

The public `cow-rs` async surface stays runtime-neutral. Long-running
operations accept cancellation through a re-exported
`tokio_util::sync::CancellationToken`, transport clients expose a shared-
client pattern for multi-service consumers, `reqwest::Error` conversions
classify failures exhaustively and strip the URL before wrapping, and
`tracing` instrumentation is an opt-in feature with a documented field
registry and a classification helper on the facade error.

## Why

A protocol SDK is consumed inside bots, MEV searchers, analytics pipelines,
and browser apps. Each embeds its own runtime, telemetry subscriber, and
error-routing policy. If the SDK forces a fixed runtime, spawns background
tasks without consent, leaks credential-bearing URLs through default error
output, or hardcodes an HTTP client per service, downstream callers either
fight those defaults or avoid the SDK. Keeping the async surface neutral,
cooperative, and redaction-safe preserves the library posture and lets
consumers plug the SDK into any async ecosystem they already run.

## Must Remain True

- Public surface: the cancellation-aware surface on `OrderBookApi`,
  `SubgraphApi`, and `TradingSdk` is expressed through the
  `cow_sdk_core::Cancellable::cancel_with(&token)` extension-trait
  combinator. Every public async method carries one canonical shape, and
  cancellation composes through the combinator at the call site, returning
  the crate-level `Cancelled` variant on every affected error aggregate.
  `SdkError::class()` returns `ErrorClass::Cancelled` for every such
  variant. `OrderBookApi` and `SubgraphApi` expose `from_shared_client`
  constructors plus a transport-policy variant so consumers can pool one
  `reqwest::Client` across chains and services. Any new long-running
  public method lands under the canonical shape.
- Runtime and support: the SDK does not call `tokio::spawn` from library
  code, does not require `rt-multi-thread`, and does not use
  `#[tokio::main]` anywhere in library sources. The combinator runs a
  biased poll against the borrowed token and drops the inner future the
  moment the token fires, releasing the underlying socket promptly.
  `std::sync::Mutex` (or `parking_lot::Mutex`) is the default lock for user
  data; `tokio::sync::Mutex` is reserved for I/O resources held across
  `.await` points.
- Validation and review: `From<reqwest::Error>` conversions on every
  transport surface classify via the upstream `is_timeout`, `is_connect`,
  `is_decode`, `is_body`, `is_redirect`, `is_builder`, `is_request`, and
  `is_status` checks and call `without_url()` before wrapping, so credential-
  bearing URLs cannot leak through error `Display`. The `tracing` feature
  stays per-crate optional and zero-cost when disabled; the facade
  `cow-sdk/tracing` feature activates the leaves in one step.
- Cost: one `Cancellable` extension trait and a small `tokio-util`
  dependency pulled in for its shared `CancellationToken`. The `tracing`
  feature lights a documented field registry that must not carry secret
  values.

## Alternatives Rejected

- Spawn tasks eagerly and broadcast shutdown internally: matches some
  platform SDKs, but contradicts the library posture and forces a runtime
  contract on consumers who already own their event loop.
- Expose `reqwest::Client` as a required constructor argument: simpler, but
  breaks the default ergonomic path for single-chain consumers and forces
  every caller to own the transport builder.
- Stringly-typed error classification on the facade aggregate: easier to
  grow, but forces every downstream telemetry layer to pattern-match on
  variant shapes instead of partition classes.
- Per-method cancellation siblings (a `_with_cancellation` variant on
  every operation): rejected as an API-surface-doubling pattern. The extension-trait combinator delivers
  the same typed-error semantics at one-to-many-times lower surface cost.

## Links

- [Architecture](../architecture.md)
- [Observability](../observability.md)
- [Performance](../performance.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)
- [ADR 0006](0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md)

**Proven by:**

- [Cooperative Cancellation Contract Audit](../audit/cooperative-cancellation-contract-audit.md)
- [Credential Surface Contract Hygiene Audit](../audit/credential-surface-contract-hygiene-audit.md)
