# ADR 0033: Minimum-Viable Panic Surface

- Status: Accepted
- Date: 2026-04-29
- Last reviewed: 2026-04-29
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: panic, safety, public-api, audit
- Anchors: Minimum-Viable Panic Surface (primary)
- Related: none

## Decision

Production code in shipped crates does not contain
`unwrap`/`expect`/`panic!`/`unreachable!`/`todo!`/`unimplemented!`
outside statically-invariant compile-time guarantees.

Each accepted statically-invariant panic site carries three things:

1. a `# Panics` rustdoc section on its public function;
2. an inline `// SAFETY:` comment naming the build-time invariant;
3. an entry in `.github/config/panic-allowlist.yaml` keyed by item path.

`policy-maintainer check-panic-allowlist` fails on any panic-bearing
production-source call not listed in the allowlist and on any allowlist
entry that points at a non-existent symbol.

## Why

Panics in production library code are semver-visible behavior. An
allowlist keyed by item path is auditable and stable across line-number
movement. Pairing the allowlist with rustdoc and inline safety comments
makes every retained panic site a deliberate exception rather than an
accidental convenience.

## Must Remain True

- The panic allowlist file is committed and reviewed.
- Adding a panic-bearing call requires either an allowlist entry with
  the required documentation or a refactor to a typed `Result` return.
- Allowlist entries use item paths, not line numbers.
- Public panic documentation and inline `// SAFETY:` comments stay close
  to the accepted panic site.
- [Panic-Free Public Surface Audit](../audit/panic-free-public-surface-audit.md)
  cross-references the allowlist count.

## Alternatives Rejected

- Anchor Minimum-Viable Panic Surface to ADR 0006: ADR 0006 is about instance-scoped
  configuration, not panic policy. Mapping panic policy there would
  dilute both ADRs.
- Make Minimum-Viable Panic Surface audit-only: insufficient, because release gates need
  a workspace-wide invariant decision that survives individual audit
  refreshes.
- Track panic exceptions by line number: easy to produce, but noisy under
  formatting and unrelated edits.

## Anchors

This ADR is the primary and sole anchor for the
Minimum-Viable Panic Surface principle.

## Links

- [Principles](../principles.md)
- [Panic-Free Public Surface Audit](../audit/panic-free-public-surface-audit.md)
- `.github/config/panic-allowlist.yaml`

**Proven by:**

- [Panic-Free Public Surface Audit](../audit/panic-free-public-surface-audit.md)
- `scripts/policy-maintainer/src/check_panic_allowlist.rs`
