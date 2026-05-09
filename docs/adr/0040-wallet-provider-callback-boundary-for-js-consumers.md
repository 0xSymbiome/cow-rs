# ADR 0040: Keep Wallet And Provider Interop Behind Typed JavaScript Callbacks

- Status: Accepted
- Date: 2026-05-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, wallet, provider, callback-boundary, eip1271
- Related: [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0007](0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0029](0029-trait-evolution-extension-traits.md)

## Decision

JavaScript wallet and HTTP runtime interop crosses `cow-sdk-wasm` through five
typed callback shapes: `TypedDataSignerCallback`, `Eip1193RequestCallback`,
`DigestSignerCallback`, `CustomEip1271Callback`, and `CowFetchCallback`.
`signOrderWithCustomEip1271` is the smart-account entry point for callers that
need custom contract-wallet behavior.

Callback dispatch uses `Promise::resolve` so plain return values, native
Promises, and thenables share the JavaScript `await` semantic. SDK-owned
timeouts use `globalThis.AbortController`; the request DTO is assembled with
`Reflect::set` so `request.signal` remains a live `AbortSignal`. The fetch
callback registry is scoped to one wasm module instance, reserves zero, and
uses idempotent handle disposal.

## Why

Wallet and provider ecosystems move faster than the Rust SDK's stable public
API. Typed callbacks keep the Rust contract language-agnostic while letting
the host application decide whether a request goes through EIP-1193, a smart
account client, custom fetch, or a service worker. The SDK still owns timeout,
error typing, ECDSA recovery-byte normalization, and redaction before values
cross the public error envelope.

## Must Remain True

- Public surface: the callback names and payloads remain typed and documented;
  raw wallet-library objects do not become SDK-owned Rust types.
- Runtime and support: ECDSA signatures normalize to legacy `27` / `28`
  recovery bytes; callback results are awaited through `Promise::resolve`;
  `AbortSignal` is passed by reference, not serialized.
- Validation and review: registry handles are local to a wasm module instance;
  disposed handles fail with a typed configuration error; callback throws,
  rejects, malformed responses, and aborts map to typed `WasmError` variants.
- Cost: hosts must provide callbacks explicitly instead of receiving a bundled
  wallet adapter, but the SDK avoids freezing one JavaScript provider stack.

## Alternatives Rejected

- Bundle a default JavaScript wallet library: convenient, but it creates a
  dependency and compatibility promise the Rust SDK should not own.
- Expose only raw EIP-1193 requests: flexible, but it would make typed-data,
  digest, and EIP-1271 flows less reviewable.
- Store callback functions inside Rust trait objects: compact, but it would
  couple `Send + Sync` Rust traits to JavaScript runtime handles.

## Links

- [cow-sdk-wasm README](../../crates/wasm/README.md)
- [Integrations](../integrations.md)
- [WASM EIP-1271 Parity Audit](../audit/wasm-eip1271-parity-audit.md)
- [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/)

**Proven by:**

- [WASM Surface Audit](../audit/wasm-surface-audit.md)
- [WASM EIP-1271 Parity Audit](../audit/wasm-eip1271-parity-audit.md)

