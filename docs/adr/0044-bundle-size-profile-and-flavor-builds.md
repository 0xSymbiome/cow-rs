---
type: Decision Record
id: ADR-0044
title: "ADR 0044: Ship Feature-Scoped WASM Flavor Builds From One Package"
description: "cow-sdk-js ships as one Cargo crate and one npm package with multiple feature-scoped flavor builds."
status: Accepted
date: 2026-05-11
last_reviewed: 2026-06-19
authors: ["0xSymbiotic"]
tags: [wasm, npm, bundle-size, package-flavors]
related: [ADR-0039, ADR-0047]
timestamp: 2026-06-19T00:00:00Z
---

# ADR 0044: Ship Feature-Scoped WASM Flavor Builds From One Package

## Decision

`cow-sdk-js` ships as one Cargo crate and one npm package with multiple
feature-scoped flavor builds. The public package exposes `default`, `orderbook`,
`signing`, and `trading` subpaths through the package exports map. Every flavour is
built for the `bundler`, `nodejs`, and `web` targets plus the standards-track
source-phase `module` build, so each one exposes a web (edge) facade at `…/edge`, a
precompiled wasm module at `…/edge/wasm`, and a source-phase build at `…/module`
(the root flavour's at `./edge`, `./edge/wasm`, and `./module`). Release builds use
the workspace release profile and a wasm optimization pass before package
verification.

## Why

Most consumers need a focused subset of the SDK. Per-flavor builds keep small
applications from paying for orderbook, signing, trading, IPFS, and Cloudflare
support when they import only one surface, while one package keeps versioning
and installation simple.

The decision to ship feature-scoped flavors does not position `cow-sdk-js` as
a replacement for the upstream `@cowprotocol/cow-sdk` TypeScript SDK. Compiling
the Rust SDK to wasm32 produces a binary larger than the upstream TypeScript SDK
at equivalent feature subsets. The flavor split exists
so consumers in specialized use cases (deterministic Rust signing parity,
single-source-of-truth Rust + TypeScript embedding, Cloudflare Workers) can
choose the smallest runtime surface that covers their workflow. For standard
browser dapps and TypeScript applications, the upstream TypeScript SDK is the
recommended choice.

## Must Remain True

- Flavor subpaths are public package exports, not deep `dist/raw` paths.
- Cloudflare Workers use a web-target facade plus a precompiled wasm module
  subpath (each flavour's `…/edge` and `…/edge/wasm`, for example `./trading/edge`).
- Flavor and target are separate axes. A flavor is a feature set; a target
  (`bundler`, `nodejs`, `web`) is a runtime loader. Every flavor is built for all
  three targets. Each flavor's `node` condition loads the `nodejs` CommonJS build;
  its `browser`, `import`, `default`, and edge conditions (`workerd`, `worker`,
  `deno`, `edge-light`, `bun`), plus the explicit `…/edge` subpath, load the `web`
  build, which instantiates its wasm through `new URL(import.meta.url)` and is
  therefore portable across every bundler and with no bundler — the bundler target's
  `import * as wasm` ESM integration is not. Browser consumers call `initialize()`
  once; Workers pass the precompiled module. The `bundler` target still produces the
  one canonical wasm binary that the `web` and `nodejs` glue reuse, but its facade
  ESM entry is not a browser export. No flavor is browser-portable while another is
  bundler-only: every flavor — `default`, `orderbook`, `signing`, and `trading` —
  ships the same target coverage, so a browser consumer of any feature set gets the
  portable web path. Each flavor's `bundler` and `web` targets emit a byte-identical
  wasm binary, so the package ships one binary per flavor, and both the web glue's
  default loader URL and the raw Worker module subpath point at that bundler copy.
- Every flavour additionally ships a source-phase `module` build at `…/module`
  (TC39 source-phase imports / Wasm ESM Integration), driven through
  `wasm-bindgen --target module` because wasm-pack does not emit it. It
  auto-initializes like the bundler build (no `initialize()` call) and shares the
  one byte-identical bundler binary through a repointed `import source` specifier.
  It is an opt-in forward-compatible path (Node 24, Deno, and esbuild today); it
  does not replace the portable `web` build as each flavour's browser default until
  source-phase is broadly supported by bundlers.
- The shipped flavor enumeration is exactly four — `default`, `orderbook`,
  `signing`, and `trading`. No `full` flavor ships: its feature set was
  mechanically identical to `default` (verified through the shipped
  `flavours.json` descriptor), so the `./full` package subpath is not in the
  exports map.
- Package verification proves every exported JavaScript and declaration target
  exists.
- Size measurement is tied to the generated package artifacts and can fail the
  release gate when budgets are exceeded.
- The cost is extra build orchestration and snapshot maintenance for each
  shipped flavor.
- Public docs do not frame `cow-sdk-js` as a replacement for the upstream
  `@cowprotocol/cow-sdk` TypeScript SDK; the consumer routing matrix in
  `README.md` and `crates/js/README.md` documents the supported use cases.
- Each flavor's gzip artifact size is verified against its configured Cloudflare
  Workers compressed-size byte budget on every release build. The byte budget
  tracks Cloudflare's published Paid/Bundled (~3 MB) compressed-size limit (the
  configured fail threshold is below that platform limit with safety margin).
- Full Workers support depends on additional release-bundle and startup-time
  gates that are not enforced by the size release gate alone; those gates are
  tracked in the comparative benchmark validation note's refresh triggers.
- Flavor builds reduce the WASM package footprint within `cow-sdk-js`; they
  do not make WASM size-competitive with the upstream `@cowprotocol/cow-sdk`
  TypeScript SDK at equivalent feature subsets.

## Alternatives Rejected

- Ship one maximal wasm artifact: simple, but it makes minimal browser and
  signing-only use cases pay for unused code.
- Publish separate npm packages per flavor: smaller mental model per package,
  but it multiplies names, versions, and install guidance before publication.
- Make raw wasm-pack targets public package contracts: convenient for build
  output, but too unstable for a consumer-facing SDK.

## Links

- [WASM Surface Audit](../audit/wasm-surface-audit.md)

**Proven by:**

- [WASM Surface Audit](../audit/wasm-surface-audit.md)
