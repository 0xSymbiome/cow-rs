# WASM Surface Audit

Status: Current
Last reviewed: 2026-06-15
Owning surface: `cow-sdk-wasm` TypeScript-callable wasm-bindgen crate, npm package layout, and JavaScript callback runtime boundary
Refresh trigger: Changes to `crates/wasm/src/**`, wasm-pack package exports, runtime support claims, wallet callback shapes, or the `JsCallbackHttpTransport` contract
Related docs:
- [ADR 0039](../adr/0039-typescript-callable-wasm-sdk-surface.md)
- [ADR 0040](../adr/0040-wallet-provider-callback-boundary-for-js-consumers.md)
- [ADR 0042](../adr/0042-pure-helpers-extraction.md)
- [ADR 0043](../adr/0043-callback-registry-internalization.md)
- [ADR 0044](../adr/0044-bundle-size-profile-and-flavor-builds.md)
- [ADR 0046](../adr/0046-transport-policy-js-exposure.md)
- [ADR 0047](../adr/0047-typescript-facade-architecture.md)
- [WASM Type Generation Audit](wasm-type-generation-audit.md)
- [WASM EIP-1271 Parity Audit](wasm-eip1271-parity-audit.md)
- [WASM Callback Shape Design Audit](wasm-callback-shape-design-audit.md)
- [WASM Public API Stability Audit](wasm-public-api-stability-audit.md)
- [WASM Facade Architecture Audit](wasm-facade-architecture-audit.md)
- [PROPERTIES.md](../../PROPERTIES.md)

## Scope

This audit covers:

- the four-layer `cow-sdk-wasm` public surface: pure helpers, wallet callbacks,
  orderbook/subgraph/IPFS clients, and trading clients
- the JavaScript callback HTTP transport and fetch-callback registry
- npm export-map support for browser, bundler, Node.js, and Cloudflare Workers
- the runtime support matrix and evidence claims attached to each runtime

It does not cover npm publication, package-name ownership, or live wallet
vendor compatibility outside the callback contract.

## Coverage

| Runtime | Support claim | Evidence |
| --- | --- | --- |
| Browser bundlers | `default-http-supported` | Playwright e2e against the browser fixture and wasm-bindgen tests |
| Node.js 22 and 24 LTS | `callback-http-tested` | Vitest coverage through the callback transport and nodejs package subpath |
| Cloudflare Workers | `callback-http-tested` | workerd fixture, `./cloudflare` plus `./cloudflare/wasm` subpaths, and forbidden dynamic-instantiation tests |
| Deno | `optional-experimental` | Runtime-neutral `CowFetchCallback`; self-built target only, no shipped build or CI fixture |
| Bun, Vercel Edge, Fly.io | `best-effort` | documented as unclaimed for CI support |

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Surface layering | Pure helpers stay host-safe in the `cow-sdk-wasm::helpers` module, while wasm-bindgen exports own JavaScript interop | Conforms |
| Wallet callbacks | Typed-data, EIP-1193, digest, and custom EIP-1271 callbacks are explicit and fail closed | Conforms |
| HTTP callbacks | `JsCallbackHttpTransport` owns timeout, abort signal, internal callback retention, and typed error mapping | Conforms |
| Event decoding | `decodeSettlementLog` and `decodeEthFlowLog` turn raw settlement and eth-flow logs into typed events with no network access and fail closed on malformed input | Conforms |
| Runtime packaging | Public imports use facade package exports; Cloudflare uses the web-target package subpaths | Conforms |
| Error posture | `WasmError` preserves typed redaction before diagnostics reach JavaScript | Conforms |
| Retry hint | The `orderbook` variant carries `retryable` plus an optional `retryAfterMs`, projecting the native retry verdict to JavaScript | Conforms |

## Current Contract

### Surface Layers

Layer 1 exposes deterministic helpers for chains, app-data, typed-data, UID,
digest, and EIP-1271 payload computation through the `cow-sdk-wasm::helpers` module, plus
the fail-closed, provider-free on-chain event-log decoders `decodeSettlementLog`
and `decodeEthFlowLog`, which reconstruct borrowed log bytes and dispatch to the
`cow-sdk-contracts` decoders without network access. Layer 2 exposes wallet and
signer callbacks. Layer 3 exposes orderbook, subgraph, and IPFS clients over
default or callback HTTP. Layer 4 exposes trading clients for quote and post
flows.

### Runtime Boundary

Browser bundlers may use the default browser fetch path. Node.js, Cloudflare
Workers, Deno, and custom runtimes pass HTTP through `CowFetchCallback` and
the `JsCallbackHttpTransport`. Callback retention is internal to the owning
client and scoped to one wasm module instance.

### TypeScript Facade

Public package imports resolve through compiled TypeScript facade modules. Raw
wasm-bindgen output remains a package-internal artifact and is not a public
import target.

### Cloudflare Contract

Cloudflare Workers consume the web-target glue through `./cloudflare` and the
precompiled wasm module through `./cloudflare/wasm`. Worker code does not call
dynamic WebAssembly compilation or streaming instantiation APIs.

### Open Questions

- Deno remains opt-in experimental until it is part of the default release
  validation set.
- Bun, Vercel Edge, and Fly.io remain best-effort until dedicated fixtures and
  CI evidence exist.

## Evidence

Primary implementation points:

- `crates/wasm/src/helpers/`
- `crates/wasm/src/exports/`
- `crates/wasm/npm/package.template.json`
- `crates/wasm/npm/scripts/build.sh`
- `crates/wasm/npm/src/`
- `crates/wasm/npm/scripts/verify-exports.mjs`
- `crates/wasm/npm/scripts/verify-no-raw-exports.mjs`
- `.github/workflows/wasm.yml`

Primary regression coverage:

- `crates/wasm/tests/host_pure_helpers.rs`
- `crates/wasm/tests/wasm_surface_contract.rs`
- `crates/wasm/tests/wasm_callback_contract.rs`
- `crates/wasm/tests/wasm_callback_transport_contract.rs`
- `crates/wasm/tests/wasm_facade_snapshot_contract.rs`
- `crates/wasm/tests/wasm_fail_closed_contract.rs`
- `crates/wasm/tests/wasm_redaction_contract.rs`
- `tests/wasm_dependency_invariant.rs`
- `e2e/wasm-typescript/tests/browser/browser.spec.ts`
- `e2e/wasm-typescript-cf/tests/forbidden-instantiation.spec.ts`

Validation surface:

```text
cargo test -p cow-sdk-wasm --test host_pure_helpers
cargo test -p cow-rs-workspace-tests --test wasm_dependency_invariant
wasm-pack test crates/wasm --headless --firefox
node crates/wasm/npm/scripts/verify-exports.mjs
node crates/wasm/npm/scripts/verify-no-raw-exports.mjs
pnpm --dir e2e/wasm-typescript test
pnpm --dir e2e/wasm-typescript-cf test
```
