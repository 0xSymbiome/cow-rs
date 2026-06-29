---
type: Decision Record
id: ADR-0072
title: "ADR 0072: Adopt the Open Knowledge Format for uniform machine-readable documentation"
description: "Every document under docs/ is an Open Knowledge Format concept document with typed YAML frontmatter, so the corpus is queryable by external tools without a bespoke parser."
status: Accepted
date: 2026-06-29
authors: ["0xSymbiotic"]
tags: [documentation, okf, machine-readable, governance]
timestamp: 2026-06-29T00:00:00Z
---

# ADR 0072: Adopt the Open Knowledge Format for uniform machine-readable documentation

## Decision

All documentation under `docs/` follows the
[Open Knowledge Format (OKF)](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md),
Google Cloud's vendor-neutral standard for knowledge as plain Markdown plus YAML
frontmatter. Every document is an OKF **concept document**: a Markdown file whose
frontmatter carries a `type` discriminant plus `title`, `description`, and
`timestamp`. The repository uses eight types — `Decision Record`, `Principle`,
`Property`, `Audit`, `Guide`, `Doctrine`, `Runbook`, and `Policy`. Each `docs/`
subfolder is an OKF bundle with a reserved `index.md`; root-level flat files
(notably `PROPERTIES.md`) were relocated into their typed bundles
(`docs/properties/`, `docs/principles/`, `docs/guides/`, …). Per-lane frontmatter
adds the keys that lane needs (an ADR's `id`/`status`, a principle's
`anchored_by`/`shape`, a property's `families`/`resource`, an audit's
`owning_surface`).

## Why

The documentation set spans a hundred-plus files across decisions, rules,
invariants, audits, and guides. Flat prose is neither queryable nor uniformly
typed, so cross-references drift and tooling cannot route or filter it. OKF makes
every document a self-describing knowledge unit while staying human-readable and
diff-friendly in git — no build step, no SDK, no proprietary tooling. A `type`
discriminant lets external tools and agents consume the corpus without a bespoke
parser, and lets the repository's own gates enforce uniformity.

## Must Remain True

- Public surface: every `docs/` document carries OKF frontmatter with a `type`,
  and every folder has an `index.md`. Per-crate `README.md` files stay
  frontmatter-free because crates.io and docs.rs render them verbatim.
- Runtime and support: the format is plain Markdown plus YAML; consuming it
  requires only a YAML reader.
- Validation: the documentation gates (`check-principles`, `check-audit-lane`,
  `docs-agree`, `audit-index`, `check-property-citations`) enforce the per-lane
  frontmatter, the cross-link graphs, and the index agreement.
- Cost: adding a document means choosing its `type` and writing the lane's
  frontmatter; a document that omits it fails the gate.

## Alternatives Rejected

- A separate `knowledge.okf` bundle mirroring the docs: rejected — it duplicates
  the prose and drifts. OKF is in-source frontmatter, so `docs/` *is* the bundle.
- A bespoke metadata schema or JSON sidecar files: rejected — it reinvents a
  standard, needs a custom parser, and splits metadata from prose.
- Free-form prose without typed frontmatter (the prior state): rejected — not
  queryable, no uniform discriminant, and drift goes uncaught.

## Links

- [Open Knowledge Format specification](https://github.com/GoogleCloudPlatform/knowledge-catalog/blob/main/okf/SPEC.md)
- [Principles](../principles/index.md), [Properties Registry](../properties/index.md), [Audits](../audit/index.md)
- ADR authoring contract in this folder's [index](index.md)
