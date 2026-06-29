# Audits

`docs/audit/` records current-state reviews of trust-significant `cow-rs`
surfaces.

> These are internal current-state engineering reviews by the maintainer, not
> independent third-party security audits. Each records what was reviewed, the
> conclusion, the explicit boundary, and the repository-visible evidence behind
> it.

## Audit Contract

Each audit states four things and nothing more:

- **Surface + boundary** — the named surface reviewed, and what is explicitly out
  of scope (the boundary no ADR, principle, or property row carries).
- **Attestation** — that the surface was reviewed against the present
  implementation, dated by `timestamp`.
- **Refresh trigger** — the condition that voids the attestation.
- **Pointers** — the ADR (decision), principle (rule), `PROP-*` rows (invariant +
  tests), governing gate, and code roots. The audit *addresses* the proof; it
  does not reproduce it.

This lane is not for exploratory notes, changelog fragments, ADR replacement, or
re-narrating the implementation.

## Contracts And On-Chain Surfaces

| Audit | Owning surface | Status | Last reviewed |
| --- | --- | --- | --- |
| [Contract Bindings Parity Audit](contract-bindings-parity-audit.md) | canonical `alloy::sol!` bindings | Current | 2026-06-20 |
| [Event Log Decoding Audit](event-log-decoding-audit.md) | contracts event-log decoders | Current | 2026-06-20 |
| [ECDSA Signature Normalization Audit](ecdsa-signature-normalization-audit.md) | `RecoverableSignature` + ECDSA recovery | Current | 2026-06-20 |
| [EIP-1271 Verification Cache Audit](eip1271-verification-cache-audit.md) | `Eip1271Cache` trait + `NoopEip1271Cache` | Current | 2026-06-24 |
| [Deployment Registry Audit](deployment-registry-audit.md) | `Registry` deployment authority | Current | 2026-06-20 |
| [COW Shed Contract Bindings Audit](cow-shed-contract-bindings-audit.md) | COW Shed bindings + app-data | Current | 2026-06-20 |

## Transport And Adapters

| Audit | Owning surface | Status | Last reviewed |
| --- | --- | --- | --- |
| [HTTP Transport Contract Audit](http-transport-contract-audit.md) | `HttpTransport` trait + adapters + policy | Current | 2026-06-20 |
| [Bounded Response Reads Audit](bounded-response-reads-audit.md) | bounded HTTP response reads | Current | 2026-06-20 |
| [Alloy Adapters Audit](alloy-adapters-audit.md) | native Alloy adapter family | Current | 2026-06-21 |

## Trading

| Audit | Owning surface | Status | Last reviewed |
| --- | --- | --- | --- |
| [Trading Order Integrity Audit](trading-order-integrity-audit.md) | trading order assembly + validation | Current | 2026-06-20 |

## JavaScript And TypeScript WASM

| Audit | Owning surface | Status | Last reviewed |
| --- | --- | --- | --- |
| [WASM Surface Audit](wasm-surface-audit.md) | `cow-sdk-js` surface + runtime | Current | 2026-06-26 |

## Cross-Cutting Safety And Hygiene

| Audit | Owning surface | Status | Last reviewed |
| --- | --- | --- | --- |
| [Credential Redaction Audit](credential-redaction-audit.md) | cross-cutting credential redaction | Current | 2026-06-21 |
| [Error Classification Audit](error-classification-audit.md) | `ErrorClass` + `class()` accessors | Current | 2026-06-21 |
| [Panic-Free Public Surface Audit](panic-free-public-surface-audit.md) | public-API panic surface | Current | 2026-06-20 |
| [Fuzz Coverage Audit](fuzz-coverage-audit.md) | `cow-sdk-fuzz` targets | Current | 2026-06-21 |
| [Dependency Gate Audit](dependency-gate-audit.md) | dependency-audit gate | Current | 2026-06-29 |
| [Workflow Security Audit](workflow-security-audit.md) | GitHub workflow security posture | Current | 2026-06-20 |
| [Source-Lock Provenance Audit](source-lock-provenance-audit.md) | source-lock provenance + preflight | Current | 2026-06-20 |

## Anchor Contract

- Every audit uses exactly these H2 sections, in order: `Scope`, `Findings`,
  `Evidence`. `dependency-gate-audit.md` additionally carries `Tracked
  advisories` because a gate keeps that table in sync with `deny.toml`.
- `Scope` ends with an explicit out-of-scope boundary sentence.
- `Findings` are conclusions, not a re-narration of the implementation.
- `Evidence` points to the ADR, principle, `PROP-*` rows, governing gate, and
  code roots — it does not list individual tests or shell commands.

## Metadata Contract

- OKF frontmatter: `type: Audit`, `id` (the slug), `title`, `description` (one
  hand-written sentence), `status`, `owning_surface` (short phrase), `related`
  (ADR identifiers only), `timestamp` (the review date). Per-audit refresh
  triggers live in `.github/config/audit-refresh-map.yml`, not the frontmatter.
- `related` is ADR-only and reciprocal: every ADR it names lists this audit under
  `**Proven by:**`, and every ADR whose `**Proven by:**` names this audit appears
  here. See-also links to other audits or guides go in prose.

## Format Contract

- Lean enough to absorb in under a minute. If an audit grows past that, it is
  re-narrating a sibling — cut to pointers.
- Not a changelog: no delivery history, no per-run command logs.
- Point, do not reproduce: the decision is the ADR's, the rule is the
  principle's, the invariant and its tests are the property row's, the mechanism
  is the code's.
- An enumerated inventory table is allowed only when a named gate keeps it in
  sync (today: `dependency-gate` ⇄ `deny.toml`).

## Status Model

- `Current` — reviewed against the present implementation; no invalidating change
  known.
- `Refresh required` — the reviewed surface or a dependency shifted; the record
  needs re-confirmation.
- `Superseded` — replaced by a newer record.

## Refresh Rule

If a change materially touches an audited surface, re-confirm the record and move
its `timestamp` in the same change set; otherwise leave it. Per-audit refresh
triggers are recorded in `.github/config/audit-refresh-map.yml`;
`cargo check-audit-freshness --base <ref>` reports — without failing — when a
mapped path changed but the owning audit's `timestamp` did not.

## Cross-Link Contract

The ADR↔audit reciprocity, the per-file skeleton, frontmatter, and `related`
scheme, and the audit↔refresh-map 1:1 correspondence and `owning_surface`
agreement are enforced by `cargo check-audit-lane` (part of `cargo
check-policies`); index-date agreement is enforced by `cargo docs-agree`.
