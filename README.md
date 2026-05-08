# cow-rs

[![CI](https://img.shields.io/badge/CI-workflow-2088FF?logo=githubactions&logoColor=white)](.github/workflows/ci.yml) [![docs.rs](https://img.shields.io/docsrs/cow-sdk?label=docs.rs)](https://docs.rs/cow-sdk) [![crates.io](https://img.shields.io/crates/v/cow-sdk)](https://crates.io/crates/cow-sdk) [![MSRV 1.94.0](https://img.shields.io/badge/MSRV-1.94.0-0A7BBB)](docs/release-checklist.md#3-compatibility-and-host-coverage) [![License GPL-3.0-only](https://img.shields.io/badge/license-GPL--3.0--only-1F6FEB)](LICENSE)

`cow-rs` is a Rust SDK for CoW Protocol.

It provides typed Rust surfaces for order creation, signing, quoting,
submission, app-data handling, orderbook access, read-only subgraph
queries, browser-compatible WASM workflows, a pluggable `HttpTransport`
seam with native and browser default adapters, a typed deployment
registry, opt-in native Alloy provider and signer adapters, and an optional
EIP-1271 signature-verification cache.

The native Alloy adapter is provided for trading-flow consumers. Generic
Ethereum applications without trading helpers should depend on Alloy directly;
the adapter exists to wire native Alloy into the SDK's signing and transaction
contracts.

## Start Here

The canonical first-touch path is [Getting Started](docs/getting-started.md).
The shipped crate family and deferred capability boundaries are listed in the
[First-Release Scope](docs/parity-scope.md#first-release-scope).

The functional published install surface will be:

```text
cargo add cow-sdk
```

Reserved-placeholder `0.0.1-reserved.0` entries are already live on crates.io
for the published crate family. They reserve package identity and are not the
functional SDK release. Until `0.1.0` is live, use the getting-started guide
and the maintained native scenarios in this repository to evaluate the same
facade and trading flow end to end.

Ready-state facade setup on the native/default transport path:

```rust
use cow_sdk::{SupportedChainId, TradingSdk};

let _sdk = TradingSdk::builder()
    .with_chain_id(SupportedChainId::Sepolia)
    .with_app_code("your-app-code")
    .build_ready()
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
| Browser-target HTTP transport (`FetchTransport`) for `wasm32-unknown-unknown` | `cow-sdk-transport-wasm` |
| Read-only subgraph queries | `cow-sdk-subgraph` |
| Browser wallet integration for WASM | `cow-sdk-browser-wallet` or `cow-sdk` with `browser-wallet` |
| Native Alloy provider, signer, or composed provider-plus-signer support | `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`, or `cow-sdk` with `alloy-provider`, `alloy-signer`, or `alloy` |
| Deterministic protocol helpers, `alloy::sol!` bindings, the `Registry` authority, and EIP-1271 verification | `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data` |
| Typed orderbook transport | `cow-sdk-orderbook` |
| High-level trading workflows | `cow-sdk-trading` |

## Public Boundary

- `cow-sdk` is a thin facade.
- `cow-sdk-trading` owns quote-to-order workflows.
- `cow-sdk-subgraph` is a separate read-only crate.
- Browser wallet support is additive and feature-gated.
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
| Verification and release posture | [Verification Guide](docs/verification-guide.md) and [Release Checklist](docs/release-checklist.md) define the maintained proof and publication contract. |
| Change history | [CHANGELOG.md](CHANGELOG.md) tracks the current unreleased public contract and future release notes. |
| Security disclosure | [SECURITY.md](SECURITY.md) defines the private repository reporting path and protocol-level escalation route. |
| Chain-RPC runtime neutrality | The default facade remains provider-neutral. Native Alloy runtime dependencies are limited to the opt-in Alloy adapter crates and facade features, and CI gates the allow-list. |
| Publication state | Reserved-placeholder `0.0.1-reserved.0` crates.io and docs.rs entries are live for the published crate family, but the functional `0.1.0` release is still pending; [Getting Started](docs/getting-started.md) and [Release Checklist](docs/release-checklist.md) describe the current repo-local and release-ready contract truthfully. |
| Compatibility and license | Public MSRV is Rust `1.94.0`; the current workspace license is `GPL-3.0-only`. |

## Documentation Paths

### For SDK Consumers

- [Getting Started](docs/getting-started.md)
- [Integrations](docs/integrations.md)
- [Documentation Index](docs/README.md)
- [Principles](docs/principles.md)
- [Architecture](docs/architecture.md)
- [Transport](docs/transport.md)
- [Deployments](docs/deployments.md)
- [Examples](docs/examples.md)
- [Bring Your Own Provider](docs/providers/README.md)
- [Native Alloy adapters](docs/providers/adapting-alloy.md)

Start with [Getting Started](docs/getting-started.md) for the shortest path
from the facade crate to deterministic signed-order output.

### For Verification And Review

- [Verification Guide](docs/verification-guide.md)
- [Validation Scope](docs/validation-scope.md)
- [Release Checklist](docs/release-checklist.md)
- [Properties Registry](PROPERTIES.md)

Use the [Documentation Index](docs/README.md) for the full public assurance,
parity, audit, and ADR map.

### For Contributors

- [Contributing](CONTRIBUTING.md)

## Examples

- [Getting Started](docs/getting-started.md)
- [Native examples](examples/native/README.md)
- [SDK verification console](examples/wasm/sdk-verification-console/README.md)
- [Browser wallet console](examples/wasm/browser-wallet-console/README.md)

## Compatibility

- Public MSRV: Rust `1.94.0`
- Contributor toolchain pin: Rust `1.94.1`
- Surface-to-proof mapping lives in [Validation Scope](docs/validation-scope.md)

The [MSRV policy](docs/msrv-policy.md) defines when the workspace may raise
the public Rust floor, including the minor-release cadence, 30-day notice
window, and dependency, language-feature, or security-advisory triggers.
