---
type: Audit
id: panic-free-public-surface
title: "Panic-Free Public Surface Audit"
description: "Every published-API-reachable runtime path is panic-free except a small allowlist of documented static-invariant sites that a gate enforces."
status: Current
owning_surface: "public runtime surface under crates/*/src reachable from published crates"
related: [ADR-0033]
timestamp: 2026-06-20
---

# Panic-Free Public Surface Audit

## Scope

Reviews every `crates/*/src/**` path reachable from the published public API for
`panic!`, `unwrap`, `expect`, and panic-capable arithmetic. It does not cover
test or example code, or panics behind `#[cfg(test)]`.

## Findings

- No published-API-reachable runtime path panics outside the reviewed
  allowlist; the remaining sites are limited to static literals, registry
  lookups guarded by deployment-resolution tests, owned-value serialization, and
  clamped numeric conversions.
- The allowlist is owned by `.github/config/panic-allowlist.yaml` and enforced
  by a gate, so this audit does not re-enumerate it; an unlisted panic site, or a
  listed item missing its `# Panics` rustdoc and in-body `// SAFETY:` rationale,
  fails the gate.
- The typestate trading terminals (`SwapBuilder` / `LimitBuilder`) read their
  invariants from marker types and return typed errors at construction rather
  than asserting on the path.
- The WASM exports return `Result<_, JsValue>` (or plain values) and install the
  panic hook once at init, so an unexpected panic surfaces as a catchable error
  rather than an aborted module.

## Evidence

- Decision: [ADR 0033](../adr/0033-minimum-viable-panic-surface.md).
- Rule: [Minimum-Viable Panic Surface](../principles/minimum-viable-panic-surface.md).
- Registered as evidence for `PROP-AUD-001` ([documentation governance](../properties/docs.md)).
- Governing gate: `cargo check-panic-allowlist` (`xtask/src/policy/check_panic_allowlist.rs`).
- Code: `.github/config/panic-allowlist.yaml`, `crates/*/src/**`.
