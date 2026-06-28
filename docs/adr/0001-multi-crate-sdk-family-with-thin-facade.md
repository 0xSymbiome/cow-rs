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
analytics access, and a JavaScript and TypeScript wasm surface. Public package
identity needs to read as
an SDK family on crates.io, while local workspace paths should remain short.
A single crate, thick root facade, or repository-shaped public crate family
would blur product identity, crate ownership, and runtime boundaries.

## Must Remain True

- Public surface: the repository may remain `cow-rs`, but published crates use
  `cow-sdk` and `cow-sdk-*`. Leaf crates are first-class entry points, while
  `cow-sdk` stays a narrow facade instead of the only meaningful integration
  surface.
- Public root surface: the `cow-sdk` crate root is an **explicit, curated,
  module-organised** surface — each leaf crate is re-exported as a named module
  (`cow_sdk::core`, `cow_sdk::trading`, `cow_sdk::orderbook`, …), and every
  workflow and identity type is reached on its module path
  (`cow_sdk::core::Address`, `cow_sdk::trading::Trading`), matching `alloy`,
  `reqwest`, and `tower`. The crate root itself carries only the cross-cutting
  aggregate error (`CowError` / `ErrorClass`) and the typed transport-policy
  surface (`cow_sdk::http`); the deployment `Registry` and the EIP-1271
  verification cache are reached on their module paths (`cow_sdk::contracts`
  and `cow_sdk::signing`), not at the root. The facade ships
  **no prelude** — there is no `cow_sdk::prelude` — and the root is never grown
  by a glob (`pub use <module>::*` is disallowed), so it stays explicit and
  pinnable in the public-API snapshot. No crate in the workspace ships a prelude;
  identity and numeric newtypes are reached on their module path, the way `alloy`
  and `reqwest` scope theirs.
- Additive growth: new capability surfaces land as additive leaf crates or
  off-by-default features (subgraph, the `cow-shed` contracts feature, the
  published `cow-sdk-test` doubles), never by widening the default
  facade closure; an optional capability a default consumer does not use adds
  nothing to its dependency graph.
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
