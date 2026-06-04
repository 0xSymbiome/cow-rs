# cow-rs

[![CI](https://img.shields.io/badge/CI-workflow-2088FF?logo=githubactions&logoColor=white)](.github/workflows/ci.yml) [![docs.rs](https://img.shields.io/docsrs/cow-sdk?label=docs.rs)](https://docs.rs/cow-sdk) [![crates.io](https://img.shields.io/crates/v/cow-sdk)](https://crates.io/crates/cow-sdk) [![MSRV 1.94.0](https://img.shields.io/badge/MSRV-1.94.0-0A7BBB)](docs/release-checklist.md#3-compatibility-and-host-coverage) [![License GPL-3.0-only](https://img.shields.io/badge/license-GPL--3.0--only-1F6FEB)](LICENSE)

`cow-rs` is a Rust SDK for CoW Protocol.

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

<!-- runtime-routing:start -->
## When to use cow-rs

| You are building... | Use | Why |
| --- | --- | --- |
| MEV bot, market maker, solver, analytics job, or treasury automation in Rust | `cow-sdk` | Native Rust facade over typed transport, signing, orderbook, trading, and subgraph surfaces |
| Native Rust app using Alloy | `cow-sdk` plus `cow-sdk-alloy-*` | Opt-in Alloy provider and signer adapters without widening the default facade |
| Rust app compiled to browser WASM | `cow-sdk-browser-wallet` plus `cow-sdk-transport-wasm` | Rust-on-wasm wallet and fetch plumbing; not the JavaScript-callable npm package |
| Standard browser dapp or CowSwap-style UI in TypeScript | Upstream [`@cowprotocol/cow-sdk`](https://www.npmjs.com/package/@cowprotocol/cow-sdk) | Substantially smaller bundle at equivalent feature subsets; mature web ecosystem fit |
| TypeScript service that needs byte-for-byte Rust signing parity (viem, ethers, wagmi, or EIP-1193 wallets) | `<published-cow-sdk-wasm-package>` | TypeScript facade over deterministic Rust helpers with wallet-stack-agnostic callbacks |
| Single-source-of-truth Rust + TypeScript embedding | `<published-cow-sdk-wasm-package>` | One implementation across Rust and JavaScript runtimes |
| Browser dapp that only needs orderbook plus signing (smaller bundle) | `<published-cow-sdk-wasm-package>/orderbook` | Smaller wasm flavor for quote, post, lookup, trade, and cancellation flows |
| Signer service or HSM proxy | `<published-cow-sdk-wasm-package>/signing` | Signing, UID, EIP-1271, deployment, and version helpers without HTTP clients |
| Node.js 22 or 24 LTS backend service | `<published-cow-sdk-wasm-package>` | Node target works with explicit fetch or callback transport |
| Cloudflare Worker proxying orderbook calls | `<published-cow-sdk-wasm-package>/cloudflare` | Size-compatible with the current Workers Free compressed-size limit at the time of measurement; full Workers support pending release-bundle and startup validation |
| Deno | `<published-cow-sdk-wasm-package>` | Experimental build-only support; validate in your own runtime before production use |
| Account-abstraction hooks via Cow Shed | `cow-sdk` with the `cow-shed` feature, or `cow-sdk-cow-shed` directly | Deterministic proxy derivation, EIP-712 hook signing, factory calldata, and the `CowShedHooks` orchestrator; opt-in and off the default closure |
| TWAP, composable, bridging, flash-loan, weiroll, or hardware-wallet flows | Upstream TypeScript packages until `cow-rs` ships those capabilities | These capability families are intentionally outside the 0.1.0 package scope |
| Non-JS wasm consumers, WASI, WebAssembly components, TinyGo, Blazor, AssemblyScript guests, or no_std | Out of scope for 0.1.0 | Use native Rust crates where possible; the npm package targets JavaScript hosts |
<!-- runtime-routing:end -->

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

Ready-state facade setup on the native/default transport path:

```rust
use cow_sdk::{SupportedChainId, Trading};

let _trading = Trading::builder()
    .chain_id(SupportedChainId::Sepolia)
    .app_code("your-app-code")
    .build()
    .unwrap();
```

Use `appCode` as the stable identifier for the application or integration
surface that originates the order flow.

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
| Shared domain types, runtime traits, and the `HttpTransport` seam with its native `ReqwestTransport` default | `cow-sdk-core` |
| Shared HTTP retry, rate-limit, jitter, `Retry-After`, and error-classification policy | `cow-sdk-transport-policy` |
| Browser-target HTTP transport (`FetchTransport`) for `wasm32-unknown-unknown` | `cow-sdk-transport-wasm` |
| TypeScript-callable wasm-bindgen SDK bindings for browser, Node.js, Workers, and optional Deno consumers | `cow-sdk-wasm` |
| Read-only subgraph queries | `cow-sdk-subgraph` |
| Browser wallet integration for WASM | `cow-sdk-browser-wallet` or `cow-sdk` with `browser-wallet` |
| Native Alloy provider, signer, or composed provider-plus-signer support | `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`, or `cow-sdk` with `alloy-provider`, `alloy-signer`, or `alloy` |
| Deterministic protocol helpers, `alloy::sol!` bindings, the `Registry` authority, and EIP-1271 verification | `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data` |
| Typed orderbook transport | `cow-sdk-orderbook` |
| High-level trading workflows | `cow-sdk-trading` |

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
- `cow-sdk-subgraph` is a separate read-only crate.
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
| Compatibility and license | Public MSRV is Rust `1.94.0`; the current workspace license is `GPL-3.0-only`. |

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
