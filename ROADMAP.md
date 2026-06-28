# Roadmap

This roadmap describes the planned public capability sequence for `cow-rs`.
It is subject to change based on consumer feedback, upstream CoW Protocol
changes, and review findings.

## Initial SDK Foundation

The initial SDK foundation focuses on the existing SDK family:
`cow-sdk`, `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`,
`cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-trading`,
and `cow-sdk-subgraph`, plus the opt-in native Alloy adapter crates
`cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, and `cow-sdk-alloy`,
and the JavaScript and TypeScript `cow-sdk-js` bindings.

This work prioritizes a clean trading-first foundation: typed orderbook DTOs,
typed signing and contract helpers, deployment registry provenance, cooperative
cancellation, provider-neutral runtime seams, a typed callback
boundary for host-supplied wallets, opt-in native Alloy provider and signer
support, and deterministic native and WASM examples.

## In-Flight Account-Abstraction And Composable Capabilities

COW Shed account-abstraction hooks are shipped behind the off-by-default
`cow-shed` feature (proxy derivation, EIP-712 hook signing, factory calldata,
the `CowShedHooks` orchestrator), bound to the deployed v1.0.x generation;
composable-order support remains in preparation. The shipped readiness
surface includes source pins, deployment provenance, schema v2 registry and
coverage taxonomy, COW Shed proxy bytecode evidence, reserved crate
manifests, ABI excerpts, parity fixtures, and audit records.

## WebAssembly Component Distribution Channel

An experimental second WASM distribution lane has landed as the additive
`cow-sdk-component` leaf crate, parallel to the wasm-bindgen npm lane. It
compiles the deterministic SDK core to `wasm32-wasip2` as a WebAssembly
Component against a typed WIT contract, so one audited Rust source is consumable
from polyglot hosts (JavaScript and TypeScript through jco, native hosts through
Wasmtime, and composition through the Component Model). The crate is a workspace
member that CI builds for `wasm32-wasip2`; it is `publish = false` and never goes
to crates.io. Per [ADR 0071](docs/adr/0071-wasm-component-distribution-channel.md)
(Proposed), the crate and its WIT contract are experimental (`0.x`): the
distribution and cross-runtime parity machinery — OCI and GitHub Release
publishing, jco and Wasmtime execution, and a WIT snapshot gate — is not yet
built, so today CI only compiles the component.

## Composable Order Capabilities

The following capability group continues around two parallel tracks:

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
- hardware-wallet implementations for Ledger and Trezor on the Alloy signer
  adapter
- additional ergonomics, documentation, and integration polish

## Stability Promise

`cow-rs` is not yet released. Public API stability commitments begin with the
first published SDK release. Until then, the surface may change as it is
validated by real consumers.

Once the SDK reaches a stable release, it will follow semantic versioning for
the public API.
