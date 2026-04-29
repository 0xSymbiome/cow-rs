# ADR 0007: Bounded Browser Wallet Support And Current Browser Runtime Contract

- Status: Accepted
- Date: 2026-04-13
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: browser-wallet, wasm, support-posture, interop
- Related: [ADR 0004](0004-feature-gated-browser-wallet-sidecar.md), [ADR 0005](0005-boundary-specific-runtime-contracts-and-strong-domain-types.md)

## Decision

Keep browser wallet support explicit, feature-scoped, and compatibility-bounded,
and keep browser-runtime interop aligned to the current leaf-local
`wasm-bindgen` contract.

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
  leaf-local to `cow-sdk-browser-wallet`. Raw `JsValue` and browser-global
  types do not become the public SDK contract.
- Validation and review: direct browser-targeted proof, higher-level browser
  automation, and optional environment-sensitive confirmation stay separate
  lanes with distinct claims.
- Cost: browser support requires extra proof, tighter wording, and slower,
  more deliberate expansion than a permissive generic adapter surface.

## Alternatives Rejected

- Claim universal injected-wallet compatibility from one reviewed path: too
  broad for the real variability of browser providers.
- Expose generic JS passthrough as the main extension surface: flexible in the
  short term, but it weakens the Rust contract and reviewability.

## Links

- [Architecture](../architecture.md)
- [Verification Matrix](../verification-matrix.md)
- [Validation Scope](../validation-scope.md)
- [Verification Guide](../verification-guide.md)
- [Browser-Runtime Proof Posture](../browser-runtime-proof-posture.md)
- [ADR 0009](0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md)

**Proven by:**

- [Browser Wallet Alloy Dependency Audit](../audit/browser-wallet-alloy-dependency-audit.md)
- [Browser Wallet Chain Coherence Audit](../audit/browser-wallet-chain-coherence-audit.md)
- [Browser Wallet Trust Posture Audit](../audit/browser-wallet-trust-posture-audit.md)
- [WASM Browser Runner Determinism Audit](../audit/wasm-browser-runner-determinism-audit.md)
- [WASM Example Proof-Posture Audit](../audit/wasm-example-proof-posture-audit.md)
