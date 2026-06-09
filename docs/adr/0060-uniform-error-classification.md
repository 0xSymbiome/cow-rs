# ADR 0060: Uniform error classification through a shared `ErrorClass`

- Status: Accepted
- Date: 2026-05-31
- Last reviewed: 2026-05-31
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: error-surface, classification, telemetry, ergonomics
- Anchors: Strong Typed Public Surfaces (supporting)
- Related: [ADR 0053](0053-typed-signer-rejection-classification.md), [ADR 0017](0017-typed-orderbook-rejection-parser.md), [ADR 0025](0025-workspace-url-redaction-convention.md)

## Decision

The coarse-grained failure-classification enum `ErrorClass`
(`Validation | Transport | Remote | RateLimited | Signing | Cancelled |
Internal`, `#[non_exhaustive]`) lives in `cow-sdk-core` and is re-exported from
the `cow-sdk` facade so the existing `cow_sdk::ErrorClass` path is unchanged.

Every public error type the facade aggregates exposes a
`const fn class(&self) -> ErrorClass` accessor:
`cow_sdk_core::CoreError`, `cow_sdk_app_data::AppDataError`,
`cow_sdk_signing::SigningError`, `cow_sdk_contracts::ContractsError`,
`cow_sdk_orderbook::OrderbookError`, `cow_sdk_trading::TradingError`, and
`cow_sdk_browser_wallet::BrowserWalletError`. Composite error types delegate to
the wrapped error's `class()` so granularity is preserved (a wrapped 429
orderbook rejection stays `RateLimited` rather than collapsing to a coarse
bucket). The facade `CowError::class()` delegates to the per-type accessors and
holds no classification logic of its own.

## Why

A consuming application that handles a bare `OrderbookError` or `TradingError`
(rather than the facade `CowError`) previously had no way to obtain the
coarse class without re-implementing the per-variant match locally, because the
classification lived only in private functions inside the facade crate. Moving
it onto each error type removes that duplication and lets retry and telemetry
layers partition any workspace error uniformly.

[ADR 0053](0053-typed-signer-rejection-classification.md) records the existing
convention that "every error type owns its own `class()` accessor returning a
type-specific enum" and notes that departing from it "requires a specific
justification." This ADR is that justification for using **one shared**
`ErrorClass` across the facade error family rather than seven near-duplicate
per-type enums:

- The facade family (`CoreError`, `AppDataError`, `SigningError`,
  `ContractsError`, `OrderbookError`, `TradingError`, `BrowserWalletError`)
  classifies into a **single shared taxonomy** — the same seven buckets the
  facade already unified through `CowError::class()`. A per-type enum per crate
  would reproduce that one taxonomy seven times, and `TradingErrorClass` would
  be a verbatim copy of `ErrorClass`.
- `TradingError` is a **composite** over the rest of the family; a shared return
  type lets it delegate to the inner accessors directly without a mapping
  cascade.

The native Alloy adapter crates keep their **own** per-type class enums
(`cow_sdk_alloy_provider::ProviderErrorClass`,
`cow_sdk_alloy_signer::SignerErrorClass`,
`cow_sdk_alloy::AlloyClientErrorClass`) under the ADR 0053 convention, because
their taxonomies genuinely differ from each other and from the facade family
(for example the signer's six signing-specific classes). They are not migrated
to the shared enum.

## Must Remain True

- `ErrorClass` stays `#[non_exhaustive]`; new buckets are additive.
- Every facade-family error type exposes `class(&self) -> ErrorClass`, and
  `CowError::class()` delegates rather than re-deriving.
- Composite error types delegate to inner `class()` so wrapped granularity
  (notably the 429 → `RateLimited` orderbook path) is preserved.
- The classification reads only typed discriminants; it never renders
  credential-bearing content, consistent with ADR 0025.
- Retry policies treat `Transport` and `Remote` (and `RateLimited` only after
  the transport retry budget is honored) as the retryable classes.
- Cost: seven small `class()` accessors plus the relocation of one public enum
  from the facade to core (re-exported for source compatibility). The facade's
  private `classify_*` functions are removed; the public surface
  (`cow_sdk::ErrorClass`, `CowError::class()`) is unchanged.

## Alternatives Rejected

- Seven per-type class enums for the facade error family (`OrderbookErrorClass`,
  `TradingErrorClass`, and so on): rejected. The family classifies into a single
  shared taxonomy, so per-type enums would reproduce that one taxonomy seven
  times, `TradingErrorClass` would be a verbatim copy of `ErrorClass`, and the
  composite `TradingError` could not delegate to its inner accessors without a
  mapping cascade.
- Keep the classification private inside the facade crate (the prior state):
  rejected. A consumer holding a bare `OrderbookError` or `TradingError` would
  then have no way to obtain the coarse class without re-implementing the
  per-variant match locally.
- Migrate the native Alloy adapter crates to the shared enum: rejected. Their
  taxonomies genuinely differ from the facade family and from each other (for
  example the signer's six signing-specific classes), so they keep their own
  per-type class enums under the [ADR 0053](0053-typed-signer-rejection-classification.md)
  convention.

## Links

- [Principles](../principles.md)
- [Shared `ErrorClass` definition](../../crates/core/src/errors.rs)
- [Facade error aggregation and re-export](../../crates/sdk/src/lib.rs)
- [ADR 0053](0053-typed-signer-rejection-classification.md)
- [ADR 0017](0017-typed-orderbook-rejection-parser.md)
- [ADR 0025](0025-workspace-url-redaction-convention.md)

## Amendment 2026-06-06: orderbook retry-decision accessors

`OrderbookError` exposes two retry-decision accessors alongside `class()`:

- `is_retryable(&self) -> bool` returns whether retrying the same request may
  succeed. A structured non-2xx response keys off the retained HTTP status
  through `cow_sdk_core::transport::policy::is_retryable_status` (the `408`, `425`,
  `429`, `500`, `502`, `503`, `504` set); a transport failure keys off its
  `TransportErrorClass` through the shared `RetryPolicy::should_retry_network`
  mapping. This is the same verdict the SDK transport retry loop applies, so a
  consumer that drives its own retry loop over a returned error does not
  re-derive the retryable-status set.
- `backoff_hint(&self) -> Option<Duration>` returns the server-suggested wait
  parsed from the failing response's `Retry-After` header (RFC 7231
  delta-seconds or HTTP-date), resolved against the wasm-safe wall clock when
  the error is constructed; an HTTP-date in the past resolves to
  `Duration::ZERO`. It is `None` for transport failures and for responses
  without a `Retry-After` header.

`is_retryable` keys off the retained status rather than `class()` because the
coarse partition collapses every non-429 remote response into
`ErrorClass::Remote`, so a retryable `503` and a non-retryable `400` are
indistinguishable at the class level; the status-precise accessor separates
them. `class()` stays the coarse telemetry bucket and is unchanged.

`OrderbookApiError` carries the parsed `Retry-After` so both the `Rejected` and
`Api` promotion paths expose it. The value is resolved through
`cow_sdk_core::transport::policy::retry_after_from_headers` while the response headers
are in scope, then attached to the error; the transport retry loop computes its
own clock-injected backoff and does not depend on the stored value.

`TradingError` and the facade `CowError` delegate both accessors to the wrapped
orderbook error and return `false` / `None` for every non-orderbook variant,
mirroring the `class()` delegation so the verdict is identical whether a caller
holds the facade error or a bare leaf error.

The accessor names follow the established Rust transport-error convention — an
`is_retryable` predicate plus a backoff hint, as on `alloy`'s transport error —
rather than a bespoke name. Anchored additionally by
[ADR 0041](0041-transport-policy-l3-layering.md) (transport-policy layering) and
[ADR 0010](0010-runtime-neutral-async-and-transport-posture.md) (runtime-neutral
transport).

The TypeScript-callable `cow-sdk-wasm` surface projects the same verdict to
JavaScript: the `WasmError` `orderbook` variant carries a `retryable` boolean
(always serialised) and an optional `retryAfterMs`, populated from these
accessors so a JavaScript consumer reaches the identical decision without
re-deriving the retryable-status set.

## Amendment 2026-06-07: subgraph joins the family behind the `subgraph` feature

When the `cow-sdk` `subgraph` feature is enabled, the read-only subgraph surface
is lifted into the facade ([ADR 0003](0003-separate-read-only-subgraph-crate.md)).
`cow_sdk_subgraph::SubgraphError` then joins the shared classification family as
an eighth member: it exposes `const fn class(&self) -> ErrorClass`, and
`CowError` gains a feature-gated `Subgraph` variant whose `class()` delegates to
it, exactly like the other facade-aggregated leaf errors.

The mapping follows the established convention: an HTTP `429` that outlived the
transport retry budget is `RateLimited`; other non-success statuses and GraphQL
error payloads are `Remote`; transport failures are `Transport`; an
unsupported-network selection is caller-side `Validation`; cancellation is
`Cancelled`; and transport-construction, host-policy, serialization,
empty-totals, and missing-data faults are `Internal`.

The `subgraph` feature is off by default, so the default facade family is
unchanged; the eighth member appears only when a consumer opts into subgraph.
The retry-decision accessors (`is_retryable` / `backoff_hint`) stay
orderbook-and-trading-scoped: `CowError` reports a subgraph error as
non-retryable with no backoff hint until those accessors are extended to the
subgraph surface.
