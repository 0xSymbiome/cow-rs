# Changelog

All notable changes to `cow-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.5] - 2026-06-18

### Features

- *(wasm)* Harden and document the WebAssembly consumer surface (#9) ([`aff75d8`](https://github.com/0xSymbiome/cow-rs/commit/aff75d876cfb788e400476991b3ca4fe9048209b))

## [0.1.0-alpha.4] - 2026-06-18

### Bug Fixes

- *(release)* Re-lock the fuzz crate on release and format the version-surface module ([`e4a8798`](https://github.com/0xSymbiome/cow-rs/commit/e4a8798fb085b430b0eba3a61d6261fbadcd177b))

### Features

- [**breaking**] *(wasm)* Serve the trading flavour on bundler, nodejs, and web targets (#8) ([`3140635`](https://github.com/0xSymbiome/cow-rs/commit/3140635584690c35c07ec87d2341f850abe22522))

## [0.1.0-alpha.3] - 2026-06-17

### Bug Fixes

- *(wasm)* Realign the example and e2e projects with the renamed package and pnpm 11.7.0 ([`bed6bcb`](https://github.com/0xSymbiome/cow-rs/commit/bed6bcb340bcd358c4aca88f6c03ef7e8674c5e7))
- *(ci)* Clear the post-release version-alignment and lint gates ([`d6cc19e`](https://github.com/0xSymbiome/cow-rs/commit/d6cc19eca06895a2353799d51293fc8c623199e7))

### Features

- *(release)* Autogenerate the changelog and gate the version surface ([`177771f`](https://github.com/0xSymbiome/cow-rs/commit/177771f869a517403fd25aa14531052f92ac3c2d))
- [**breaking**] PartnerFee v1.1.0, subgraph bearer auth, wasm v2 reads, and tooling/docs cleanup (#7) ([`10936a0`](https://github.com/0xSymbiome/cow-rs/commit/10936a052ec51a9540f7c64c5e2dfbcb936bd02c))
- [**breaking**] *(workspace)* Retire the browser-wallet crate and move the wasm examples to a dedicated repo (#6) ([`b70f95c`](https://github.com/0xSymbiome/cow-rs/commit/b70f95c4c6dce506cc5f604082f7aaa1765165f2))
- *(wasm)* Prepare the npm package for its 0.1.0-alpha.1 release ([`bdf9d7c`](https://github.com/0xSymbiome/cow-rs/commit/bdf9d7cf6198fc79c67b7651d852f8c38e43c41c))

## [0.1.0-alpha.1] - 2026-06-15

The first functional release of `cow-rs`, a Rust SDK for CoW Protocol: the typed
`cow-sdk` crate family (`core`, `contracts`, `signing`, `app-data`, `orderbook`,
`trading`, `subgraph`, and the opt-in native Alloy adapters), the
TypeScript-callable `@symbiome-forge/cow-sdk-wasm` package, and in-memory test
doubles. See the [README](README.md) and [`docs/`](docs/) for the full public
surface and the architecture decision records.
