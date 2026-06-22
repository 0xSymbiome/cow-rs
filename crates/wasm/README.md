# cow-sdk-wasm

The TypeScript-callable WebAssembly layer of the `cow-rs` SDK. It compiles the
same deterministic Rust protocol logic the native crates run — order signing,
EIP-712 / EIP-1271 envelope construction, UID packing, app-data hashing,
event-log decoding, and the orderbook, subgraph, IPFS, and trading clients — to
`wasm32` and exposes it to JavaScript and TypeScript through typed DTOs and
explicit callbacks. It wraps the existing `cow-sdk-core`, `cow-sdk-contracts`,
`cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-subgraph`,
and `cow-sdk-trading` crates rather than reimplementing them, so a signature a
browser produces is byte-identical to the one a Rust service produces.

This README is the **engineering face** of the crate — how the binding is built
and why it holds. **Consumer documentation — import selection, quickstarts, and
runtime support — is the published npm package README**
([`@symbiome-forge/cow-sdk-wasm`](https://www.npmjs.com/package/@symbiome-forge/cow-sdk-wasm)).
The crate is a `wasm-bindgen` `cdylib` published only to npm; Rust consumers use
the [`cow-sdk`](https://crates.io/crates/cow-sdk) facade on a `wasm32` target
rather than depending on this crate directly.

## Surface Layers

| Layer | Surface | Purpose |
| --- | --- | --- |
| Pure helpers | `domainSeparator`, `orderTypedData`, `computeOrderUid`, app-data helpers, deployment helpers, supported-chain helpers, `wrappedNativeToken` | Deterministic protocol output without JavaScript runtime state |
| Wallet callbacks | typed-data signer, digest signer, custom EIP-1271 callbacks, cancellation signing | Host-owned wallet integration through typed callback shapes |
| Clients | orderbook, subgraph, IPFS, and callback-fetch clients | CoW service access through default browser fetch or callback HTTP |
| Trading facade | quote and post clients, including EIP-1271 posting, plus native `buildWrapTx` / `buildUnwrapTx` transaction builders | Higher-level trading flows over the same DTO and callback boundary |

## Callback Shapes

The public callback boundary names the host responsibilities explicitly:

- `TypedDataSignerCallback` signs canonical EIP-712 typed-data payloads. A raw
  EIP-1193 provider wraps into this callback through `eth_signTypedData_v4`.
- `DigestSignerCallback` signs raw digests for explicit EthSign flows.
- `CustomEip1271Callback` returns the final smart-account EIP-1271 signature.
- `ContractReadCallback` runs a read-only `eth_call` (the trading flavour's
  allowance read) and returns the ABI-decoded value.
- `CowFetchCallback` dispatches HTTP for Node.js, Workers, Deno, and custom
  runtimes.

Callback returns are normalized through JavaScript `await` semantics, so a
callback may return either a plain value, a native Promise, or a thenable.

## EIP-1271 Smart-Account Pattern

`signOrderWithCustomEip1271` calls JavaScript at the wasm facade boundary and
expects the callback to resolve the final ABI-encoded EIP-1271 signature. Rust
then wraps that string in a pure resolved provider. No JavaScript function or
`JsValue` is stored inside the Rust EIP-1271 provider trait object, preserving
the same `Send + Sync` trait shape used by native consumers.

## HTTP Runtime Model

Browser bundlers may use the default fetch-backed client. Non-window runtimes
use `JsCallbackHttpTransport`, which implements the same
`cow_sdk_core::HttpTransport` trait as native and browser adapters but delegates
wire I/O to a `CowFetchCallback`.

The callback request includes method, URL, headers, optional body, timeout, and
a live `AbortSignal`. Timeout remains SDK-owned through
`globalThis.AbortController`, and `TimerGuard` owns both the opaque timeout
handle and timeout closure so cleanup happens on success, throw, rejection,
malformed response, or abort.

## Browser, edge, and the explicit initialize

Every flavour's `browser`, `import`, and `default` export conditions, its edge
conditions (`workerd`, `deno`, `edge-light`, `bun`), and its explicit `…/edge`
subpath all resolve to the web-target build. wasm-bindgen's bundler-target ESM
integration (`import * as wasm from "./…_bg.wasm"`) is not portable across bundlers
— unsupported on several and webpack-first elsewhere — so the web build is the
browser default for every flavour. A browser caller runs `initialize()` once; its
loader resolves the bundled module through `new URL(import.meta.url)`, the universal
asset path. Workers cannot compile WebAssembly from bytes at runtime, so the `…/edge`
subpath takes a statically imported `WebAssembly.Module` through `initialize(module)`
instead, paired with the precompiled module at `…/edge/wasm`. The `node` condition
keeps the auto-initializing CommonJS build.

Every flavour also ships the wasm-bindgen source-phase build at `…/module`
(`--target module`, TC39 source-phase imports / Wasm ESM Integration), driven
directly because wasm-pack does not emit it. It auto-instantiates synchronously on
import with no `initialize()` call — the standards-track forward path as browser
bundlers adopt source-phase — and is opt-in today (Node 24, Deno, esbuild).

Each flavour emits one wasm binary across its bundler, Node, web, and module
targets — the web glue's default loader URL and the module glue's `import source`
both repoint at the single bundler copy. The release pipeline enforces a per-build
gzip byte budget against the Cloudflare Workers Paid/Bundled (~3 MB) compressed-size limit, and
a `workers-vitest` job runs the Cloudflare end-to-end suite under
`@cloudflare/vitest-pool-workers`. The consumer-facing runtime-support matrix and
quickstarts are in the npm package README.

## TypeScript Declarations

All DTOs that cross the wasm ABI are represented in TypeScript declarations.
The committed declaration snapshots under `crates/wasm/snapshots/` are an
**API-lock**: CI diffs them on every build, so any change to the public
TypeScript contract surfaces as a reviewed diff rather than a silent drift. This
is the wasm/TypeScript analog of the Rust ecosystem's `cargo-public-api`
snapshot pattern (and of TypeScript's API Extractor report files). The two
snapshot layers lock complementary surfaces and are not redundant:

- `snapshots/raw/` (bundler and nodejs targets) locks the wasm-bindgen output
  and the `tsify` DTO **fields** — the field-level shape a consumer sees through
  the re-exported DTO types. Every flavour's web and source-phase `module` targets
  add only wasm-bindgen's standard target scaffolding on top of the bundler surface,
  so they are not snapshotted separately; the facade snapshot pins their public
  `initialize` contract.
- `snapshots/facade/` locks the public **class and function surface** of the
  TypeScript facade — method signatures, option objects, and disposal.

A third contract, `wasm_facade_coverage_contract.rs`, locks the **relationship**
between the two layers: every public raw symbol — client method, free function, and
DTO type — must be exposed by the facade or carry a reasoned allowlist entry. Adding
a `#[wasm_bindgen]` export without wiring it through `crates/wasm/npm/src/*.ts` then
fails the build, so a binding can never reach the raw layer yet be absent from the
published facade.

Committing generated declarations is a deliberate choice: pure-`wasm-bindgen`
projects typically regenerate them, but locking the published contract matches
this SDK's parity-and-stability goal. The package export verification script
additionally checks that every exported target exists and that declaration files
carry the required TypeScript library references.

## Error Contract

The Rust `WasmError` discriminated union projects to JavaScript as a single
`CowError` class — a real `Error` subclass keyed by `kind` that consumers catch,
narrow with the exported `isCowError`, and `switch` on. Transport, app-data,
signing, orderbook, subgraph, trading, wallet, cancellation, and internal
failures keep low-cardinality fields visible while preserving the SDK's redaction
posture for URLs, headers, response bodies, and secret-shaped details. A thrown
error carries no `schemaVersion`; only the success envelope is version-tagged.

The `orderbook` variant additionally carries the services `errorType` wire tag
(`"InsufficientAllowance"` vs `"InsufficientBalance"`, the fine-grained partner of
the coarse `category`), a `retryable` boolean, and an optional `retryAfterMs` backoff
hint parsed from the response `Retry-After` header, mirroring the native
`OrderbookError::is_retryable` and `backoff_hint` accessors. The facade exports
`isRetryable`, `retryAfterMs`, `isUserRejection` (a declined-signature / cancellation
guard), and a `withRetry` helper so a JavaScript consumer driving its own retry loop
reaches the same verdict as the Rust core.

## Where To Next

- [Getting Started](https://github.com/0xSymbiome/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/0xSymbiome/cow-rs/blob/main/docs/integrations.md)
- [Architecture Overview](https://github.com/0xSymbiome/cow-rs/blob/main/docs/architecture.md)
- [WASM Surface Audit](https://github.com/0xSymbiome/cow-rs/blob/main/docs/audit/wasm-surface-audit.md)
- [EIP-1271 Verification Cache Audit](https://github.com/0xSymbiome/cow-rs/blob/main/docs/audit/eip1271-verification-cache-audit.md)

## License

Licensed under GPL-3.0-or-later. See the workspace
[LICENSE](https://github.com/0xSymbiome/cow-rs/blob/main/LICENSE)
file for the full text.
