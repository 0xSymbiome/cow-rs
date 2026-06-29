---
type: Principle
title: "Cooperative Cancellation Coverage"
description: "Every long-running async public method composes with Cancellable::cancel_with(&token); the caller owns the token."
tags: [principle]
timestamp: 2026-06-29T00:00:00Z
anchored_by: [ADR-0010, ADR-0006]
shape: rule
enforced_by: "crates/core/tests/cancellation_coverage_validator.rs"
---

# Cooperative Cancellation Coverage

**Invariant** — Every long-running async public method on `OrderbookApi`, `SubgraphApi`,
`Trading`, or any future client is composable with `cow_sdk_core::Cancellable::cancel_with(&token)`.
The error aggregate of every public API lifts `Cancelled` through `From`. Cancellation is
cooperative: the caller owns the token and the SDK installs no hidden global cancellation state.

**Why** — Without uniform cooperative cancellation a caller cannot bound a hung request; a
runtime-wide cancel switch would itself be the hidden global state this SDK refuses to keep.

**How to comply**
- Route a new long-running async method through the `Cancellable` seam and add its row to the
  crate's cancellation-composition table.
- Give the crate's public error enum a `Cancelled` variant reachable via `From`.

**Enforced by** — `crates/core/tests/cancellation_coverage_validator.rs` discovers every public
async method by parsing the client sources and fails closed if one lacks a cancellation-coverage
row (and on stale rows), backed by each crate's `cancellation_composition_contract.rs`.

**Anchored by**: [ADR 0010](../adr/0010-runtime-neutral-async-and-transport-posture.md) (primary). Supporting: [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md).
