# ADR 0001: Multi-Crate SDK Family With Thin Facade

- Status: Accepted
- Date: 2026-04-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: topology, packages, facade

## Decision

Use a multi-crate workspace with short local folders under `crates/*`, a
`cow-sdk` public facade, and `cow-sdk-*` leaf crates that own behavior.

## Why

The SDK spans protocol transforms, transport clients, trading workflows,
analytics access, and browser support. Public package identity needs to read as
an SDK family on crates.io, while local workspace paths should remain short.
A single crate, thick root facade, or repository-shaped public crate family
would blur product identity, crate ownership, and runtime boundaries.

## Must Remain True

- Public surface: the repository may remain `cow-rs`, but published crates use
  `cow-sdk` and `cow-sdk-*`. Leaf crates are first-class entry points, while
  `cow-sdk` stays a narrow facade instead of the only meaningful integration
  surface.
- Runtime and support: runtime-specific dependencies can stay isolated instead
  of forcing one dependency and runtime model across the whole workspace.
- Validation and review: targeted crate-level tests, docs, and review can stay
  aligned to owned behavior instead of one large mixed surface, and local
  folders can remain shorter than published crate names.
- Cost: the workspace has more packages to coordinate, document, publish, and
  name deliberately.

## Alternatives Rejected

- Single crate: couples unrelated dependencies, runtime assumptions, and semver
  surfaces.
- Thick root facade: hides crate boundaries and makes cross-cutting behavior
  harder to review.
- Public `cow-rs-*` crates or long mirrored local folder names: more symmetric
  with the repository name, but weaker for SDK identity and local ergonomics.

## Links

- [Architecture](../architecture.md)
- [ADR 0002](0002-dedicated-trading-orchestration-crate.md)
- [ADR 0003](0003-separate-read-only-subgraph-crate.md)
- [ADR 0004](0004-feature-gated-browser-wallet-sidecar.md)
