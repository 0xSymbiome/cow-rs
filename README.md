# cow-rs

`cow-rs` is a Rust SDK for CoW Protocol.

As of 2026-04-09, this workspace includes order creation, signing, and submission flows, low-level contract helpers, app-data encoding and CID handling, typed orderbook transport, read-only subgraph queries, WASM builds, and feature-gated browser wallet integration.

## Workspace

| Crate | Role |
| --- | --- |
| `cow-sdk` | Thin facade for the primary public entrypoint |
| `cow-sdk-core` | Shared types, config, validation, and runtime traits |
| `cow-sdk-contracts` | Order hashing, settlement encoding, contract helpers |
| `cow-sdk-signing` | EIP-712 signing, cancellation signing, UID helpers |
| `cow-sdk-app-data` | App-data generation, schema validation, CID conversion, pinning seams |
| `cow-sdk-orderbook` | Typed orderbook client, request policy, decoding helpers |
| `cow-sdk-trading` | Quote, build, sign, submit, cancel, allowance, approval workflows |
| `cow-sdk-subgraph` | Read-only subgraph query helpers |
| `cow-sdk-browser-wallet` | Async EIP-1193 browser wallet integration for WASM consumers |

`cow-sdk` stays intentionally thin. Trading workflows live in `cow-sdk-trading`. Subgraph access stays in `cow-sdk-subgraph`. Browser wallet support is exposed through the optional `browser-wallet` feature and the dedicated `cow-sdk-browser-wallet` crate.

## Docs

- [Strategy](docs/strategy.md)
- [Architecture](docs/architecture.md)
- [Review Guide](docs/review-guide.md)
- [Security And Test Matrix](docs/security-matrix.md)
- [Parity Matrix](docs/parity-matrix.md)
- [Parity Sources](docs/parity-sources.md)
- [Parity Scope](docs/parity-scope.md)
- [Audits](docs/audit/README.md)
- [Examples](docs/examples.md)
- [Open Questions](docs/open-questions.md)
- [ADRs](docs/adr/README.md)

## Validation

```text
cargo test --workspace
cargo check -p cow-sdk --examples
cargo build --target wasm32-unknown-unknown -p cow-sdk --features browser-wallet
```

## Examples

- `examples/native/` contains native SDK scenarios.
- `examples/wasm/sdk-verification-console/` contains a deterministic WASM verification console.
- `examples/wasm/browser-wallet-console/` contains mock-wallet and injected-wallet browser flows.
