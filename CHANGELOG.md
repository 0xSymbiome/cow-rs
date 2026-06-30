# Changelog

All notable changes to `cow-rs` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0-alpha.10] - 2026-06-30

### Bug Fixes

- *(ci)* Green the workspace gates for the placement and component surface ([`6def60c`](https://github.com/0xSymbiome/cow-rs/commit/6def60ce7697028da7f1366759bc2fcaf2f2286e))

### Features

- *(trading)* Place orders from smart-contract wallets via an authorization value ([`b7bf976`](https://github.com/0xSymbiome/cow-rs/commit/b7bf976d2e7ed44b3bcc96fc204720d1b653ab27))

## [0.1.0-alpha.9] - 2026-06-29

### Bug Fixes

- *(js)* Decode the domain-separator envelope in the wasm surface test ([`af8f752`](https://github.com/0xSymbiome/cow-rs/commit/af8f752cf1f130e4e696134e2b86a219eadda5d5))
- *(deps)* Patch the RustSec advisory gate ([`6826f8e`](https://github.com/0xSymbiome/cow-rs/commit/6826f8e345be220da18d7b001d6e6710aea3644c))
- *(ci)* Green the workspace build ([`b4405c5`](https://github.com/0xSymbiome/cow-rs/commit/b4405c57e6d954cc11880ab7de1a244b414a2e14))
- *(fuzz)* Drop Result handling from the now-infallible digest helpers ([`876d773`](https://github.com/0xSymbiome/cow-rs/commit/876d77342fe6ea55af217ae1d74680635545a074))
- *(trading)* Clarify the owner-mismatch rejection message ([`7409c85`](https://github.com/0xSymbiome/cow-rs/commit/7409c85fbb84680ece4322815d7a074444f1f62a))
- *(orderbook)* Classify host-policy and scheme-conflict faults as validation ([`22e66e7`](https://github.com/0xSymbiome/cow-rs/commit/22e66e7fb4025da4d8b25eb6bf268abd8f156524))
- *(contracts)* Redact signer-authored text in cow-shed errors ([`9434bb7`](https://github.com/0xSymbiome/cow-rs/commit/9434bb7a21bd3ac3da4e1cc9abc38ac4090dc99a))
- *(wasm)* Surface a declined trading signature as a typed wallet rejection ([`edbac3b`](https://github.com/0xSymbiome/cow-rs/commit/edbac3b200b6b65babdc63ce0af11300d8187cd9))
- *(wasm)* Feature-gate the orderbook and trading dto imports ([`431cc22`](https://github.com/0xSymbiome/cow-rs/commit/431cc22f426fecfd8b24b4d89b6dcf47806da694))

### Features

- *(component)* Expose the pure trading-math helpers in the engine world ([`8e29e8c`](https://github.com/0xSymbiome/cow-rs/commit/8e29e8c7998bb824af12a66dcedd1928019b9b53))
- *(component)* Add the WebAssembly Component distribution crate ([`47308c8`](https://github.com/0xSymbiome/cow-rs/commit/47308c8aaa9d32f95a4f1c6a6271def487aba80e))
- *(composable)* Add the ComposableCoW TWAP conditional-order surface ([`939ef10`](https://github.com/0xSymbiome/cow-rs/commit/939ef10676477d06c6249e5ac49eb8645f9228c8))
- [**breaking**] *(wasm)* Type the boundary hex scalars as viem-compatible 0x template literals ([`ab89f5b`](https://github.com/0xSymbiome/cow-rs/commit/ab89f5bf13722c9c554d5191ec0dd348693bb4d1))

### Refactor

- [**breaking**] *(js)* Rename the wasm-bindgen leaf cow-sdk-wasm to cow-sdk-js ([`e237d64`](https://github.com/0xSymbiome/cow-rs/commit/e237d64b09c505b93be9d5bc8abbae7ed6f4dfef))
- [**breaking**] *(wasm)* De-Dto the boundary and surface native types directly ([`d4b257c`](https://github.com/0xSymbiome/cow-rs/commit/d4b257c6bd8c372571f8cb527bc6e837743d7871))
- [**breaking**] *(repo)* Drop unused error-class and signature helpers, type the quote metadata ([`f27a9fd`](https://github.com/0xSymbiome/cow-rs/commit/f27a9fd4e58fa82da82aa1b4cf0850f6f9d93ac3))
- [**breaking**] *(signing)* Drop the in-memory EIP-1271 cache, keep the seam ([`c0f1cc2`](https://github.com/0xSymbiome/cow-rs/commit/c0f1cc286f4cc506753b78f66c8ca8dea3312e80))
- [**breaking**] *(contracts)* Complete the gas-free transaction builder surface ([`a3bb743`](https://github.com/0xSymbiome/cow-rs/commit/a3bb743f0436a78f1bc6405a97c7d8e0d6c41e66))
- [**breaking**] *(wasm)* Drop the eip1193 signer and collapse the eip1271 alias ([`8119740`](https://github.com/0xSymbiome/cow-rs/commit/81197400dbcadfa4e2636d908d0673b5d90a0955))
- [**breaking**] Set Order.total_fee from the typed executed_fee and drop the dead calculate_total_fee ([`c11e140`](https://github.com/0xSymbiome/cow-rs/commit/c11e140daa031a62fda04e6c55672161cfd0fa34))
- [**breaking**] Type the amount and slippage math, dropping the decimal-string round-trips ([`7292e2d`](https://github.com/0xSymbiome/cow-rs/commit/7292e2d93783b226348b6a158f9bb59d7519733b))
- [**breaking**] *(contracts)* Own the pure tx builders and settlement encoders ([`ce67fe9`](https://github.com/0xSymbiome/cow-rs/commit/ce67fe98a07efcfcb48576ede26ddbc1e0a0a0f9))
- [**breaking**] *(wasm)* Collapse js error plumbing and align the limit owner ([`c6da747`](https://github.com/0xSymbiome/cow-rs/commit/c6da747b8a1d5eb0cbf9e9bfe7c23fdedb43967e))
- [**breaking**] *(core)* Drop the operator-tunable window from ValidTo::relative ([`2b1fc1b`](https://github.com/0xSymbiome/cow-rs/commit/2b1fc1b5994ed9c7447b07eae8309f3d9a1476d0))
- [**breaking**] Single-source duplicated transport and registry helpers ([`d04fdb3`](https://github.com/0xSymbiome/cow-rs/commit/d04fdb39eb15af7654a85f526ffd9c493b2d50cb))
- [**breaking**] *(app-data)* Convert IpfsFetchTransport to native async fn in trait ([`2c5a343`](https://github.com/0xSymbiome/cow-rs/commit/2c5a3434f557d6f5ce09a1546f14740fad76fbe9))
- [**breaking**] *(contracts)* Drop the vestigial Result from infallible EIP-712 helpers ([`9193f6e`](https://github.com/0xSymbiome/cow-rs/commit/9193f6e5e8998451df99b770a7a517e29d80406c))
- [**breaking**] *(alloy)* Derive adapter error Display via thiserror over redacted payloads ([`cdbdb28`](https://github.com/0xSymbiome/cow-rs/commit/cdbdb286203217c7b196ea599701c043fd1bd2b6))
- [**breaking**] *(alloy)* Make EIP-712 signing unconditional ([`9768a54`](https://github.com/0xSymbiome/cow-rs/commit/9768a54e88a75c6ac13ca0e97135b8a22fc2f760))

## [0.1.0-alpha.8] - 2026-06-20

### Bug Fixes

- *(wasm)* Expose the wrap surface to consumers and gate per-flavour dtos ([`2bac1e0`](https://github.com/0xSymbiome/cow-rs/commit/2bac1e0abda0a85ea96c8a2096861c9891d079ed))

### Features

- [**breaking**] *(wasm)* Make CowError a thrown Error subclass with the orderbook errorType tag and retry helpers ([`1aaea0f`](https://github.com/0xSymbiome/cow-rs/commit/1aaea0fb7bb06ce844e1b044e591a84ee0c10c3d))

### Refactor

- [**breaking**] *(errors)* Share one serde classifier and rename the app-data serialization variant ([`d74fbae`](https://github.com/0xSymbiome/cow-rs/commit/d74fbaed9e72285eff21b5bf3c7a33292a1f6eda))

## [0.1.0-alpha.7] - 2026-06-19

### Bug Fixes

- *(examples)* Cover token_balance in the deterministic example runner ([`559346e`](https://github.com/0xSymbiome/cow-rs/commit/559346e8f2719901cb8f896a64d185588cbf94d0))

### Features

- *(ergonomics)* Add OrderStatus::is_fulfilled and LocalAlloySigner::from_private_key ([`ba3c3bd`](https://github.com/0xSymbiome/cow-rs/commit/ba3c3bd3f536eb57403375799c877b3d1e70b130))
- *(trading)* Add native wrap and unwrap transaction builders ([`91366dd`](https://github.com/0xSymbiome/cow-rs/commit/91366dddfe5be4f3f17e961aeba3ddb4be70abbc))

## [0.1.0-alpha.6] - 2026-06-19

### Features

- [**breaking**] *(wasm)* Serve every flavour on the web and source-phase module targets (#10) ([`15d32cc`](https://github.com/0xSymbiome/cow-rs/commit/15d32cca7d0602f75f34234cf15dfc41e1b43cc3))

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
