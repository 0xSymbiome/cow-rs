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
- The facade is a curated re-export layer. Package-specific implementation behavior stays in the leaf crates that own it.

Native subgraph examples live under `examples/native/` and use `cow-sdk-subgraph` directly. The custom-query example uses the explicit `SubgraphQueryRequest` contract, and the live example is opt-in through explicit environment configuration.

## Browser Wallet Support

Browser wallet integration is a supported leaf capability for browser runtimes. It is not part of the default root-facade contract.

- Deterministic proof mode uses the mock wallet and crate tests to validate browser-wallet request shape, signing, approvals, and trading orchestration without an extension dependency.
- Injected-provider mode uses explicit EIP-1193 browser wallet flows on supported chains and requires user authorization plus wallet support for the requested methods.
- Injected discovery is explicit and bounded. Multi-wallet discovery requires caller selection instead of silently picking one provider.
- Typed chain management uses `WalletChainParameters` and `WalletNativeCurrency` for add-chain requests rather than exposing a generic raw wallet-RPC passthrough.
- Non-WASM targets keep the browser-wallet types available for deterministic tests and docs, but injected discovery resolves to an empty result set and direct detection resolves to `None`.
- Broader browser-extension behavior remains environment-sensitive. Authorization persistence, chain inventory, discovery timing, vendor-specific prompts, and non-standard wallet behavior are controlled by the extension and browser runtime rather than normalized by `cow-sdk`.

## Trading SDK Configuration

`TradingSdk` uses instance-scoped builder and options configuration.

- `TradingSdk::builder()` configures trader defaults and optional injected orderbook clients.
- Injected orderbook clients are authoritative for orderbook-bound chain and env selection. Conflicting SDK defaults or call-level requests fail explicitly.
- Advanced quote and post settings override overlapping call-level trade fields.
- Call-level params override SDK defaults for owner, env, and protocol address overrides.
- Signer address resolution is only an owner fallback for signer-backed quote and post flows.
- Quote-only flows resolve owner from the effective trade parameters first and otherwise use the supplied quoter account.
- Limit-order submission uses `0` basis points when slippage is omitted.
- Trading slippage and fee helpers use integer math with explicit rounding, truncation, and clamping rules.

## Orderbook Transport Contract

`cow-sdk-orderbook` keeps transport policy local to the crate instead of hiding it behind a generic shared HTTP abstraction.

- `OrderBookTransportPolicy` owns retry and rate-limit behavior, while `cow_sdk_core::HttpClientPolicy` owns timeout and user-agent only.
- `OrderBookApi::with_context_override()` updates chain, env, base URL maps, and API key on a cloned client.
- `OrderBookApi::with_env_base_url()` is the highest-precedence base-URL override for a specific environment.
- Clones of the same `OrderBookApi` share one limiter instance. Replacing the transport policy creates a new client/runtime pair for that clone lineage.

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

The MSRV is the compatibility contract for downstream users. The exact toolchain pin exists to keep local execution, CI, and reproducible validation aligned.

## Quality Gates

The main CI lane enforces formatting, baseline Clippy, workspace tests, `nextest`, docs builds with rustdoc warnings denied, typo checks, dependency-policy checks for bans, licenses, and sources, and a depth-1 feature matrix for the published crate family.

The workspace manifest also defines focused Clippy policy for documented failure contracts, discard-prone helper returns, and readable large literals through `missing_errors_doc`, `missing_panics_doc`, `must_use_candidate`, and `unreadable_literal`.

CI also enforces public API rustc lints with `missing_docs`, `missing_debug_implementations`, `unreachable_pub`, and `unnameable_types` across the published crate family: `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, `cow-sdk-trading`, `cow-sdk-browser-wallet`, and the `cow-sdk` facade. RustSec advisories continue to run as a separate report.

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
- [ADRs](docs/adr/README.md)

## Validation

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo nextest run --workspace --all-features --config-file .github/config/nextest.toml
cargo doc --workspace --all-features --no-deps
cargo hack check --workspace --feature-powerset --depth 1
typos --config .github/config/typos.toml
cargo deny check bans licenses sources --config .github/config/deny.toml
cargo check -p cow-sdk --examples
cargo build --target wasm32-unknown-unknown -p cow-sdk --features browser-wallet
cargo package -p cow-sdk --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-trading.path='crates/trading'" --config "patch.crates-io.cow-sdk-browser-wallet.path='crates/browser-wallet'"
```

```text
RUSTFLAGS="-Wmissing-docs -Wmissing-debug-implementations -Wunreachable-pub -Wunnameable-types" cargo check --workspace --all-features
cargo deny check advisories --config .github/config/deny.toml
```

## Examples

- `examples/native/` contains native SDK scenarios.
- `examples/native/scenarios/subgraph_query_roundtrip.rs`, `subgraph_custom_query_roundtrip.rs`, and `subgraph_live_query.rs` cover canonical helper, custom-query, and opt-in live subgraph usage through `cow-sdk-subgraph`.
- `examples/wasm/sdk-verification-console/` contains deterministic WASM checks and a browser review surface for SDK verification.
- `examples/wasm/browser-wallet-console/` contains deterministic mock-wallet proof mode and explicit injected-provider browser flows.
