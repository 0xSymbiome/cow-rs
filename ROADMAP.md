# Roadmap

This roadmap describes the planned public capability sequence for `cow-rs`.
It is subject to change based on consumer feedback, upstream CoW Protocol
changes, and review findings.

## Initial SDK Foundation

The initial SDK foundation focuses on the existing ten-crate SDK family:
`cow-sdk`, `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`,
`cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-trading`,
`cow-sdk-subgraph`, `cow-sdk-browser-wallet`, and
`cow-sdk-transport-wasm`.

This work prioritizes a clean trading-first foundation: typed orderbook DTOs,
typed signing and contract helpers, deployment registry provenance, cooperative
cancellation, provider-neutral runtime seams, browser wallet support, and
deterministic native and WASM examples.

## Next Planned Capabilities

The next planned capability group adds opt-in alloy adapter crates:
`cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, and the `cow-sdk-alloy`
umbrella crate. These crates are planned as additive adapters over the existing
provider and signer traits, keeping the default SDK provider-neutral.

This group also includes `cow-shed` integration, depending on implementation
capacity and review scope.

## Composable Order Capabilities

The following capability group is planned around two parallel tracks:

- composable orders, including TWAP support
- an EIP-2612 permit signing wrapper for typed permit calldata and hook
  metadata

These tracks are intended to remain modular so each can ship without forcing
unrelated application changes.

## Cross-Chain Capabilities

Cross-chain bridging is planned after the account-abstraction and composable
order foundations are in place. The intended shape is a typed bridge-provider
abstraction plus a first provider integration that can quote and build bridge
intents through the SDK.

## Later Capabilities

Later releases are reserved for advanced capabilities and polish, including:

- flash-loan helpers
- weiroll command support
- hardware-wallet implementations for Ledger and Trezor on the alloy signer
  adapter
- additional ergonomics, documentation, and integration polish

## Stability Promise

`cow-rs` is not yet released. Public API stability commitments begin with the
first published SDK release. Until then, the surface may change as it is
validated by real consumers.

Once the SDK reaches a stable release, it will follow semantic versioning for
the public API.
