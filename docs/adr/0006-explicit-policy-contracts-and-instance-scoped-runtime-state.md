---
type: Decision Record
id: ADR-0006
title: "ADR 0006: Explicit Policy Contracts And Instance-Scoped Runtime State"
description: "Keep shared policy contracts explicit and review-visible, and keep optional runtime state instance-scoped through builders or typed configuration."
status: Accepted
date: 2026-04-11
authors: ["0xSymbiotic"]
tags: [policy, transport, builders, runtime-state]
related: [ADR-0002, ADR-0005]
timestamp: 2026-04-11T00:00:00Z
---

# ADR 0006: Explicit Policy Contracts And Instance-Scoped Runtime State

## Decision

Keep shared policy contracts explicit and review-visible, and keep optional
runtime state instance-scoped through builders or typed configuration.

## Why

Hidden defaults, singleton clients, implicit precedence, and undocumented
package policy are common ways to erode a clean SDK architecture.
If policy is not visible in code, docs, manifests, and tests, later capability
work will copy accidental behavior instead of intentional design.

## Must Remain True

- Public surface: shared transport policy, package posture, feature behavior,
  and other semver-visible knobs remain explicit in builders, typed config,
  manifests, and public docs rather than hiding in library defaults.
- Runtime and support: no process-global clients, caches, adapters, or hidden
  precedence matrices become the default pattern. Optional runtime state stays
  owned by the instance that uses it.
- Validation and review: tests and docs make precedence, override rules,
  transport behavior, docs.rs posture, MSRV posture, and release expectations
  reviewable rather than implicit.
- Cost: constructors, manifests, and documentation are more explicit, and the
  SDK gives up some convenience that would rely on hidden policy.

## Alternatives Rejected

- Hide policy inside transport defaults or global helpers: short-term
  convenience, but poor reviewability and poor reuse discipline.
- Encode optional behavior as loose flag combinations: easy to grow, but hard
  to reason about once precedence and runtime state interact.

## Links

- [Architecture](../guides/architecture.md)
- [Release Checklist](../guides/release-checklist.md)
- [Verification Guide](../guides/verification.md)

**Proven by:**

- [Credential Redaction Audit](../audit/credential-redaction-audit.md)
- [Dependency Gate Audit](../audit/dependency-gate-audit.md)
