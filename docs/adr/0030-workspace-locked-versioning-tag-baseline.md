# ADR 0030: Workspace-Locked Versioning With Patch Tag Baselines

- Status: Accepted
- Date: 2026-04-29
- Last reviewed: 2026-04-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: versioning, semver, release, compatibility
- Anchors: Evidence-Backed Public Claims (supporting)
- Related: [ADR 0026](0026-alloy-major-release-absorption-plan.md)

## Decision

Every crate in the `cow-sdk` family carries the same workspace version
through `0.x.y`. Per-crate version divergence is allowed only after
`1.0.0` and only for patch releases where the changed crate can publish
without changing the facade contract.

`cargo-semver-checks` is blocking for patch releases starting at
`0.1.1`. The first functional release, `0.1.0`, has no baseline. Patch
baseline source is the crates.io version published from the previous tag
`v0.x.(y-1)`, so release branches fetch tags before the lane runs. The
facade crate `cow-sdk` is checked as a first-class package alongside
the leaf crates.

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

- [Principles](../principles.md)
- [Verification Matrix](../verification-matrix.md)
- [Release Checklist](../release-checklist.md)
- [Source-Lock Provenance Audit](../audit/source-lock-provenance-audit.md)
- [ADR 0026](0026-alloy-major-release-absorption-plan.md)

**Proven by:**

- [Source-Lock Provenance Audit](../audit/source-lock-provenance-audit.md)
- `scripts/policy-maintainer/tests/classify_release.rs`
