---
type: Audit
id: credential-redaction
title: "Credential Redaction Audit"
description: "Every credential-bearing field across config, transport, RPC, orderbook, subgraph, app-data, native Alloy, and the wasm error envelope renders only sanitized identity."
status: Current
owning_surface: "cross-cutting credential redaction"
related: [ADR-0005, ADR-0006, ADR-0010, ADR-0025]
timestamp: 2026-06-21
---

# Credential Redaction Audit

## Scope

Reviews credential redaction wherever a secret could reach public output:
builder and config storage, URL-bearing config and its dispatch seams, subgraph
production routing and its `Authorization` header, the `Redacted<T>` newtype and
the URL-map wrappers, every public error family (provider, signer, RPC,
transport, response-body, orderbook-rejection, subgraph-context, caller-input),
the native Alloy adapters' opaque `Debug`, the `redact_response_body` scanner,
and the `WasmError` envelope. It does not cover transport-policy behavior (the
HTTP Transport Contract Audit) or the typed parsing of the GraphQL envelope.

## Findings

- Credential-bearing fields are stored in `Redacted<T>` (or a typestate that
  never stores the secret), so `Debug` / `Display` / `Serialize` emit
  `[redacted]` by construction; raw bytes are reached only through explicit
  accessors at dispatch seams.
- URL maps (`RedactedUrlMap`, `RedactedOptionalUrlMap`) redact values while
  keeping chain-id keys and unsupported-chain `None` markers visible.
- Subgraph production routing carries the partner key in the `Authorization`
  header against the key-free gateway URL, so no stored URL, request path, or
  telemetry span endpoint ever holds the key.
- Every reviewed error family keeps safe diagnostics visible (chain ids, status
  codes, field names, sanitized rejection tags) while wrapping or typing away
  free-form and credential-bearing payloads; decode failures surface only the
  serde category and 1-based line/column.
- Host-policy rejections retain only a redacted host or a sanitized failure
  class — never raw URL credentials, paths, queries, or fragments.
- The `redact_response_body` scanner runs an ordered JWT → Bearer → strict-URL →
  bare-userinfo → credential-keyed-value detector with recursive key-prefix
  scanning, so scheme-mangled or embedded credentials cannot ship verbatim.
- The wasm `WasmError` re-redacts response bodies through `redact_response_body`
  and never hands a `Redacted<T>` secret to JavaScript.

## Evidence

- Decision: [ADR 0025](../adr/0025-workspace-url-redaction-convention.md), [ADR 0005](../adr/0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md), [ADR 0010](../adr/0010-runtime-neutral-async-and-transport-posture.md).
- Rule: [Credential Redaction by Construction](../principles/credential-redaction-by-construction.md).
- Invariants: the `PROP-SEC` family ([security](../properties/security.md)), whose evidence columns carry the regression set.
- Governing gate: `crates/sdk/tests/error_redaction_contract.rs` + `cargo check-wasm-invariant`.
- Code: `crates/core/src/redaction/`, `crates/core/src/config/hosts.rs`, the per-crate `error.rs` files, `crates/js/src/exports/errors.rs`.
