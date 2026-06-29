---
type: Audit
id: bounded-response-reads
title: "Bounded Response Reads Audit"
description: "Every HTTP response body the SDK buffers is bounded by a configurable per-client byte limit measured on decoded bytes, on both the success and error paths, with a non-retryable over-limit outcome."
status: Current
owning_surface: "HTTP response reads across core, wasm, and the signature-decode path"
related: [ADR-0055]
timestamp: 2026-06-20
---

# Bounded Response Reads Audit

## Scope

Reviews the byte bound on buffered HTTP response bodies across the native and
browser transports and the signature hex pre-decode bound: the per-client
defaults, the decoded-bytes measurement, and the non-retryable over-limit
classification. It does not cover transport retry orchestration (the HTTP
Transport Contract Audit) or time bounds.

## Findings

- The native transport streams chunks under a configurable limit and refuses an
  over-limit body after at most one over-limit chunk, on both the success and
  non-2xx error paths.
- Where gzip is enabled (the orderbook and trading clients), the bound is
  measured on decoded bytes, so a decompression bomb is rejected on its
  decompressed size; a core-only build without gzip bounds the compressed size.
- The browser `FetchTransport` and the JS-callback transport apply the same
  limit as a post-receipt bound on the JS-materialized body, with the residual
  documented.
- Per-client defaults are tuned to the surface — generous for orderbook and
  trading, tighter for the untrusted subgraph gateway, and sized to the app-data
  protocol limit for IPFS.
- The over-limit outcome is a deterministic, non-retryable
  `TransportErrorClass::ResponseTooLarge`, and signature hex is length-bounded
  before decode allocates.

## Evidence

- Decision: [ADR 0055](../adr/0055-bounded-response-reads.md).
- Invariants: the `PROP-TPP` ([transport policy](../properties/transport-policy.md)) and `PROP-SEC` ([security](../properties/security.md)) families.
- Governing gate: the gzip-bomb / `ResponseTooLarge` transport-contract test in `crates/core`.
- Code: `crates/core/src/transport/`, `crates/contracts/src/` (signature hex bound), `crates/js/src/exports/`.
