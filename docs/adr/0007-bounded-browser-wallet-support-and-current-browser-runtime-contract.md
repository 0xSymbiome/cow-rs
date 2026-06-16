# ADR 0007: Bounded Browser Wallet Support And Current Browser Runtime Contract

- Status: Superseded by ADR 0039 and ADR 0040
- Date: 2026-04-13
- Last reviewed: 2026-05-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: browser-wallet, wasm, support-posture, interop
- Related: [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0040](0040-wallet-provider-callback-boundary-for-js-consumers.md)

> Superseded: the bounded native browser-wallet crate this ADR governed has
> been retired. Browser/wallet integration for JavaScript and TypeScript
> consumers is now served by the `cow-sdk-wasm` package plus the host app's own
> wallet stack (viem, wagmi, or any EIP-1193 provider): the EIP-1193
> request-callback boundary lives in [ADR 0040](0040-wallet-provider-callback-boundary-for-js-consumers.md)
> and the TypeScript-callable wasm surface in [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md).
> The decision below is retained as design history.

## Decision

Keep browser wallet support explicit, feature-scoped, and compatibility-bounded,
and keep browser-runtime interop aligned to the current leaf-local
`wasm-bindgen` contract.

The leaf-local rule applies per WASM leaf. Browser/wallet integration is owned
by `cow-sdk-wasm`, the TypeScript-callable surface used by JavaScript consumers,
which exposes the EIP-1193 request-callback boundary (ADR 0040); the host app
supplies the wallet connection (viem, wagmi, or any EIP-1193 provider). The
browser `fetch` HTTP transport (`FetchTransport`) is not a separate leaf: it
ships as a target-gated module of `cow-sdk-core`, the `wasm32` sibling of the
native `ReqwestTransport` (ADR 0010).

## Why

Browser wallets are injected async runtimes with material provider variance.
Treating them as a universal support claim or exposing raw JS escape hatches as
the public contract would weaken both safety and credibility. The owned browser
runtime seam also needs a current, reviewable interop pattern instead of
carrying compatibility-era behavior indefinitely.

## Must Remain True

- Public surface: browser wallet support remains optional and explicit. Support
  claims use bounded compatibility language, and wallet method growth stays
  typed rather than widening into a generic raw RPC surface.
- Runtime and support: browser-only dependencies, Promise handling,
  `serde-wasm-bindgen`, typed JS imports, and bridge-local interop remain
  leaf-local to their owning WASM crates. Raw `JsValue` and browser-global
  types do not become the public SDK contract.
- Validation and review: direct browser-targeted proof, higher-level browser
  automation, and optional environment-sensitive confirmation stay separate
  lanes with distinct claims.
- Cost: each WASM leaf requires extra proof, tighter wording, and slower, more
  deliberate expansion than a permissive generic adapter surface.

## Alternatives Rejected

- Claim universal injected-wallet compatibility from one reviewed path: too
  broad for the real variability of browser providers.
- Expose generic JS passthrough as the main extension surface: flexible in the
  short term, but it weakens the Rust contract and reviewability.

## Links

- [Architecture](../architecture.md)
- [Verification Matrix](../verification.md)
- [Validation Scope](../verification.md)
- [Verification Guide](../verification.md)
- See also: ADR 0039 and ADR 0040.
