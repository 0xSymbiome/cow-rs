# cow-rs

[![CI](https://github.com/0xSymbiome/cow-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/0xSymbiome/cow-rs/actions/workflows/ci.yml) [![docs.rs](https://img.shields.io/docsrs/cow-sdk?label=docs.rs)](https://docs.rs/cow-sdk) [![crates.io](https://img.shields.io/static/v1?label=crates.io&message=v0.1.0-alpha.8&color=e6a96d)](https://crates.io/crates/cow-sdk) [![npm](https://img.shields.io/npm/v/@symbiome-forge/cow-sdk-wasm/alpha?label=npm&color=cb3837)](https://www.npmjs.com/package/@symbiome-forge/cow-sdk-wasm) [![MSRV 1.94.0](https://img.shields.io/badge/MSRV-1.94.0-0A7BBB)](docs/release-checklist.md#3-compatibility-and-host-coverage) [![License GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-1F6FEB)](LICENSE)

`cow-rs` is a model-first Rust SDK for [CoW Protocol](https://cow.fi): build and
sign orders, drive the orderbook, and decode settlement across every supported
chain.

Model-first means the protocol's invariants are encoded into the type system and
enforced by construction, then backed by executable evidence: typed amounts and
addresses, typestate builders that turn a misconfigured client into a compile
error, quote-to-order binding that fails closed when a quote drifts from its
request, signature rules pinned to upstream fixtures, credential redaction by
construction, and a panic-free production surface. Every invariant is indexed to
its proof in the [Properties Registry](PROPERTIES.md) and held by release-gating
CI policy, so correctness is enforced by the build rather than trusted to memory.

## Capabilities and guarantees

- **Quote → sign → post → look up → cancel** — the whole order lifecycle through
  the fluent `Trading::swap` pipeline, plus limit, pre-sign, and EthFlow paths.
- **All four signing schemes** (EIP-712, EthSign, EIP-1271, pre-sign) across the
  11 chains in `SupportedChainId`, Mainnet through Sepolia.
- **Misconfiguration is a compile error** — typestate builders make a transposed
  sell/buy leg or a missing amount fail to build, not fail at runtime.
- **Typed failure taxonomy, not strings** — every orderbook `errorType` decodes
  to a typed rejection variant with `is_retryable()` and `backoff_hint()`, and
  the `Retry-After` header is retained, so production retry logic is a match arm.
- **Capability isolation by crate boundary** — the default facade carries no
  Ethereum runtime; the native-Alloy provider and local-key signer live in
  opt-in adapter crates whose dependency boundary is enforced by a CI allow-list,
  so a keystore signer never enters a graph that did not ask for one.
- **Runtime-free protocol core** — hashing, signing, and contract decoding
  compute with no async runtime; only the HTTP client needs a reactor, and a
  `wasm32` build drops the `reqwest` stack at compile time. The `cow-sdk`
  facade and the wasm-facing crates (`cow-sdk-wasm`, `cow-sdk-orderbook`,
  `cow-sdk-subgraph`) compile to `wasm32-unknown-unknown` with a
  headless-browser e2e lane in CI.
- **Evidence over adjectives** — every protocol transform is cross-checked
  byte-for-byte against pinned `cowprotocol/services` and `cowprotocol/contracts`
  fixtures in CI; see [Parity and Provenance](docs/parity.md).

## Install

```toml
[dependencies]
cow-sdk = "0.1.0-alpha.8"
```

`cow-sdk` is in alpha, so the pre-release is pinned explicitly; `cargo add
cow-sdk@0.1.0-alpha.8` does the same. JavaScript and TypeScript consumers install
the wasm bindings from npm:

```sh
npm install @symbiome-forge/cow-sdk-wasm@0.1.0-alpha.8
```

Published as [`cow-sdk`](https://crates.io/crates/cow-sdk) on crates.io and
[`@symbiome-forge/cow-sdk-wasm`](https://www.npmjs.com/package/@symbiome-forge/cow-sdk-wasm)
on npm. MSRV Rust `1.94.0`, edition 2024.

The native Alloy adapter wires native Alloy into the SDK's signing and
transaction contracts for trading-flow consumers; generic Ethereum applications
without trading helpers should depend on Alloy directly.

## Quickstart

Sell WETH for COW on Sepolia — quote, sign, and post in one fluent call. This is
a complete program: export `PRIVATE_KEY` and run it. Tokens are compile-time
validated `Address` literals; the named setters keep the sell and buy legs from
being transposed, and `execute` is reachable only once both tokens and an amount
are set. The order owner defaults to the signer address:

```rust,no_run
use std::error::Error;

use cow_sdk::alloy_signer::LocalAlloySigner;
use cow_sdk::core::{address, Address, Amount, SupportedChainId};
use cow_sdk::trading::Trading;

// Compile-time validated address literals — the lowercase wire form, no runtime
// parse and no unwrap.
const WETH: Address = address!("0xfff9976782d46cc05630d1f6ebab18b2324d6b14");
const COW: Address = address!("0x0625afb445c3b6b7b929342a04a22599fd5dbb59");

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let chain = SupportedChainId::Sepolia;

    let signer = LocalAlloySigner::from_private_key(&std::env::var("PRIVATE_KEY")?, chain)?;

    let trading = Trading::builder()
        .chain_id(chain)
        .app_code("your-app-code")
        .build()?;

    let posted = trading
        .swap()
        .sell_token(WETH)
        .buy_token(COW)
        .sell_amount(Amount::parse_units("0.1", 18)?)
        .execute(&signer)
        .await?;

    println!("https://explorer.cow.fi/sepolia/orders/{}", posted.order_id);
    Ok(())
}
```

This program uses `cow-sdk` with the `alloy-signer` feature; any
`cow_sdk::core::Signer` works in `execute`, and the local-key signer is the
batteries-included one. The same fluent flow is compiled on every CI run as the
[`cow-sdk` crate doctest](crates/sdk/README.md). For an end-to-end runnable path,
see the [native examples](examples/native/README.md).

## Surface map

The facade re-exports each leaf crate as a named module reached on its module
path (`cow_sdk::trading::Trading`, `cow_sdk::core::Address`), matching the
`alloy`, `reqwest`, and `tower` convention. The consumer-facing modules:

| Module path | What it owns | Guide |
| --- | --- | --- |
| `cow_sdk::trading` | High-level quote-to-order workflows: the fluent `Trading::swap` lifecycle, limit and pre-sign posting, EthFlow, and cancellation | [Getting Started](docs/getting-started.md) |
| `cow_sdk::orderbook` | Typed orderbook transport: quote, post, lookup, and cancellation requests | [orderbook crate](crates/orderbook/README.md) |
| `cow_sdk::signing` | Deterministic protocol signing: order UID and digest, EIP-712 domain, EIP-1271 verification | [signing crate](crates/signing/README.md) |
| `cow_sdk::core` | Shared domain types and runtime traits: `Address`, `Amount`, `SupportedChainId`, `Signer`, the `HttpTransport` seam, and transport policy | [core crate](crates/core/README.md) |
| `cow_sdk::app_data` | App-data document generation, validation, and CID handling | [app-data crate](crates/app-data/README.md) |
| `cow_sdk::subgraph` | Read-only subgraph analytics, behind the off-by-default `subgraph` feature | [subgraph crate](crates/subgraph/README.md) |
| `cow_sdk::alloy_signer` / `cow_sdk::alloy_provider` | Opt-in native Alloy signer and provider adapters, behind the `alloy-signer` / `alloy-provider` features | [native examples](examples/native/README.md) |

The full crate-by-need breakdown is in the [Crate Guide](#crate-guide) below.

Every protocol transform is cross-checked byte-for-byte against the pinned
upstream fixture corpus under [`parity/fixtures/`](parity/fixtures), so the
Rust encoding stays byte-identical to the upstream protocol producers
(`cowprotocol/services` for the wire DTOs, `cowprotocol/contracts` and EthFlow
for the on-chain surfaces) across releases;
the full source-to-fixture matrix is in
[Parity And Provenance](docs/parity.md).

## When to use cow-rs

| You are building in… | Use |
| --- | --- |
| Rust — bot, solver, market maker, analytics, or treasury automation | `cow-sdk`, plus `cow-sdk-alloy-*` for native Alloy provider/signer adapters |
| JavaScript or TypeScript — Node, browser bundler, Cloudflare Workers, or Deno | the npm package [`@symbiome-forge/cow-sdk-wasm`](https://www.npmjs.com/package/@symbiome-forge/cow-sdk-wasm), driven by the host application's own wallet stack (viem, wagmi, or any EIP-1193 provider) |
| A standard browser dapp where minimal bundle size dominates | upstream [`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk) |

The npm package ships in `default`, `orderbook`, `signing`, and `trading`
flavors; pick the smallest one that covers your calls. Every flavour serves browser
bundlers, Node, and edge: its web-target build is the browser default
(`await initialize()` once) and its explicit Workers entry is at `…/edge` (for
example `@symbiome-forge/cow-sdk-wasm/trading/edge`), with a source-phase `…/module`
build alongside. Account-abstraction hooks ship behind the opt-in `cow-shed`
feature. Capability families outside the 0.1.0
scope — TWAP, composable orders, bridging, flash loans, and hardware-wallet
flows — remain on the upstream TypeScript packages until cow-rs ships them.

## Start Here

The canonical first-touch path is [Getting Started](docs/getting-started.md).
The shipped crate family and deferred capability boundaries are listed in the
[First-Release Scope](docs/parity.md#first-release-scope).

Install is one line (see [Install](#install) above): `cargo add
cow-sdk@0.1.0-alpha.8` for Rust, `npm install @symbiome-forge/cow-sdk-wasm@0.1.0-alpha.8`
for JavaScript and TypeScript.

Use `appCode` as the stable identifier for the application or integration
surface that originates the order flow; the [Quickstart](#quickstart) above shows
it wired into a full swap on the native/default transport path.

JavaScript and TypeScript hosts that connect a wallet in the browser drive the
`@symbiome-forge/cow-sdk-wasm` package with their own wallet stack (viem, wagmi,
or any EIP-1193 provider). The wasm surface exposes typed signing and HTTP
callbacks; the host wraps its provider into the typed-data signer and supplies the
wallet connection.

## Crate Guide

| Need | Crate |
| --- | --- |
| Main Rust SDK entrypoint | `cow-sdk` |
| Shared domain types, runtime traits, the `HttpTransport` seam with its native `ReqwestTransport` default and browser `FetchTransport` default (the latter gated to `wasm32-unknown-unknown` in the `transport::fetch` module), and the opt-in HTTP retry, rate-limit, jitter, `Retry-After`, and error-classification policy (`transport-policy` feature) | `cow-sdk-core` |
| TypeScript-callable wasm-bindgen SDK bindings for browser, Node.js, Workers, and optional Deno consumers | `cow-sdk-wasm` |
| Read-only subgraph queries | `cow-sdk-subgraph` or `cow-sdk` with `subgraph` |
| Native Alloy provider, signer, or composed provider-plus-signer support | `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`, or `cow-sdk` with `alloy-provider`, `alloy-signer`, or `alloy` |
| Deterministic protocol helpers, `alloy::sol!` bindings, the `Registry` authority, and EIP-1271 verification | `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data` |
| Typed orderbook transport | `cow-sdk-orderbook` |
| High-level trading workflows | `cow-sdk-trading` |
| In-memory test doubles for the public traits (`OrderbookClient`, `Signer`, `Provider`) so downstream apps test without a live orderbook, RPC, or wallet | `cow-sdk-test` |

## TypeScript-Callable WASM

`cow-sdk-wasm` exposes deterministic Rust SDK logic to JavaScript and
TypeScript through a TypeScript facade, typed DTOs, explicit callbacks for
signing and HTTP dispatch, per-call cancellation, per-call timeouts, and
flavor-specific imports. Browser, Node.js, Workers, and other JavaScript hosts
configure transport explicitly through `transport: { kind: "fetch" }` or
`transport: { kind: "callback", callback }`.

## Public Boundary

- `cow-sdk` is a thin facade.
- `cow-sdk-trading` owns quote-to-order workflows.
- `cow-sdk-subgraph` is a separate read-only crate, re-exported through `cow-sdk` behind the off-by-default `subgraph` feature.
- TypeScript-callable WASM support is an additive leaf crate with explicit
  JavaScript callbacks rather than bundled wallet-library dependencies.
- Pure transform crates do not hide network I/O.
- Public claims are backed by repository-visible tests, fixtures, and release
  documentation.
- Public evolution follows the [Forward-Compatible Public Surfaces](docs/principles.md#forward-compatible-public-surfaces),
  [Credential Redaction by Construction](docs/principles.md#credential-redaction-by-construction),
  [Cooperative Cancellation Coverage](docs/principles.md#cooperative-cancellation-coverage),
  and [Minimum-Viable Panic Surface](docs/principles.md#minimum-viable-panic-surface)
  principles.

## Trust And Maintenance

| Signal | Current state |
| --- | --- |
| Verification and release posture | [Verification](docs/verification.md) and [Release Checklist](docs/release-checklist.md) define the maintained proof and publication contract. |
| Change history | [CHANGELOG.md](CHANGELOG.md) tracks the current unreleased public contract and future release notes. |
| Security disclosure | [SECURITY.md](SECURITY.md) defines the private repository reporting path and protocol-level escalation route. |
| Chain-RPC runtime neutrality | The default facade remains provider-neutral. Native Alloy runtime dependencies are limited to the opt-in Alloy adapter crates and facade features, and CI gates the allow-list. |
| Publication state | `cow-sdk` `0.1.0-alpha.8` is published on crates.io and `@symbiome-forge/cow-sdk-wasm` `0.1.0-alpha.8` on npm. This is a pre-release: the public surface is stabilizing toward `0.1.0` and may change between alpha versions, as [Getting Started](docs/getting-started.md) and the [Release Checklist](docs/release-checklist.md) describe. |
| Compatibility and license | Public MSRV is Rust `1.94.0`; the current workspace license is `GPL-3.0-or-later`. |

## Documentation

The full public map — consumer guides, verification, parity, audits, and ADRs —
lives in the [Documentation Index](docs/README.md). Quick starts:

- [Getting Started](docs/getting-started.md) — facade-first path to a signed order
- [Architecture](docs/architecture.md) — crate ownership and public boundaries
- [Verification](docs/verification.md) and [Parity And Provenance](docs/parity.md) — proof classes and upstream authorities
- [cow-rs and the TypeScript SDK](docs/comparison-with-typescript-sdk.md) — deferred scope and the guarantees Rust adds
- [Contributing](CONTRIBUTING.md)

## Examples

- [Getting Started](docs/getting-started.md)
- [Native examples](examples/native/README.md)

## Compatibility

- Public MSRV: Rust `1.94.0`
- Contributor toolchain pin: Rust `1.94.1`
- Surface-to-proof mapping lives in [Verification](docs/verification.md)

The [MSRV policy](docs/msrv-policy.md) defines when the workspace may raise
the public Rust floor, including the minor-release cadence, 30-day notice
window, and dependency, language-feature, or security-advisory triggers.
