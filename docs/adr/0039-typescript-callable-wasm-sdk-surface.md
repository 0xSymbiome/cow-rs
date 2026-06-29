---
type: Decision Record
id: ADR-0039
title: "ADR 0039: Keep The JavaScript and TypeScript WASM SDK Surface As An Additive Leaf Crate"
description: "cow-sdk-js is the canonical JavaScript and TypeScript SDK surface."
status: Accepted
date: 2026-05-09
last_reviewed: 2026-06-26
authors: ["0xSymbiotic"]
tags: [wasm, typescript, public-surface, additive-leaf-crates]
related: [ADR-0010, ADR-0013, ADR-0024, ADR-0038, ADR-0044, ADR-0052]
timestamp: 2026-06-26T00:00:00Z
---

# ADR 0039: Keep The JavaScript and TypeScript WASM SDK Surface As An Additive Leaf Crate

## Decision

`cow-sdk-js` is the canonical JavaScript and TypeScript SDK surface. It remains a
publishable additive leaf crate, not part of `cow-sdk-core`, and exposes four
layers: pure protocol helpers in the host-safe `cow-sdk-js::helpers` module, wallet and
signer callback functions, orderbook plus subgraph plus IPFS clients, and
trading clients. EIP-1271 signing uses a facade-resolves-callback pattern:
JavaScript resolves the final signature at the wasm boundary, while Rust stores
only a pure `Send + Sync` provider.

The public npm package surface is the compiled TypeScript facade. Raw
wasm-bindgen output remains an internal package artifact; public package
exports point to facade modules selected by flavor and runtime target. Client
construction uses one typed config object per client, callback registries stay
internal, and non-browser runtimes configure explicit callback HTTP transport
plus `TransportPolicyConfig`.

The runtime support matrix is explicit: browser bundlers are
`default-http-supported`, Node.js 24 LTS plus Cloudflare Workers are
`callback-http-tested` through the shipped web (edge) build — which
every flavour ships, exercised end-to-end via the `trading` flavour in CI — and
Node.js 22 LTS, Deno, Bun, Vercel Edge, and Fly.io are best-effort without a CI
claim.

## Why

JavaScript consumers already own their wallet library, event loop, fetch stack,
and deployment runtime. A callback boundary lets viem, ethers, wagmi, EIP-1193
wallets, Workers, Node, and Deno integrate without forcing any one JavaScript
ecosystem into the Rust crate. Keeping the surface as a leaf preserves the
native SDK dependency graph and keeps wasm-bindgen concerns local to the crate
that exports them.

This decision establishes `cow-sdk-js` as the canonical JavaScript and TypeScript
surface FOR THE COW-RS RUST WASM PACKAGE. It does NOT establish `cow-sdk-js`
as the default CoW Protocol TypeScript SDK for consumers. The upstream
[`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk)
remains the recommended TypeScript SDK for standard browser dapps, web apps,
CowSwap-style UIs, and most TypeScript applications because it is substantially
smaller at equivalent feature subsets.

Runtime support claims for `cow-sdk-js` are split into distinct gates that
this ADR does not blur:

- Public API and facade contract (governed by this ADR).
- Build and package target support (governed by ADR 0044 and the package
  release pipeline).
- Runtime performance and support evidence (governed by the comparative
  benchmark validation note and its refresh triggers).
- Cloudflare deployment and startup evidence (separately tracked; see the
  validation note's refresh triggers — every flavour's web (edge) build is
  size-compatible with the current Workers Paid/Bundled (~3 MB) compressed-size limit at the
  time of measurement, and the `trading` flavour is built and tested end-to-end
  in CI (Workers Vitest), within the Workers compressed-size budget).

## Must Remain True

1. The crate builds both host tests and wasm-bindgen exports.
2. Host-compiled code stays in pure modules; wasm-bindgen and tsify remain in
   export modules.
3. Every cross-ABI input and output uses typed DTOs or typed callbacks.
4. Raw `JsValue` stays local to wasm exports and never becomes a public Rust SDK
   contract.
5. `OrderUid` and `OrderDigest` strings come from `to_hex_string()` (or
   `Display`), not byte re-encoding.
6. EIP-1271 signing resolves JavaScript callbacks at the facade boundary before
   storing a pure provider.
7. Provider objects stored behind Rust trait objects remain `Send + Sync`.
8. Timer handles use opaque JavaScript values so browser and Node runtimes both
   work.
9. Timer guards own and drop both the timeout handle and closure on every return
   path.
10. Callback dispatch awaits `Promise::resolve(...)` so sync and async callbacks
    share one semantic.
11. Request DTO construction preserves a live `AbortSignal` object across the
    JavaScript boundary.
12. `WasmError` messages use redacted display strings and response-body
    redaction.
13. Browser and edge consumers of a `web`-target flavor import the web build and
    call `initialize` once per module instance — browsers with no argument (the
    bundled module resolves through `new URL(import.meta.url)`), Workers with the
    precompiled module. The `node` condition loads the auto-initializing CommonJS
    build.
14. Worker source does not call dynamic WebAssembly compilation or streaming
    instantiation APIs.
15. Callback registries are scoped to a wasm module instance.
16. Package exports stay package-relative and avoid deep generated-file import
    paths.
17. Snapshot-gated TypeScript declarations remain the source of truth for the
    npm surface.
18. The wasm32 dependency tree excludes native Alloy transport, reqwest, and
    hyper.
19. Public declarations expose the TypeScript facade, not raw wasm-bindgen
    classes or callback registry handles.
20. Client constructors accept one typed config object and do not grow parallel
    free-function constructor families.
21. Package flavors stay explicit in the exports map and their declarations are
    snapshot-gated.
22. JavaScript transport policy configuration maps into the shared Rust
    `TransportPolicy` contract.
23. The phrase "canonical JavaScript and TypeScript surface" in this ADR refers to
    canonicality within cow-rs's WASM package, not to default-recommendation
    status for CoW Protocol TypeScript consumers; see the comparative
    benchmark validation note for the consumer-routing discipline.
24. The thrown error type is one `CowError` class that extends `Error`. Every
    error a wasm export throws is `instanceof CowError` (and `instanceof Error`),
    is keyed by `kind`, and exposes its per-kind fields — including the
    `__unknown` sentinel's `raw` — as enumerable own properties. A thrown error
    carries no `schemaVersion`; only the success envelope is version-tagged.
    `toJSON` and the static `CowError.fromJSON` round-trip the full field set
    across a `structuredClone` or worker boundary, and the facade exports
    `CowError`, `isCowError`, `normalizeError`, `isRetryable`, `retryAfterMs`,
    `isUserRejection`, and `withRetry` (the retry verdict is governed by
    [ADR 0060](0060-uniform-error-classification.md)).
25. The runtime known-error-kind registry is compile-checked against the
    `CowError` discriminant union, so the normalizer cannot drift from the
    declared error kinds.
26. A `kind: "orderbook"` error carries the services `errorType` wire tag (the
    sanitized protocol identifier — `"InsufficientAllowance"` vs
    `"InsufficientBalance"`) as the fine-grained partner of the coarse
    `category`, so a consumer can branch on the exact rejection without parsing
    the message. Only the sanitized tag crosses the boundary; the free-form
    services description never does.
27. Consumer-facing boundary type names mirror the native Rust type they
    project: no `Input` suffix is added where a native name exists, and
    request shapes follow the upstream `…Params` / `…Request` convention. The
    EIP-712 typed-data message type is exposed as `TypedDataMessage`.

## Alternatives Rejected

- Bundle viem, ethers, or wagmi inside the wasm package: easier examples, but
  it would turn a protocol SDK into a wallet-library opinion.
- Store raw `JsValue` or `js_sys::Function` in public Rust trait objects:
  shorter plumbing, but it would break the `Send + Sync` contract and make the
  account-abstraction boundary runtime-specific.
- Merge the surface into `cow-sdk-core`: fewer crates, but it would leak
  wasm-bindgen, TypeScript declaration, and JavaScript callback concerns into
  native consumers.

## Links

- [cow-sdk-js README](../../crates/js/README.md)
- [WASM Surface Audit](../audit/wasm-surface-audit.md)
- [Upstream TypeScript SDK](https://github.com/cowprotocol/cow-sdk)
- [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [tsify crate docs](https://docs.rs/tsify/latest/tsify/)

**Proven by:**

- [WASM Surface Audit](../audit/wasm-surface-audit.md)
