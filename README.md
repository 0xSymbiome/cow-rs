# cow-rs

`cow-rs` is a Rust SDK for CoW Protocol.

This workspace includes order creation, signing, and submission flows, low-level contract helpers, app-data encoding and CID handling, typed orderbook transport, read-only subgraph queries, WASM builds, and feature-gated browser wallet integration.

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

`cow-sdk` is intentionally thin. Trading workflows live in `cow-sdk-trading`. Subgraph access lives in `cow-sdk-subgraph`. Browser wallet support is exposed through the optional `browser-wallet` feature and the dedicated `cow-sdk-browser-wallet` crate.

## Facade Surface

`cow-sdk` is the primary facade crate.

- Native and server-side consumers use the default `cow-sdk` surface for core types, contracts, signing, orderbook access, app-data helpers, and trading workflows.
- WASM consumers can use the same facade surface for pure SDK flows.
- Browser wallet support is additive and exposed through the `browser-wallet` feature plus the `cow-sdk-browser-wallet` crate.
- Subgraph access uses the separate `cow-sdk-subgraph` crate and is intentionally not re-exported from `cow-sdk`.

## Trading SDK Configuration

`TradingSdk` uses instance-scoped builder and options configuration.

- `TradingSdk::builder()` configures trader defaults and optional injected orderbook clients.
- Injected orderbook clients are authoritative for orderbook-bound chain and env selection. Conflicting SDK defaults or call-level requests fail explicitly.
- Advanced quote and post settings override overlapping call-level trade fields.
- Call-level params override SDK defaults for owner, env, and protocol address overrides.
- Signer address resolution is only an owner fallback for signer-backed quote and post flows.

## Typed Public API

The default Rust contract is strongly typed where this workspace owns the meaning:

- addresses use `cow_sdk_core::Address`
- hashes and digests use `cow_sdk_core::Hash32` aliases such as `TransactionHash` and `OrderDigest`
- token amounts, transaction values, and gas limits use `cow_sdk_core::Amount`
- signed balance deltas use `cow_sdk_core::SignedAmount`
- raw calldata and byte payloads use `cow_sdk_core::HexData`

String-heavy values live in explicit wire DTOs such as `cow-sdk-orderbook` request and response models, because the upstream HTTP API is string-heavy.

## Toolchain Policy

- Public MSRV: Rust `1.94`
- Contributor toolchain pin: Rust `1.94.1` in [rust-toolchain.toml](rust-toolchain.toml)

The MSRV is the compatibility contract for downstream users. The exact toolchain pin exists to keep local development, CI, and reproducible validation aligned.

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
cargo package -p cow-sdk --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-trading.path='crates/trading'" --config "patch.crates-io.cow-sdk-browser-wallet.path='crates/browser-wallet'"
```

## Examples

- `examples/native/` contains native SDK scenarios.
- `examples/wasm/sdk-verification-console/` contains deterministic WASM checks and a browser review surface for SDK verification.
- `examples/wasm/browser-wallet-console/` contains mock-wallet and injected-wallet browser flows.
