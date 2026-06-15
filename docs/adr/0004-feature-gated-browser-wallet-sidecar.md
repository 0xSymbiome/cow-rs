# ADR 0004: Feature-Gated Browser Wallet Sidecar

- Status: Superseded by [ADR 0007](0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md)
- Date: 2026-04-09
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: browser-wallet, wasm, feature-gating

## Superseded

The feature-gated browser-wallet sidecar decision — browser-wallet support lives
in its own `cow-sdk-browser-wallet` leaf behind an off-by-default feature, never
in the default facade closure — is now part of
[ADR 0007](0007-bounded-browser-wallet-support-and-current-browser-runtime-contract.md).
ADR 0007 records the bounded browser-wallet posture and the three-peer WASM leaf
map (`cow-sdk-browser-wallet`, `cow-sdk-transport-wasm`, `cow-sdk-wasm`), each
single-purpose and additive per the multi-crate-family growth rule in
[ADR 0001](0001-multi-crate-sdk-family-with-thin-facade.md).
