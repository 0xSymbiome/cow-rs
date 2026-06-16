# cow-rs

[![CI](https://github.com/0xSymbiome/cow-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/0xSymbiome/cow-rs/actions/workflows/ci.yml) [![docs.rs](https://img.shields.io/docsrs/cow-sdk?label=docs.rs)](https://docs.rs/cow-sdk) [![crates.io](https://img.shields.io/static/v1?label=crates.io&message=v0.1.0-alpha.1&color=e6a96d)](https://crates.io/crates/cow-sdk) [![MSRV 1.94.0](https://img.shields.io/badge/MSRV-1.94.0-0A7BBB)](docs/release-checklist.md#3-compatibility-and-host-coverage) [![License GPL-3.0-or-later](https://img.shields.io/badge/license-GPL--3.0--or--later-1F6FEB)](LICENSE)

`cow-rs` is a Rust SDK for CoW Protocol.

It is built model-first: the protocol's invariants are encoded into the type
system and enforced by construction, then backed by executable evidence —
typed amounts and addresses, typestate builders that turn a misconfigured
client into a compile error, quote-to-order binding that fails closed if a
quote response drifts from the request, signature rules checked against pinned
upstream fixtures, credential redaction by construction, and a panic-free
production surface.
Every such invariant is indexed with its proof in the
[Properties Registry](PROPERTIES.md) and held by release-gating CI policy, so
correctness is enforced by the build rather than trusted to memory.

It provides typed Rust surfaces for order creation, signing, quoting,
submission, app-data handling, orderbook access, read-only subgraph
queries, browser-compatible WASM workflows, a pluggable `HttpTransport`
seam with native and browser default adapters, shared retry and rate-limit
transport policy, a typed deployment registry, opt-in native Alloy provider
and signer adapters, TypeScript-callable wasm-bindgen bindings, and an
optional EIP-1271 signature-verification cache.

The native Alloy adapter is provided for trading-flow consumers. Generic
Ethereum applications without trading helpers should depend on Alloy directly;
the adapter exists to wire native Alloy into the SDK's signing and transaction
contracts.

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

    let signer = LocalAlloySigner::builder()
        .private_key(&std::env::var("PRIVATE_KEY")?)?
        .chain_id(chain)
        .build()?;

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

<!-- runtime-routing:start -->
## When to use cow-rs

| You are building... | Use | Why |
| --- | --- | --- |
| MEV bot, market maker, solver, analytics job, or treasury automation in Rust | `cow-sdk` | Native Rust facade over typed transport, signing, orderbook, trading, and subgraph surfaces |
| Native Rust app using Alloy | `cow-sdk` plus `cow-sdk-alloy-*` | Opt-in Alloy provider and signer adapters without widening the default facade |
| Rust app compiled to browser WASM | `cow-sdk-browser-wallet` plus `cow-sdk-core` | Rust-on-wasm wallet and the browser `FetchTransport` from `cow-sdk-core`; not the JavaScript-callable npm package |
| Standard browser dapp or CowSwap-style UI in TypeScript | Upstream [`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk) | Substantially smaller bundle at equivalent feature subsets; mature web ecosystem fit |
| TypeScript service that needs byte-for-byte Rust signing parity (viem, ethers, wagmi, or EIP-1193 wallets) | npm package&nbsp;† | TypeScript facade over deterministic Rust helpers with wallet-stack-agnostic callbacks |
| Single-source-of-truth Rust + TypeScript embedding | npm package&nbsp;† | One implementation across Rust and JavaScript runtimes |
| Browser dapp that only needs orderbook plus signing (smaller bundle) | npm package&nbsp;† (orderbook flavor) | Smaller wasm flavor for quote, post, lookup, trade, and cancellation flows |
| Signer service or HSM proxy | npm package&nbsp;† (signing flavor) | Signing, UID, EIP-1271, deployment, and version helpers without HTTP clients |
| Node.js 22 or 24 LTS backend service | npm package&nbsp;† | Node target works with explicit fetch or callback transport |
| Cloudflare Worker proxying orderbook calls | npm package&nbsp;† (cloudflare flavor) | Size-compatible with the current Workers Free compressed-size limit at the time of measurement; full Workers support pending release-bundle and startup validation |
| Deno | npm package&nbsp;† | Experimental build-only support; validate in your own runtime before production use |
| Account-abstraction hooks via Cow Shed | `cow-sdk` with the `cow-shed` feature, or `cow-sdk-contracts` with the `cow-shed` feature | Deterministic proxy derivation, EIP-712 hook signing, factory calldata, and the `CowShedHooks` orchestrator; opt-in and off the default closure |
| TWAP, composable, bridging, flash-loan, weiroll, or hardware-wallet flows | Upstream TypeScript packages until `cow-rs` ships those capabilities | These capability families are intentionally outside the 0.1.0 package scope |
| Non-JS wasm consumers, WASI, WebAssembly components, TinyGo, Blazor, AssemblyScript guests, or no_std | Out of scope for 0.1.0 | Use native Rust crates where possible; the npm package targets JavaScript hosts |
<!-- runtime-routing:end -->

† The TypeScript-callable WASM package name is finalized at npm publication; the
install command is in [Start Here](#start-here). It ships in default,
`orderbook`, `signing`, and `cloudflare` flavors.

## Start Here

The canonical first-touch path is [Getting Started](docs/getting-started.md).
The shipped crate family and deferred capability boundaries are listed in the
[First-Release Scope](docs/parity.md#first-release-scope).

The functional published install surface will be:

```text
cargo add cow-sdk
```

The TypeScript-callable WASM package name is resolved at npm publication time:

```text
npm install <published-cow-sdk-wasm-package>
```

Reserved-placeholder `0.0.1-reserved.0` entries are already live on crates.io
for the published crate family. They reserve package identity and are not the
functional SDK release. Until `0.1.0` is live, use the getting-started guide
and the maintained native scenarios in this repository to evaluate the same
facade and trading flow end to end.

Use `appCode` as the stable identifier for the application or integration
surface that originates the order flow; the [Quickstart](#quickstart) above shows
it wired into a full swap on the native/default transport path.

Browser-wallet integrations that wrap a reviewed local transport should keep
the trusted origin explicit:

```rust
use cow_sdk::browser_wallet::{BrowserWallet, MockEip1193Transport, Origin};

let transport = MockEip1193Transport::sepolia().with_label("example wallet");
let origin = Origin::new("test://example-wallet").expect("example origin must be valid");
let _wallet = BrowserWallet::from_trusted_transport(transport, origin)
    .expect("trusted example transport must build");
```

## Crate Guide

| Need | Crate |
| --- | --- |
| Main Rust SDK entrypoint | `cow-sdk` |
| Shared domain types, runtime traits, the `HttpTransport` seam with its native `ReqwestTransport` default and browser `FetchTransport` default (the latter gated to `wasm32-unknown-unknown` in the `transport::fetch` module), and the opt-in HTTP retry, rate-limit, jitter, `Retry-After`, and error-classification policy (`transport-policy` feature) | `cow-sdk-core` |
| TypeScript-callable wasm-bindgen SDK bindings for browser, Node.js, Workers, and optional Deno consumers | `cow-sdk-wasm` |
| Read-only subgraph queries | `cow-sdk-subgraph` or `cow-sdk` with `subgraph` |
| Browser wallet integration for WASM | `cow-sdk-browser-wallet` or `cow-sdk` with `browser-wallet` |
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
- Browser wallet support is additive and feature-gated.
- TypeScript-callable WASM support is an additive leaf crate with explicit
  JavaScript callbacks rather than bundled wallet-library dependencies.
- Pure transform crates do not hide network I/O.
- Public claims are backed by repository-visible tests, fixtures, and release
  documentation.
- Public evolution follows the [Forward-Compatible Public Surfaces](docs/principles.md#forward-compatible-public-surfaces),
  [Credential Redaction by Construction](docs/principles.md#credential-redaction-by-construction),
  [Cooperative Cancellation Coverage](docs/principles.md#cooperative-cancellation-coverage),
  [Type The Lifecycle](docs/principles.md#type-the-lifecycle),
  and [Minimum-Viable Panic Surface](docs/principles.md#minimum-viable-panic-surface)
  principles.

## Trust And Maintenance

| Signal | Current state |
| --- | --- |
| Verification and release posture | [Verification](docs/verification.md) and [Release Checklist](docs/release-checklist.md) define the maintained proof and publication contract. |
| Change history | [CHANGELOG.md](CHANGELOG.md) tracks the current unreleased public contract and future release notes. |
| Security disclosure | [SECURITY.md](SECURITY.md) defines the private repository reporting path and protocol-level escalation route. |
| Chain-RPC runtime neutrality | The default facade remains provider-neutral. Native Alloy runtime dependencies are limited to the opt-in Alloy adapter crates and facade features, and CI gates the allow-list. |
| Publication state | Reserved-placeholder `0.0.1-reserved.0` crates.io and docs.rs entries are live for the published crate family, but the functional `0.1.0` release is still pending; [Getting Started](docs/getting-started.md) and [Release Checklist](docs/release-checklist.md) describe the current repo-local and release-ready contract truthfully. |
| Compatibility and license | Public MSRV is Rust `1.94.0`; the current workspace license is `GPL-3.0-or-later`. |

## Documentation

The full public map — consumer guides, verification, parity, audits, and ADRs —
lives in the [Documentation Index](docs/README.md). Quick starts:

- [Getting Started](docs/getting-started.md) — facade-first path to a signed order
- [Architecture](docs/architecture.md) — crate ownership and public boundaries
- [Verification](docs/verification.md) and [Parity And Provenance](docs/parity.md) — proof classes and upstream authorities
- [Contributing](CONTRIBUTING.md)

## Examples

- [Getting Started](docs/getting-started.md)
- [Native examples](examples/native/README.md)
- [Browser-wallet trade example (Dioxus, wasm)](examples/wasm/cow-trader-dioxus/README.md)

## Compatibility

- Public MSRV: Rust `1.94.0`
- Contributor toolchain pin: Rust `1.94.1`
- Surface-to-proof mapping lives in [Verification](docs/verification.md)

The [MSRV policy](docs/msrv-policy.md) defines when the workspace may raise
the public Rust floor, including the minor-release cadence, 30-day notice
window, and dependency, language-feature, or security-advisory triggers.
