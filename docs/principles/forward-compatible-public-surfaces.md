---
type: Principle
title: "Forward-Compatible Public Surfaces"
description: "Every public type absorbs additive upstream growth without a breaking change, or is deliberately frozen."
tags: [principle]
timestamp: 2026-06-29T00:00:00Z
anchored_by: [ADR-0031, ADR-0027, ADR-0058]
shape: classify
enforced_by: "check-enum-policy + clippy missing_errors_doc/must_use_candidate under -D warnings"
---

# Forward-Compatible Public Surfaces

**Invariant** — Every public type can absorb additive upstream or protocol growth without a
breaking change, *or* is deliberately frozen because growth there is itself a protocol or schema
change. Public response DTOs preserve unknown fields under `serde` defaults; frozen chain-RPC
traits grow through opt-in capability supertraits, never by widening the base; fallible public
APIs carry `#[must_use]` and a `# Errors` doc section.

**Why** — A surface that breaks on every upstream addition forces a major version for routine
protocol growth; a surface left open where it should be frozen hides a protocol change as an
innocuous minor.

**How to comply**
- Classify every public enum in `enum-policy.yaml`; `upstream-growing` enums carry `#[non_exhaustive]`.
- Pick a struct posture by role — walk the decision below.
- Keep response DTOs on `serde` defaults (preserve unknown fields), not `deny_unknown_fields`.

**Decision**

```mermaid
flowchart TD
  start(["New or reviewed public item"]) --> kind{"What kind?"}
  kind -->|"enum"| e{"Variant source?"}
  e -->|"protocol or upstream can grow"| e1["non_exhaustive (class: upstream-growing)"]
  e -->|"fixed protocol set"| e2["exhaustive (class: protocol-fixed-exhaustive)"]
  e -->|"SDK-internal state machine"| e3["may be exhaustive (class: sdk-local-state)"]
  e1 --> ereg[["record in enum-policy.yaml (CI-enforced)"]]
  e2 --> ereg
  e3 --> ereg
  kind -->|"struct"| s{"Role?"}
  s -->|"SDK-produced output: DTO, event, recorded call"| s1["non_exhaustive; if a DTO, keep unknown fields via serde defaults"]
  s -->|"frozen wire or ABI: fixed by contract or closed schema"| s2["exhaustive and literal-constructible, e.g. OrderData"]
  s -->|"caller-built request or config"| s3["exhaustive with new() and with_ builder methods; deny_unknown_fields if it mirrors a closed schema"]
  kind -->|"chain-RPC trait"| t["grow via capability supertrait such as SigningProvider or LogProvider; never widen the base"]
  kind -->|"any fallible API"| m["add must_use and an Errors doc section"]
```

**Enforced by** — `check-enum-policy` validates the manifest; the `missing_errors_doc`,
`missing_panics_doc`, and `must_use_candidate` clippy lints (warn in `Cargo.toml`) are promoted to
hard errors by `cargo clippy -- -D warnings` in the quality gate.

**Anchored by**: [ADR 0031](../adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md) (primary). Supporting: [ADR 0027](../adr/0027-post-quantum-signing-absorption-plan.md), [ADR 0058](../adr/0058-typed-quote-request-response-surface.md).
