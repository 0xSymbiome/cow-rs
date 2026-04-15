# ADR 0004: Feature-Gated Browser Wallet Sidecar

- Status: Accepted
- Date: 2026-04-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: browser-wallet, wasm, feature-gating
- Related: [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)

## Decision

Implement browser wallet support in `cow-sdk-browser-wallet` and expose it from
`cow-sdk` only behind the `browser-wallet` feature.

## Why

Browser wallets use injected, async EIP-1193 providers and a WASM runtime model
that does not fit the native-first default SDK surface. The browser path needs
its own dependency and runtime boundary.

## Must Remain True

- Public surface: browser wallet support is explicit and feature-scoped instead
  of being part of the default `cow-sdk` contract.
- Runtime and support: browser-only dependencies and async provider behavior
  stay isolated in `cow-sdk-browser-wallet`, which keeps native defaults lean
  and support claims bounded.
- Validation and review: browser-targeted proof can stay separate from native validation
  and from environment-sensitive wallet confirmation.
- Cost: browser consumers need an explicit feature or a direct dependency on
  the sidecar crate.

## Alternatives Rejected

- Treat browser support as raw private-key handling inside examples: unsafe and
  not representative of browser wallet workflows.
- Add browser globals and wallet shims directly to `cow-sdk`: widens the root
  facade and leaks browser-only dependencies into default builds.

## Links

- [Architecture](../architecture.md)
- [Verification Guide](../verification-guide.md)
- [ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md)
- [Browser Wallet Chain Coherence Audit](../audit/browser-wallet-chain-coherence-audit.md)
