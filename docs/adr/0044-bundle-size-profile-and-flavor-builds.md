# ADR 0044: Ship Feature-Scoped WASM Flavor Builds From One Package

- Status: Accepted
- Date: 2026-05-11
- Last reviewed: 2026-06-18
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, npm, bundle-size, package-flavors
- Related: [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), ADR 0047

## Decision

`cow-sdk-wasm` ships as one Cargo crate and one npm package with multiple
feature-scoped flavor builds. The public package exposes `default`, `orderbook`,
`signing`, and `trading` subpaths through the package exports map, plus the
`trading` flavour's web (edge) facade at `./trading/edge` and its precompiled
wasm module at `./trading/edge/wasm`. Release builds use the workspace release
profile and a wasm optimization pass before package verification.

## Why

Most consumers need a focused subset of the SDK. Per-flavor builds keep small
applications from paying for orderbook, signing, trading, IPFS, and Cloudflare
support when they import only one surface, while one package keeps versioning
and installation simple.

The decision to ship feature-scoped flavors does not position `cow-sdk-wasm` as
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
  subpath (`./trading/edge` and `./trading/edge/wasm`).
- Flavor and target are separate axes. A flavor is a feature set; a target
  (`bundler`, `nodejs`, `web`) is a runtime loader. The `trading` flavor is built
  for all three targets, so one feature set serves browser dApps (bundler), Node
  (nodejs), and edge runtimes (web) — the web build is reached at the
  `./trading/edge` subpath and through the `workerd`, `worker`, `deno`,
  `edge-light`, and `bun` export conditions on `./trading`, not through a
  vendor-named flavor. The `default`, `orderbook`, and `signing` flavors ship the
  `bundler` and `nodejs` targets only: their facade ESM and CommonJS entries
  consume the `bundler` and `nodejs` raw builds respectively, so a standalone
  `web` build for those flavors is unreferenced by any package export and is not
  produced. The `trading` flavor's `bundler` and `web` targets emit a
  byte-identical wasm binary, so the package ships one binary per flavor and the
  raw Worker module subpath reuses the bundler copy.
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
- Public docs do not frame `cow-sdk-wasm` as a replacement for the upstream
  `@cowprotocol/cow-sdk` TypeScript SDK; the consumer routing matrix in
  `README.md` and `crates/wasm/README.md` documents the supported use cases.
- The `trading` flavor's gzip artifact size is verified against the configured
  Cloudflare Workers compressed-size byte budget on every release build. The
  byte budget tracks Cloudflare's published Free compressed-size limit (the
  configured fail threshold is below the platform limit with safety margin).
- Full Workers support depends on additional release-bundle and startup-time
  gates that are not enforced by the size release gate alone; those gates are
  tracked in the comparative benchmark validation note's refresh triggers.
- Flavor builds reduce the WASM package footprint within `cow-sdk-wasm`; they
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
