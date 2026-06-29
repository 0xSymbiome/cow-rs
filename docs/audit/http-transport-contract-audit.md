---
type: Audit
id: http-transport-contract
title: "HTTP Transport Contract Audit"
description: "The HttpTransport trait is the sole production HTTP dispatch seam for orderbook and subgraph, with dyn-compatible adapters, typed errors, URL redaction, and shared retry orchestration."
status: Current
owning_surface: "the HttpTransport trait, its adapters, and the shared transport policy"
related: [ADR-0013, ADR-0041]
timestamp: 2026-06-20
---

# HTTP Transport Contract Audit

## Scope

Reviews the `cow-sdk-core::HttpTransport` trait, its native `ReqwestTransport`
and browser `FetchTransport` adapters, and the shared transport policy: the
trait shape and dyn-compatibility, `TransportResponse`, the typed errors, the
per-call controls, URL redaction, the sole-dispatch contract, and the
retry/jitter/cooldown surface. It does not cover the response byte bound (the
Bounded Response Reads Audit) or credential storage (the Credential Redaction
Audit).

## Findings

- `HttpTransport` is the sole production HTTP injection point and is
  dyn-compatible, so every live REST or GraphQL call from orderbook and subgraph
  routes through the injected transport.
- The success channel returns `TransportResponse` carrying the real 2xx status,
  redacted headers, and body, so the calling layer reads true metadata rather
  than a fabricated result.
- Every method carries per-call headers and an optional timeout, merged with the
  adapter defaults and applied as a deadline when supplied.
- Failures route through typed `Transport`, `Configuration`, and `HttpStatus`
  variants, and both default adapters strip the URL before wrapping so a
  credential-bearing query string never surfaces through `Debug` or `Display`.
- The native and browser adapters report the same `TransportErrorClass` for
  matching failure classes, and a non-2xx response surfaces through `HttpStatus`
  with the numeric status preserved.
- The retry driver honors `Retry-After` on 429/503 — waiting the larger of the
  jittered local backoff and the server cooldown, against a wasm-safe clock — and
  emits retry telemetry.

## Evidence

- Decision: [ADR 0013](../adr/0013-http-transport-injection-and-typestate-builders.md), [ADR 0041](../adr/0041-transport-policy-l3-layering.md).
- Invariants: the `PROP-CORE` ([core](../properties/core.md)), `PROP-ORD` ([orderbook](../properties/orderbook.md)), and `PROP-TPP` ([transport policy](../properties/transport-policy.md)) families.
- Governing gate: the sole-dispatch test in `crates/orderbook/tests/api_contract.rs`.
- Code: `crates/core/src/transport/http.rs`, `crates/core/src/transport/policy/`, `crates/orderbook/src/api.rs`, `crates/subgraph/src/api.rs`.
