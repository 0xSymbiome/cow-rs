---
type: Principle
title: "Deterministic Protocol Transforms"
description: "Hashing, signing, UID packing, app-data encoding, and CID handling stay deterministic for the same canonical input."
tags: [principle]
timestamp: 2026-06-29T00:00:00Z
anchored_by: [ADR-0012, ADR-0011, ADR-0022, ADR-0052]
shape: rule
enforced_by: "crates/contracts/tests/parity_contract.rs + property proptests + source fences"
---

# Deterministic Protocol Transforms

**Invariant** — Hashing, signing, UID packing, app-data encoding, and CID handling produce
identical bytes for identical canonical input, and domain identities that share a byte width
stay type-level distinct so a transform cannot consume the wrong domain's bytes.

**Why** — A protocol transform that is non-deterministic, or that silently reads another
domain's bytes, signs over the wrong intent and produces orders that fail to settle.

**How to comply**
- Keep transforms pure functions of their canonical input — no clocks, RNG, map-iteration
  order, or ambient configuration.
- Give each domain identity (order UID, app-data hash, digest) its own type even when the
  underlying width matches, per [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md).

**Enforced by** — parity fixtures (`crates/contracts/tests/parity_contract.rs`) and determinism
proptests (`crates/contracts/tests/property_contract.rs`), plus the `ecdsa-v-normalization` and
`amount-radix` source fences in `xtask/src/policy/fences.rs`.

**Anchored by**: [ADR 0012](../adr/0012-alloy-sol-bindings-and-registry-authority.md) (primary). Supporting: [ADR 0011](../adr/0011-typed-amount-boundary-and-typestate-ready-state-construction.md), [ADR 0022](../adr/0022-ecdsa-signature-v-normalization.md), [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md).
