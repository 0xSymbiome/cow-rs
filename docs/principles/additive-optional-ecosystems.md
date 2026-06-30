---
type: Principle
title: "Additive Optional Ecosystems"
description: "Optional capabilities grow through leaf crates and feature-gated additions, never by widening the default facade."
tags: [principle]
timestamp: 2026-06-29T00:00:00Z
anchored_by: [ADR-0001, ADR-0071]
shape: structure
enforced_by: "crates/js/tests/wasm_flavour_reachability_contract.rs + check-wasm-invariant"
---

# Additive Optional Ecosystems

**Invariant** — Optional capabilities grow through leaf crates and feature-gated additions.
Provider-specific behavior, JavaScript and TypeScript wasm integration, and future capability
families do not silently widen the default facade contract.

**Why** — If an optional capability widens the default build, every consumer pays for it in
compile time, binary size, and API surface — the opposite of opt-in.

**How to comply**
- Ship a new capability as a leaf crate or an off-by-default Cargo feature.
- Leave the default facade contract unchanged when adding an optional family.

**Shape**

```mermaid
flowchart TD
  subgraph cl_default["Default facade: unchanged by additions"]
    core["cow-sdk core surface"]
  end
  subgraph cl_opt_in["Opt-in: off by default"]
    alloy["cow-sdk-alloy adapter crates"]
    js["cow-sdk-js (wasm leaf)"]
    future["future capability families"]
  end
  cl_opt_in -->|"adds onto"| cl_default
```

**Enforced by** — `crates/js/tests/wasm_flavour_reachability_contract.rs` proves each wasm
flavour's bindings stay reachable (an under-gated feature leaking into a leaner build fails),
backed by the `check-wasm-invariant` gate.

**Anchored by**: [ADR 0001](../adr/0001-multi-crate-sdk-family-with-thin-facade.md) (primary). Supporting: [ADR 0071](../adr/0071-wasm-component-distribution-channel.md).
