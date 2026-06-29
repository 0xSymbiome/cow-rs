---
type: Audit
id: error-classification
title: "Error Classification Audit"
description: "Every facade-family error type exposes a const class() accessor over the shared ErrorClass, plus orderbook retry-decision accessors, with composites delegating to preserve granularity."
status: Current
owning_surface: "the class(), is_retryable(), and backoff_hint() accessors and the shared ErrorClass"
related: [ADR-0060]
timestamp: 2026-06-21
---

# Error Classification Audit

## Scope

Reviews the shared `cow_sdk_core::ErrorClass` partition, the per-type
`class()` accessors on the facade error family, the orderbook
`is_retryable()` / `backoff_hint()` retry-decision accessors, and the
signer-rejection routing. It does not cover the native Alloy adapters' own class
enums (the Alloy Adapters Audit) or credential redaction within errors (the
Credential Redaction Audit).

## Findings

- `ErrorClass` (the seven `#[non_exhaustive]` buckets) lives in `cow-sdk-core`
  and is re-exported as `cow_sdk::ErrorClass`, so the path is unchanged.
- Every facade-family error type exposes a `const fn class()`, and
  `CowError::class()` delegates to them holding no classification logic of its
  own.
- Composite errors delegate to the wrapped error's `class()`, so a wrapped 429
  orderbook rejection stays `RateLimited` rather than collapsing to a coarse
  bucket.
- `OrderbookError` exposes `is_retryable()` (keyed off the retained HTTP status,
  distinguishing a non-retryable 400 from a retryable 503) and `backoff_hint()`
  (the parsed `Retry-After`); `TradingError` and `CowError` delegate.
- The classification reads only typed discriminants and never renders
  credential-bearing content; the native Alloy adapters keep their own per-type
  class enums because their taxonomies genuinely differ.

## Evidence

- Decision: [ADR 0060](../adr/0060-uniform-error-classification.md).
- Invariants: the `PROP-ORD` family ([orderbook](../properties/orderbook.md)).
- Governing gate: the facade error-class contract test in `crates/sdk/tests/`.
- Code: `crates/core/src/errors.rs`, `crates/orderbook/src/error.rs`, `crates/trading/src/error.rs`, `crates/sdk/src/lib.rs`.
