---
type: Principle
title: "Instance-Scoped Configuration"
description: "Policy-heavy behavior is configured per instance through typed builders or options, never process-global mutable state."
tags: [principle]
timestamp: 2026-06-29T00:00:00Z
anchored_by: [ADR-0006]
shape: rule
enforced_by: "documentation-only (unenforced)"
---

# Instance-Scoped Configuration

**Invariant** — Policy-heavy behavior — quote settings, transport tuning, caching, and provider
selection — is configured per instance through typed builders or options. `cow-rs` hides no
process-global mutable state behind convenience APIs.

**Why** — Process-global policy makes two SDK instances in one process interfere, breaks
reentrancy, and turns configuration into action at a distance that no call site can see.

**How to comply**
- Take policy as builder or option parameters on the instance that uses it.
- Never read or write a mutable global static for policy; immutable constants and one-time
  seeds are fine.

**Enforced by** — documentation-only (unenforced). No gate forbids process-global state today;
a process-global-state fence is a candidate future hardening.

**Anchored by**: [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md) (primary). Supporting: none.
