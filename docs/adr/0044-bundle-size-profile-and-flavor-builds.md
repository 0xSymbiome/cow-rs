# ADR 0044: Ship Feature-Scoped WASM Flavor Builds From One Package

- Status: Accepted
- Date: 2026-05-11
- Last reviewed: 2026-06-01
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, npm, bundle-size, package-flavors
- Related: [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0047](0047-typescript-facade-architecture.md)

## Decision

`cow-sdk-wasm` ships as one Cargo crate and one npm package with multiple
feature-scoped flavor builds. The public package exposes default, orderbook,
signing, Cloudflare, and Cloudflare wasm subpaths through the package
exports map. Release builds use the workspace release profile and a wasm
optimization pass before package verification.

## Why

Most consumers need a focused subset of the SDK. Per-flavor builds keep small
applications from paying for orderbook, signing, trading, IPFS, and Cloudflare
support when they import only one surface, while one package keeps versioning
and installation simple.

The decision to ship feature-scoped flavors does not position `cow-sdk-wasm` as
a replacement for the upstream `@cowprotocol/cow-sdk` TypeScript SDK. The
benchmark documented in
[cow-sdk-wasm Comparative Benchmark Validation Note](../audit/cow-sdk-wasm-comparative-benchmark-validation-note.md)
shows that compiling the Rust SDK to wasm32 produces a binary larger than the
upstream TypeScript SDK at equivalent feature subsets. The flavor split exists
so consumers in specialized use cases (deterministic Rust signing parity,
single-source-of-truth Rust + TypeScript embedding, Cloudflare Workers) can
choose the smallest runtime surface that covers their workflow. For standard
browser dapps and TypeScript applications, the upstream TypeScript SDK is the
recommended choice.

## Must Remain True

- Flavor subpaths are public package exports, not deep `dist/raw` paths.
- Cloudflare Workers use a web-target facade plus a precompiled wasm module
  subpath.
- The standalone `web` target is built only for the Cloudflare flavor. The
  `default`, `orderbook`, and `signing` flavors ship the `bundler` and `nodejs`
  targets only: their facade ESM and CommonJS entries consume the `bundler` and
  `nodejs` raw builds respectively, so a standalone `web` build for those flavors
  is unreferenced by any package export and is not produced. Browser consumers
  resolve the ESM facade entry through a bundler; a bundler-free browser entry
  for those flavors would come from the wasm-bindgen source-phase `module`
  target, tracked as a future addition rather than shipped today.
- The shipped flavor enumeration is exactly four — `default`, `orderbook`,
  `signing`, and `cloudflare`. No `full` flavor ships: its feature set was
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
- The cloudflare-flavor gzip artifact size is verified against the configured
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

- [WASM Performance Budget Audit](../audit/wasm-performance-budget-audit.md)
- [WASM Public API Stability Audit](../audit/wasm-public-api-stability-audit.md)
- [WASM Facade Architecture Audit](../audit/wasm-facade-architecture-audit.md)

**Proven by:**

- [WASM Performance Budget Audit](../audit/wasm-performance-budget-audit.md)
- [WASM Public API Stability Audit](../audit/wasm-public-api-stability-audit.md)
- [cow-sdk-wasm Comparative Benchmark Validation Note](../audit/cow-sdk-wasm-comparative-benchmark-validation-note.md)
