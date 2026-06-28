# ADR 0040: Keep Wallet And Provider Interop Behind Typed JavaScript Callbacks

- Status: Accepted
- Date: 2026-05-09
- Last reviewed: 2026-06-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: wasm, wallet, provider, callback-boundary, eip1271
- Related: [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), ADR 0007 (superseded by 0039/0040), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), ADR 0043 (superseded by 0039), [ADR 0045](0045-async-signer-trait-narrowing.md)

## Decision

JavaScript wallet and HTTP runtime interop crosses `cow-sdk-js` through five
typed callback shapes: `TypedDataSignerCallback`, `DigestSignerCallback`,
`CustomEip1271Callback`, `ContractReadCallback`, and `CowFetchCallback`. Signing
exposes one entry per on-wire scheme — `TypedDataSignerCallback` for `eip712`
(the primary path), `DigestSignerCallback` for `ethsign`, and
`signOrderWithCustomEip1271` (`CustomEip1271Callback`) for smart-account
`eip1271`. The SDK ships no raw EIP-1193 request signer: a host that holds only
an EIP-1193 provider wraps it into `TypedDataSignerCallback` through
`eth_signTypedData_v4`, keeping the provider-protocol detail at the host edge.
Scheme fallback — a wallet that rejects typed data — is host-owned: the host
chooses the entry point, falling back to the `eth_sign` digest entry for legacy
or hardware wallets.

Callback dispatch uses `Promise::resolve` so plain return values, native
Promises, and thenables share the JavaScript `await` semantic. SDK-owned
timeouts use `globalThis.AbortController`; the request DTO is assembled with
`Reflect::set` so `request.signal` remains a live `AbortSignal`. Per-call
options carry `signal` and `timeoutMs`, and signing methods carry
`walletConfig.timeoutMs` for wallet-owned requests. The fetch callback registry
is internal to the owning client and is not exposed through public handle
types.

## Why

Wallet and provider ecosystems move faster than the Rust SDK's stable public
API. Typed callbacks keep the Rust contract language-agnostic while letting the
host application own its wallet stack — a viem or ethers typed-data signer, an
EIP-1193 provider it wraps, a smart-account client, custom fetch, or a service
worker. Naming one callback per signing operation, rather than baking a specific
provider protocol such as `eth_signTypedData_v4` request-shaping into the SDK,
keeps that fast-moving detail at the host edge. The SDK still owns timeout, error
typing, ECDSA recovery-byte normalization, and redaction before values cross the
public error envelope.

## Must Remain True

- Public surface: the callback names and payloads remain typed and documented;
  raw wallet-library objects do not become SDK-owned Rust types. Signing exposes
  one entry per on-wire scheme (`eip712`, `ethsign`, `eip1271`) and no raw
  EIP-1193 request signer; scheme fallback is host-owned.
- Runtime and support: ECDSA signatures normalize to legacy `27` / `28`
  recovery bytes; callback results are awaited through `Promise::resolve`;
  `AbortSignal` is passed by reference, not serialized.
- Validation and review: registry state is local to a wasm module instance and
  hidden from public declarations; callback throws, rejects, malformed
  responses, timeouts, and aborts map to typed `WasmError` variants.
- Cleanup: callback retention, abort listeners, and timeout handles are dropped
  on success and failure paths.
- Cost: hosts must provide callbacks explicitly instead of receiving a bundled
  wallet adapter, but the SDK avoids freezing one JavaScript provider stack.

## Alternatives Rejected

- Bundle a default JavaScript wallet library: convenient, but it creates a
  dependency and compatibility promise the Rust SDK should not own.
- Expose only raw EIP-1193 requests: flexible, but it would make typed-data,
  digest, and EIP-1271 flows less reviewable.
- Keep a dedicated raw EIP-1193 request signer alongside the typed-data signer:
  it duplicates the `eip712` scheme behind a second shape and bakes
  `eth_signTypedData_v4` request-shaping into the SDK — the host-edge concern
  this ADR keeps out. A host with only a provider wraps it into the typed-data
  callback instead.
- Store callback functions inside Rust trait objects: compact, but it would
  couple `Send + Sync` Rust traits to JavaScript runtime handles.

## Links

- [cow-sdk-js README](../../crates/js/README.md)
- [Integrations](../integrations.md)
- [WASM Surface Audit](../audit/wasm-surface-audit.md)
- [EIP-1271 Verification Cache Audit](../audit/eip1271-verification-cache-audit.md)
- [wasm-bindgen guide](https://rustwasm.github.io/docs/wasm-bindgen/)

**Proven by:**

- [WASM Surface Audit](../audit/wasm-surface-audit.md)
- [EIP-1271 Verification Cache Audit](../audit/eip1271-verification-cache-audit.md)
