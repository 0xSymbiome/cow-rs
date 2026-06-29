---
type: Principle
title: "Minimum-Viable Panic Surface"
description: "Shipped crates contain no unwrap/expect/panic outside statically-invariant, allowlisted, documented sites."
tags: [principle]
timestamp: 2026-06-29T00:00:00Z
anchored_by: [ADR-0033]
shape: classify
enforced_by: "check-panic-allowlist (.github/config/panic-allowlist.yaml)"
---

# Minimum-Viable Panic Surface

**Invariant** — Production code in shipped crates contains no
`unwrap`/`expect`/`panic!`/`unreachable!`/`todo!`/`unimplemented!` outside statically-invariant
compile-time guarantees. Each allowed panic site carries a `# Panics` rustdoc section on its
public function and an inline `// SAFETY:` comment naming the build-time invariant.
`.github/config/panic-allowlist.yaml` keys allowed sites by item path; the regression contract
fails on uncommented additions.

**Why** — A panic in a library is a denial-of-service for the consumer's process. The only
acceptable ones are sites a compiler-proven invariant makes genuinely unreachable.

**How to comply**
- Return a `Result` and model the error instead of unwrapping.
- If a site is truly statically invariant, allowlist it and document why — walk the rule below.

**Decision**

```mermaid
flowchart TD
  start(["Reaching for unwrap, expect, panic, unreachable"]) --> q{"Is it a statically-invariant compile-time guarantee?"}
  q -->|"no"| res["Return a Result and model the error; no panic"]
  q -->|"yes"| allow["Allowlist it in panic-allowlist.yaml; add a Panics rustdoc section and an inline SAFETY comment naming the invariant"]
```

**Enforced by** — `check-panic-allowlist` (`xtask/src/policy/check_panic_allowlist.rs`) fails on
any panic-bearing call in a shipped crate that is not allowlisted, and requires the `# Panics`
doc and `// SAFETY:` comment on every allowlisted site.

**Anchored by**: [ADR 0033](../adr/0033-minimum-viable-panic-surface.md) (primary). Supporting: none.
