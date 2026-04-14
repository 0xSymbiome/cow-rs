# ADR 0006: Explicit Policy Contracts And Instance-Scoped Runtime State

- Status: Accepted
- Date: 2026-04-11
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: policy, transport, builders, runtime-state
- Related: [ADR 0002](0002-dedicated-trading-orchestration-crate.md), [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)

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

- [Architecture](../architecture.md)
- [Release Checklist](../release-checklist.md)
- [Verification Guide](../verification-guide.md)
