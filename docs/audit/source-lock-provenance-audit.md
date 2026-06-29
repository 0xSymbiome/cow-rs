---
type: Audit
id: source-lock-provenance
title: "Source-Lock Provenance Audit"
description: "Every committed parity fixture and source-derived evidence file is pinned to an exact upstream commit, with pins held behind upstream frozen by documented hold reasons."
status: Current
owning_surface: "source-lock provenance pins and the release preflight that validates them"
related: [ADR-0026, ADR-0030]
timestamp: 2026-06-20
---

# Source-Lock Provenance Audit

## Scope

Reviews `parity/source-lock.yaml` and the preflight that holds every
`parity/fixtures/**/*.json` to a pinned upstream commit: the per-file provenance
headers, the freshness disclosure for pins held behind upstream, and the deep
upstream-root validation. It does not cover the fixtures' wire values (the
per-surface parity audits) or files outside the `parity/fixtures/**` glob.

## Findings

- The lock pins each upstream source (`contracts`, `services`, `cow-sdk`,
  `cow-shed`, `ethflowcontract`) by exact commit; a pin held behind current
  upstream (notably `cow-shed` at its v1.0.1 tag) carries a documented hold
  reason, and the refresh tool never advances a held pin.
- Every `parity/fixtures/**/*.json` carries typed provenance headers (surface,
  sources, standards) validated per file; the validator rejects unknown or
  missing fields fail-closed.
- `cargo parity-validate --upstream-root <dir>` performs deep validation against
  independent local checkouts — expected remotes, pinned `HEAD`, clean producer
  paths, and a vendored-OpenAPI body match — and fails closed on any mismatch.
- Amount-shaped fixture strings round-trip byte-identically through the `Amount`
  codec, so a transcribed wire value cannot silently drift.
- Per-audit refresh ownership is recorded in `.github/config/audit-refresh-map.yml`,
  which points source-lock changes at this record.

## Evidence

- Decision: [ADR 0026](../adr/0026-alloy-major-release-absorption-plan.md), [ADR 0030](../adr/0030-workspace-locked-versioning-tag-baseline.md).
- Rule: [Evidence-Backed Public Claims](../principles/evidence-backed-public-claims.md).
- Governing gate: `cargo parity-validate` (`xtask/src/parity/`).
- Code: `parity/source-lock.yaml`, `parity/fixtures/`, `.github/config/audit-refresh-map.yml`.
