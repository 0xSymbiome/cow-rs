# cow-rs

`cow-rs` is a Rust SDK for CoW Protocol.

It provides typed Rust surfaces for order creation, signing, quoting,
submission, app-data handling, orderbook access, read-only subgraph queries,
and browser-compatible WASM workflows.

## Start Here

Install the main facade crate:

```text
cargo add cow-sdk
```

Minimal facade setup:

```rust
use cow_sdk::{Address, PartialTraderParameters, TradingSdk};

let _owner = Address::new("0x1111111111111111111111111111111111111111").unwrap();
let _sdk = TradingSdk::builder()
    .with_trader_defaults(PartialTraderParameters::default())
    .build()
    .unwrap();
```

## Crate Guide

| Need | Crate |
| --- | --- |
| Main Rust SDK entrypoint | `cow-sdk` |
| Read-only subgraph queries | `cow-sdk-subgraph` |
| Browser wallet integration for WASM | `cow-sdk-browser-wallet` or `cow-sdk` with `browser-wallet` |
| Low-level deterministic protocol helpers | `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data` |
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

## Documentation Paths

### For SDK Consumers

- [Documentation Index](docs/README.md)
- [Principles](docs/principles.md)
- [Architecture](docs/architecture.md)
- [Examples](docs/examples.md)

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

- [Native examples](examples/native/README.md)
- [SDK verification console](examples/wasm/sdk-verification-console/README.md)
- [Browser wallet console](examples/wasm/browser-wallet-console/README.md)

## Compatibility And Release

- Public MSRV: Rust `1.94.0`
- Contributor toolchain pin: Rust `1.94.1`
- Native, WASM, publication, and provenance checks are defined in
  [Release Checklist](docs/release-checklist.md)
- Surface-to-proof mapping lives in
  [Validation Scope](docs/validation-scope.md)
