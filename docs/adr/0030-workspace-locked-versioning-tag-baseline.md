---
type: Decision Record
id: ADR-0030
title: "ADR 0030: Workspace-Locked Versioning With Patch Tag Baselines"
description: "Every crate in the cow-sdk family carries the same workspace version through 0.x.y."
status: Accepted
date: 2026-04-29
last_reviewed: 2026-06-15
authors: ["0xSymbiotic"]
tags: [versioning, semver, release, compatibility]
related: [ADR-0026]
timestamp: 2026-06-15T00:00:00Z
---

# ADR 0030: Workspace-Locked Versioning With Patch Tag Baselines

## Decision

Every crate in the `cow-sdk` family carries the same workspace version
through `0.x.y`. Per-crate version divergence is allowed only after
`1.0.0` and only for patch releases where the changed crate can publish
without changing the facade contract.

The version-lockstep rule above is active today. The `cargo-semver-checks`
policy below is the **target release-gate policy**, not a currently-running
CI lane: a pre-1.0 semver report against an unpublished baseline is
non-blocking, so the lane is removed through the pre-1.0 cycle and is
reintroduced on the 1.0 runway (see [Release Checklist](../guides/release-checklist.md)).
The `SemverChecksMode` classifier in `xtask` already encodes the per-release
modes below, so the gate activates without rework. When it runs, the patch
baseline is the crates.io version published from the previous tag
`v0.x.(y-1)`, and the facade crate `cow-sdk` is checked as a first-class
package alongside the leaf crates.

## Why

Pre-1.0 sibling crates are easiest for consumers to reason about when
they move in lockstep. A patch release should not break a user who pins
`cow-sdk = "0.1"`, even while the workspace is still pre-1.0. Naming the
baseline tag removes ambiguity from the semver-checks lane and makes the
patch contract reproducible for reviewers.

## Must Remain True

| Release | `cargo-semver-checks` mode | Baseline |
| --- | --- | --- |
| `0.1.0` first functional release | skipped | none |
| `0.1.y` patch where `y >= 1` | blocking | tag `v0.1.(y-1)` |
| `0.x -> 0.(x+1)` minor | advisory | previous tag |
| `1.x.y` patch | blocking | previous patch tag |
| `1.x -> 1.(x+1)` minor | blocking unless explicitly opted out | previous tag |
| major release | skipped | none |

- Workspace crates use the same version through `0.x.y`.
- Patch releases fetch tags before running semver checks.
- Advisory minor reports are preserved with release evidence and, when
  they detect public breaks, named in the changelog.
- The `cow-sdk` facade is never omitted from semver compatibility checks.

## Alternatives Rejected

- Let each crate version independently during `0.x`: flexible, but makes
  resolver behavior and consumer upgrade paths harder to predict.
- Run semver checks only after `1.0.0`: avoids early noise, but leaves
  patch consumers without a compatibility floor.
- Compare against an implicit latest crates.io version: convenient, but
  not reproducible when branch state and published tags disagree.

## Anchors

This ADR supports the Evidence-Backed Public Claims principle.

## Links

- [Principles](../principles/index.md)
- [Verification Matrix](../guides/verification.md)
- [Release Checklist](../guides/release-checklist.md)
- [Source-Lock Provenance Audit](../audit/source-lock-provenance-audit.md)
- [ADR 0026](0026-alloy-major-release-absorption-plan.md)

**Proven by:**

- [Source-Lock Provenance Audit](../audit/source-lock-provenance-audit.md)
- `xtask/tests/classify_release.rs`
