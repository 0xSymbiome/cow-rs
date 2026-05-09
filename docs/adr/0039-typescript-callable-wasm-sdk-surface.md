# ADR 0039: Keep The TypeScript-Callable WASM SDK Surface As An Additive Leaf Crate

- Status: Accepted
- Date: 2026-05-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, typescript, public-surface, additive-leaf-crates
- Related: [ADR 0007](0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md), [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0013](0013-http-transport-injection-and-typestate-builders.md), [ADR 0019](0019-http-transport-sole-dispatch.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0029](0029-trait-evolution-extension-traits.md), [ADR 0037](0037-alloy-umbrella-adapter.md), [ADR 0038](0038-transaction-lifecycle-types.md)

## Decision

`cow-sdk-wasm` is the canonical TypeScript-callable SDK surface. It remains a
publishable additive leaf crate, not part of `cow-sdk-core`, and exposes four
layers: pure protocol helpers, wallet and signer callback functions, orderbook
plus subgraph plus IPFS clients, and trading clients. EIP-1271 signing uses a
facade-resolves-callback pattern: JavaScript resolves the final signature at
the wasm boundary, while Rust stores only a pure `Send + Sync` provider.

The runtime support matrix is explicit: browser bundlers are
`default-http-supported`, Node.js 24 LTS and Cloudflare Workers are
`callback-http-tested`, Deno is opt-in experimental, and Bun, Vercel Edge, and
Fly.io are best-effort without a CI claim.

## Why

JavaScript consumers already own their wallet library, event loop, fetch stack,
and deployment runtime. A callback boundary lets viem, ethers, wagmi, EIP-1193
wallets, Workers, Node, and Deno integrate without forcing any one JavaScript
ecosystem into the Rust crate. Keeping the surface as a leaf preserves the
native SDK dependency graph and keeps wasm-bindgen concerns local to the crate
that exports them.

## Must Remain True

1. The crate builds both host tests and wasm-bindgen exports.
2. Host-compiled code stays in pure modules; wasm-bindgen and tsify remain in
   export modules.
3. Every cross-ABI input and output uses typed DTOs or typed callbacks.
4. Raw `JsValue` stays local to wasm exports and never becomes a public Rust SDK
   contract.
5. `OrderUid` and `OrderDigest` strings come from `as_str()`, not byte
   re-encoding.
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
13. Cloudflare imports the web-target package subpaths and initializes the
    module once per isolate.
14. Worker source does not call dynamic WebAssembly compilation or streaming
    instantiation APIs.
15. Callback registries are scoped to a wasm module instance.
16. Package exports stay package-relative and avoid deep generated-file import
    paths.
17. Snapshot-gated TypeScript declarations remain the source of truth for the
    npm surface.
18. The wasm32 dependency tree excludes browser-wallet, native Alloy transport,
    reqwest, and hyper.

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

- [cow-sdk-wasm README](../../crates/wasm/README.md)
- [WASM Surface Audit](../audit/wasm-surface-audit.md)
- [WASM Type Generation Audit](../audit/wasm-type-generation-audit.md)
- [Upstream TypeScript SDK](https://github.com/cowprotocol/cow-sdk)
- [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/)
- [tsify crate docs](https://docs.rs/tsify/latest/tsify/)

**Proven by:**

- [WASM Surface Audit](../audit/wasm-surface-audit.md)
- [WASM Type Generation Audit](../audit/wasm-type-generation-audit.md)
