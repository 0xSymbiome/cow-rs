---
type: Audit
id: dependency-gate
title: "Dependency Gate Audit"
description: "The release-facing dependency gate splits cargo-deny and cargo-audit policy, sources every advisory tolerance from one register, and fences the native Alloy and wasm32 dependency boundaries."
status: Current
owning_surface: "release-facing dependency-audit gate"
related: [ADR-0006]
timestamp: 2026-06-29
---

# Dependency Gate Audit

## Scope

Reviews the dependency-audit gate used by routine CI and release readiness: the
cargo-deny / cargo-audit split, the canonical advisory register, the
dependency-source whitelist, the native Alloy two-family lockfile invariant and
adapter allow-lists, and the `cow-sdk-js` wasm32 exclusion list. It does not
cover broader dependency-freshness reporting or license policy beyond the
blocking gate.

## Findings

- `cargo deny` owns allowed sources, licenses, yanked-crate policy, and curated
  duplicate-version tolerances, while `cargo audit --deny unsound --deny
  unmaintained` blocks RustSec vulnerabilities plus unsound and unmaintained
  advisories.
- `.github/config/deny.toml` is the single advisory register; CI derives the
  `cargo audit --ignore` arguments from it, and `cargo docs-agree` fails if any
  tolerated advisory lacks a rationale in this audit (see Tracked advisories).
- The dependency-source policy is fail-closed: unknown registries and all git
  sources are denied, anchoring the graph to crates.io plus workspace paths.
- The lockfile keeps Alloy runtime crates and Alloy Core ABI crates as separate
  reviewed families with exactly one resolved version each; an xtask test fails
  on a second version.
- Only the reviewed adapter crates may carry `alloy-provider` /
  `alloy-signer-local`; the `cargo check-alloy-{provider,signer}-invariant`
  gates fail if that reachability escapes into any other shipped crate.
- The `cow-sdk-js` wasm32 dependency tree excludes the native Alloy adapters,
  reqwest, and hyper, and the `helpers` module stays free of JavaScript FFI
  crates.

## Tracked advisories

Four RustSec identifiers are tolerated, each living in
`.github/config/deny.toml` under `[advisories].ignore`. Three are build-time
macro subtrees of the native Alloy stack that compile no runtime `cow-sdk`
code; the fourth is an unmaintained timekeeping shim confined to the
experimental WebAssembly component's WASI 0.2 lane:

- `RUSTSEC-2024-0388` — `derivative`, reached only through the
  `alloy-trie` / `alloy-consensus` subtree. Revisit when that stack drops it.
- `RUSTSEC-2024-0436` — `paste`, reached only through the `alloy-sol-macro` /
  `alloy-primitives` proc-macro subtrees. Revisit when the pinned Alloy family
  drops it.
- `RUSTSEC-2026-0173` — `proc-macro-error2`, reached only through the
  `alloy-sol-macro` subtree that derives the inline `sol!` bindings. Revisit
  when that release drops it.
- `RUSTSEC-2024-0384` — `instant`, an unmaintained (not vulnerable) timekeeping
  shim reached only through `wstd`'s pinned `futures-lite 1.x` / `fastrand 1.x`
  subtree in the `cow-sdk-component` WASI 0.2 sync-HTTP lane
  (`world-client-sync`). `wstd 0.6.x` pins the old `futures-lite`, so it cannot
  be resolved away without an upstream `wstd` release, and the component is a
  `publish = false` experimental leaf off the default dependency graph. Revisit
  when `wstd` ships a `futures-lite 2.x` line.

## Evidence

- Decision: [ADR 0006](../adr/0006-explicit-policy-contracts-and-instance-scoped-runtime-state.md).
- Invariants: the `PROP-WS` family ([workspace policy](../properties/workspace.md)).
- Governing gate: `cargo deny check`, `cargo audit`, and `cargo docs-agree` (`xtask/src/docs/agree.rs`).
- Code: `.github/config/deny.toml`, `xtask/src/policy/dependency_invariant.rs`, `tests/alloy_two_family_lockfile_invariant.rs`, `tests/wasm_dependency_invariant.rs`.
