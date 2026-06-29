---
type: Audit
id: fuzz-coverage
title: "Fuzz Coverage Audit"
description: "The cow-sdk-fuzz crate targets every public untrusted-input boundary, compiles on the stable toolchain in CI, and traces each target to a registered invariant."
status: Current
owning_surface: "the cow-sdk-fuzz crate and its cargo-fuzz targets"
related: []
timestamp: 2026-06-21
---

# Fuzz Coverage Audit

## Scope

Reviews the standalone `cow-sdk-fuzz` crate and its `cargo-fuzz` targets: the
boundary coverage across the published crates, the stable-toolchain compile
gate, the seed-class contract, and the trace from each target to the invariant
it strengthens. It does not cover libFuzzer mutation internals, per-run corpus
growth, or sanitizer-runtime requirements (the scheduled `fuzz` workflow owns
execution).

## Findings

- The targets cover every reviewed public untrusted-input boundary across the
  core, contracts, app-data, orderbook, subgraph, signing, and trading crates;
  the audit describes coverage by boundary class rather than a count that would
  rot on every add (`cargo +nightly fuzz list --fuzz-dir fuzz` is the
  authoritative inventory).
- The fuzz crate stands alone (it needs a nightly toolchain) but is kept
  type-checked against stable on every pull request through the shared quality
  gate, whose path filter includes `fuzz/**`.
- A report-only `fuzz` workflow runs every target for a bounded budget on a
  weekly cron and on demand; it is non-gating and never a pull-request check.
- Each target is seeded locally from a canonical / boundary / adversarial class
  taxonomy; the `fuzz/corpus/` tree is gitignored per cargo-fuzz convention and
  regenerated from the documented classes.
- Each target traces to a `PROP-*` invariant in the
  [Properties Registry](../properties/index.md), and most targets assert a
  semantic property (round-trip equality, classifier determinism, credential
  absence) beyond bare panic-freedom.
- Every target imports only published SDK surface; a crate-private helper is
  exercised through its nearest public wrapper.

## Evidence

- Invariants: the `PROP-*` rows whose evidence cites a fuzz target across the
  [Properties Registry](../properties/index.md).
- Governing gate: the `Check fuzz crate against the stable toolchain` step in
  `.github/workflows/_quality-gate.yml`; the scheduled `.github/workflows/fuzz.yml`.
- Code: `fuzz/Cargo.toml`, `fuzz/fuzz_targets/`, `fuzz/README.md`.
