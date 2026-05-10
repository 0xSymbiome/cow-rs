# cow-sdk-wasm

TypeScript-callable wasm-bindgen bindings for the CoW Protocol Rust SDK. The
crate exposes deterministic Rust protocol logic to JavaScript and TypeScript
through typed DTOs, package export subpaths, and explicit callbacks for wallet,
signer, smart-account, and HTTP runtime behavior.

The crate is a peer leaf of the native Rust facade. It wraps existing
`cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`,
`cow-sdk-orderbook`, `cow-sdk-subgraph`, and `cow-sdk-trading` helpers instead
of reimplementing protocol primitives.

## Install

The npm package name is selected at publication time and rendered into the
package manifest by the package build script.

```bash
npm install <published-cow-sdk-wasm-package>
```

For Rust crate consumers:

```toml
[dependencies]
cow-sdk-wasm = "0.1"
```

## Surface Layers

| Layer | Surface | Purpose |
| --- | --- | --- |
| Pure helpers | `domainSeparator`, `orderTypedData`, `computeOrderUid`, app-data helpers, deployment helpers, supported-chain helpers | Deterministic protocol output without JavaScript runtime state |
| Wallet callbacks | typed-data signer, EIP-1193 request, digest signer, custom EIP-1271 callbacks, cancellation signing | Host-owned wallet integration through typed callback shapes |
| Clients | orderbook, subgraph, IPFS, and callback-fetch clients | CoW service access through default browser fetch or callback HTTP |
| Trading facade | quote and post clients, including EIP-1271 posting | Higher-level trading flows over the same DTO and callback boundary |

## Callback Shapes

The public callback boundary names the host responsibilities explicitly:

- `TypedDataSignerCallback` signs canonical EIP-712 typed-data payloads.
- `Eip1193RequestCallback` lets an injected or hosted provider answer
  EIP-1193 requests.
- `DigestSignerCallback` signs raw digests for explicit EthSign flows.
- `CustomEip1271Callback` returns the final smart-account EIP-1271 signature.
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

## Runtime Support

| Runtime | Support claim | HTTP transport |
| --- | --- | --- |
| Browser bundlers (Vite, webpack, Next.js, Rollup, Parcel, esbuild) | `default-http-supported` | Default browser fetch |
| Node.js 24 LTS | `callback-http-tested` | `CowFetchCallback` |
| Cloudflare Workers (workerd) | `callback-http-tested` | `CowFetchCallback` through `./cloudflare` and `./cloudflare/wasm` |
| Deno | `optional-experimental` | `CowFetchCallback`, built only when the Deno target is enabled |
| Bun, Vercel Edge, Fly.io | `best-effort` | No CI support claim |

Cloudflare Workers use the web-target package output through the package export
map. Consumers should import public subpaths such as `./cloudflare` and
`./cloudflare/wasm`; nested build-output paths are not public API.

## TypeScript Declarations

All DTOs that cross the wasm ABI are represented in TypeScript declarations.
The committed declaration snapshots for the web, bundler, and nodejs targets
live under `crates/wasm/snapshots/raw/` and are compared during validation so
export drift is visible. The package export verification script also checks
that every exported target exists and that declaration files carry the required
TypeScript library references.

## Error Contract

JavaScript-visible errors use a typed `WasmError` discriminated union. Transport,
app-data, signing, orderbook, subgraph, trading, wallet, cancellation, and
internal failures keep low-cardinality fields visible while preserving the SDK's
redaction posture for URLs, headers, response bodies, and secret-shaped details.

## Where To Next

- [Getting Started](https://github.com/cowdao-grants/cow-rs/blob/main/docs/getting-started.md)
- [Integrations Guide](https://github.com/cowdao-grants/cow-rs/blob/main/docs/integrations.md)
- [Architecture Overview](https://github.com/cowdao-grants/cow-rs/blob/main/docs/architecture.md)
- [WASM Surface Audit](https://github.com/cowdao-grants/cow-rs/blob/main/docs/audit/wasm-surface-audit.md)
- [WASM Type Generation Audit](https://github.com/cowdao-grants/cow-rs/blob/main/docs/audit/wasm-type-generation-audit.md)
- [WASM EIP-1271 Parity Audit](https://github.com/cowdao-grants/cow-rs/blob/main/docs/audit/wasm-eip1271-parity-audit.md)

## License

Licensed under GPL-3.0-only. See the workspace
[LICENSE](https://github.com/cowdao-grants/cow-rs/blob/main/LICENSE)
file for the full text.
