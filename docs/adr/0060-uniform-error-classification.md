---
type: Decision Record
id: ADR-0060
title: "ADR 0060: Uniform error classification through a shared ErrorClass"
description: "The shared ErrorClass enum lives in cow-sdk-core and is re-exported from the cow-sdk facade, and every facade-family error type exposes a const class() accessor, with composites delegating to preserve granularity."
status: Accepted
date: 2026-05-31
last_reviewed: 2026-06-15
authors: ["0xSymbiotic"]
tags: [error-surface, classification, telemetry, ergonomics]
related: [ADR-0053, ADR-0017, ADR-0025]
timestamp: 2026-06-15T00:00:00Z
---

# ADR 0060: Uniform error classification through a shared `ErrorClass`

## Decision

The coarse-grained failure-classification enum `ErrorClass`
(`Validation | Transport | Remote | RateLimited | Signing | Cancelled |
Internal`, `#[non_exhaustive]`) lives in `cow-sdk-core` and is re-exported from
the `cow-sdk` facade so the existing `cow_sdk::ErrorClass` path is unchanged.

Every public error type the facade aggregates exposes a
`const fn class(&self) -> ErrorClass` accessor:
`cow_sdk_core::CoreError`, `cow_sdk_app_data::AppDataError`,
`cow_sdk_signing::SigningError`, `cow_sdk_contracts::ContractsError`,
`cow_sdk_orderbook::OrderbookError`, and `cow_sdk_trading::TradingError`.
Composite error types delegate to
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
`ErrorClass` across the facade error family rather than six near-duplicate
per-type enums:

- The facade family (`CoreError`, `AppDataError`, `SigningError`,
  `ContractsError`, `OrderbookError`, `TradingError`)
  classifies into a **single shared taxonomy** — the same seven buckets the
  facade already unified through `CowError::class()`. A per-type enum per crate
  would reproduce that one taxonomy six times, and `TradingErrorClass` would
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
- Retry-decision accessors: `OrderbookError` exposes `is_retryable(&self) -> bool`
  (keyed off the retained HTTP status — the same verdict the transport retry loop
  applies, so a non-retryable `400` and a retryable `503` are distinguished where
  the coarse `Remote` class cannot) and `backoff_hint(&self) -> Option<Duration>`
  (the parsed `Retry-After`, resolved against the wasm-safe clock); `TradingError`
  and `CowError` delegate to the wrapped orderbook error and the wasm `WasmError`
  projects `retryable` / `retryAfterMs`, alongside the specific `errorType` wire
  tag (the fine-grained partner of the coarse `OrderBookRejectionCategory`).
- Subgraph eighth member: under the off-by-default `subgraph` feature,
  `cow_sdk_subgraph::SubgraphError` joins the family with its own `class()` and a
  feature-gated `CowError::Subgraph` variant that delegates; the retry-decision
  accessors stay orderbook/trading-scoped (a subgraph error reports non-retryable
  with no hint).
- Cost: six small `class()` accessors plus the relocation of one public enum
  from the facade to core (re-exported for source compatibility). The facade's
  private `classify_*` functions are removed; the public surface
  (`cow_sdk::ErrorClass`, `CowError::class()`) is unchanged.

## Alternatives Rejected

- Six per-type class enums for the facade error family (`OrderbookErrorClass`,
  `TradingErrorClass`, and so on): rejected. The family classifies into a single
  shared taxonomy, so per-type enums would reproduce that one taxonomy six
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

- [Principles](../principles/index.md)
- [Shared `ErrorClass` definition](../../crates/core/src/errors.rs)
- [Facade error aggregation and re-export](../../crates/sdk/src/lib.rs)
- [ADR 0053](0053-typed-signer-rejection-classification.md)
- [ADR 0017](0017-typed-orderbook-rejection-parser.md)
- [ADR 0025](0025-workspace-url-redaction-convention.md)

**Proven by:**

- [Error Classification Audit](../audit/error-classification-audit.md)
