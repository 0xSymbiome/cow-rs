# ADR 0007: Bounded Browser Wallet Support And Current Browser Runtime Contract

- Status: Accepted
- Date: 2026-04-13
- Last reviewed: 2026-05-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: browser-wallet, wasm, support-posture, interop
- Related: [ADR 0004](0004-feature-gated-browser-wallet-sidecar.md), [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md), [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md), [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md), [ADR 0040](0040-wallet-provider-callback-boundary-for-js-consumers.md)

## Decision

Keep browser wallet support explicit, feature-scoped, and compatibility-bounded,
and keep browser-runtime interop aligned to the current leaf-local
`wasm-bindgen` contract.

The leaf-local rule applies per WASM leaf. cow-rs supports two peer WASM
leaves: `cow-sdk-browser-wallet` for the Rust-native EIP-1193 wallet adapter
and `cow-sdk-wasm` for the TypeScript-callable surface used by JavaScript
consumers. Each leaf is single-purpose and additive per ADR 0008. The browser
`fetch` HTTP transport (`FetchTransport`) is not a separate leaf: it ships as a
target-gated module of `cow-sdk-core`, the `wasm32` sibling of the native
`ReqwestTransport` (ADR 0010).

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
- [Browser-Runtime Proof Posture](../browser-runtime-proof-posture.md)
- [ADR 0065](0065-canonical-browser-wallet-example.md)
- See also: ADR 0039 and ADR 0040.

**Proven by:**

- [Browser Wallet Alloy Dependency Audit](../audit/browser-wallet-alloy-dependency-audit.md)
- [Browser Wallet Chain Coherence Audit](../audit/browser-wallet-chain-coherence-audit.md)
- [Browser Wallet Trust Posture Audit](../audit/browser-wallet-trust-posture-audit.md)
- [WASM Browser Runner Determinism Audit](../audit/wasm-browser-runner-determinism-audit.md)
