# ADR 0065: Single Canonical Browser-Wallet Example Replaces The WASM Console Genre

- Status: Accepted
- Date: 2026-06-03
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: examples, wasm, browser-wallet, proof-posture
- Related: [ADR 0004](0004-feature-gated-browser-wallet-sidecar.md), [ADR 0007](0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md), [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)
- Supersedes: ADR 0009

## Decision

The shipped WASM example surface is one canonical, runnable browser-wallet trade
example — `examples/wasm/cow-trader-dioxus/` — that drives the public `cow-sdk`
browser-wallet and trading contract end to end (discover, connect, sign, wrap,
approve, quote, swap) using only SDK public types. It replaces the former
multi-console "verification console" genre defined by ADR 0009.

This decision governs the browser-wallet trade example. The TypeScript-callable
`cow-sdk-wasm` package ships its own specialized consumer examples, governed
separately by [ADR 0039](0039-typescript-callable-wasm-sdk-surface.md) and the
[examples guide](../examples.md); they are not part of this single-example
surface.

## Why

The verification consoles were heavy proof scaffolding: a fixed console
template, a mock-versus-injected dual-pane UI, per-console Playwright lanes, a
hosted Pages deploy, and a dedicated proof-posture audit. That scaffolding
duplicated proof the SDK already owns. The reviewable browser-runtime contract
lives in the crate — `cow-sdk-browser-wallet` host-side and headless `wasm-pack`
tests, plus the chain-coherence and trust-posture audits — not in an example
surface. A single end-to-end example teaches the supported path more honestly
than a console dashboard and carries far less surface to keep current.

## Must Remain True

- Public surface: exactly one shipped browser-wallet trade example. It is a
  consumer demonstration of the public browser-wallet and trading contract, not a
  proof dashboard, and it uses only `cow-sdk` public types — no raw JavaScript and
  no raw RPC.
- Runtime and support: the example is a standalone workspace that path-depends
  into the SDK crates; it never widens the default facade or the crate contracts
  it demonstrates.
- Validation and review: the example is held to the crate quality bar — it
  builds for `wasm32-unknown-unknown` cleanly, stays clippy- and rustfmt-clean,
  and is gated in CI. Browser-runtime proof stays in the crate test lanes and
  the browser-wallet audits, not in the example.
- Cost: the example talks to the live orderbook, so it is a demonstration and a
  compile gate rather than a deterministic release gate.

## Alternatives Rejected

- Keep the verification-console genre: it duplicated crate-owned proof and grew
  a console, a Playwright lane, a Pages deploy, and an audit per capability.
- Ship no runnable browser example: a feature-gated browser-wallet crate with
  only unit tests leaves consumers without a working end-to-end reference.

## Links

- [Architecture](../architecture.md)
- [Examples](../examples.md)
- [Browser-Runtime Proof Posture](../browser-runtime-proof-posture.md)
- [ADR 0004](0004-feature-gated-browser-wallet-sidecar.md)
- [ADR 0007](0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- [ADR 0008](0008-additive-capability-expansion-through-leaf-crates-and-owned-sidecars.md)
- [ADR 0009](0009-wasm-verification-consoles-hybrid-extensibility-and-two-tier-proof.md) (superseded)
