# Alloy Runtime Dependency Audit Baseline

Status: Current
Last reviewed: 2026-05-05
Owning surface: native Alloy adapter dependency graph
Refresh trigger: changes to native Alloy adapter Cargo features, Alloy runtime
versions, or the workspace lockfile dependency graph

## Scope

This baseline covers RustSec advisories introduced by adding the native Alloy
runtime dependency set to the workspace lockfile.

It does not approve or suppress advisories. It records the observed delta so
dependency-audit review can distinguish newly introduced runtime exposure from
pre-existing workspace state.

## Evidence

The advisory comparison used:

```text
cargo audit --json
cargo metadata --format-version 1 --locked
```

The comparison result is:

| Measurement | Count |
| --- | ---: |
| Advisories before adding the native Alloy runtime dependency set | 1 |
| Advisories after adding the native Alloy runtime dependency set | 2 |
| Newly introduced advisories | 1 |
| Newly introduced vulnerability advisories | 0 |
| Newly introduced informational advisories | 1 |

## Newly Introduced Advisories

| Advisory | Kind | Package | Resolved version | Summary |
| --- | --- | --- | --- | --- |
| RUSTSEC-2024-0388 | unmaintained | derivative | 2.2.0 | The crate is unmaintained; cargo-audit reports this as informational. |

No vulnerability advisory was introduced by the resolved native Alloy runtime
dependency set.
