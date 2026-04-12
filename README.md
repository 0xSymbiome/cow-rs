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

- Public MSRV: Rust `1.94.0`
- Contributor toolchain pin: Rust `1.94.1` in [rust-toolchain.toml](rust-toolchain.toml)

The public compatibility floor is exercised directly with `cargo check --workspace --all-features` and `cargo test --workspace` on Rust `1.94.0`.

Primary CI and release validation use the pinned `1.94.1` contributor toolchain for formatting, Clippy, library and integration tests, an explicit workspace doctest lane, docs, feature-matrix checks, and repo-local publication verification. CI also checks the SDK and native example surfaces. Release-readiness adds a separate pinned-upstream provenance lane. WASM and browser-wallet target validation stay in the dedicated WASM workflows so native compatibility checks do not inherit browser-specific assumptions.

A separate Windows stable lane runs `cargo check --workspace --all-features` and `cargo test --workspace --lib --tests` on `windows-latest`. It is intentionally limited to native workspace compatibility; browser-target and publication-specific validation stay on their dedicated lanes.

## Quality Gates

The primary native CI lane runs on the pinned `1.94.1` contributor toolchain and enforces formatting, baseline Clippy, workspace library and integration tests, a dedicated workspace doctest lane, `nextest`, docs builds with rustdoc warnings denied, typo checks, dependency-policy checks for bans, licenses, and sources, and a depth-1 feature matrix for the published crate family.

A separate compatibility-floor lane runs `cargo check --workspace --all-features` and `cargo test --workspace` on Rust `1.94.0`.

A separate Windows stable lane runs `cargo check --workspace --all-features` and `cargo test --workspace --lib --tests` on `windows-latest` so the declared native host surface is exercised outside the Linux-only publication and provenance lanes.

Security analysis runs in a dedicated `codeql.yml` workflow that scans both Rust and GitHub Actions on pull requests, pushes to `main` and `develop`, and a weekly schedule. It complements the dependency-policy lane instead of replacing `cargo-deny` or `cargo-audit`.

Documentation quality also has a dedicated `docs-quality.yml` workflow. It keeps the stable docs build in the primary CI lane, adds all-feature doctests, and runs a nightly docs.rs-style rustdoc build with `docsrs` cfg plus nightly-only rustdoc presentation flags.

The routine PR-blocking workflow also publishes a final `ci-success` status that aggregates the required native quality, compatibility, and publication jobs. Workflow jobs use explicit timeout budgets, and checkout steps in the native validation workflows disable credential persistence.

Crate-isolation maintenance runs separately in `crate-checks.yml` on a schedule and manual dispatch. It uses `cargo hack check --workspace --each-feature --no-dev-deps` to catch crate-level dependency and feature assumptions that workspace unification can hide, without turning that maintenance-depth check into a routine PR requirement.

The workspace manifest also defines focused Clippy policy for documented failure contracts, discard-prone helper returns, and readable large literals through `missing_errors_doc`, `missing_panics_doc`, `must_use_candidate`, and `unreadable_literal`.

Duplicate-version maintenance is reviewed through `cargo-deny` plus `cargo tree -d --workspace`, where every accepted subtree stays explicit in `.github/config/deny.toml`. The maintenance-depth Clippy sweep keeps `clippy::multiple_crate_versions` out of that command so duplicate-version review stays anchored to the curated dependency policy instead of the coarse global lint.

CI also enforces public API rustc lints with `missing_docs`, `missing_debug_implementations`, `unreachable_pub`, and `unnameable_types` across the published crate family: `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-subgraph`, `cow-sdk-trading`, `cow-sdk-browser-wallet`, and the `cow-sdk` facade.

Publication verification is split deliberately:

- `ci.yml` runs repo-local source-lock validation and the full published package-family dry-run from the current workspace.
- `release-readiness.yml` reruns that repo-local publication contract and then provisions pinned independent upstream clones from `parity/source-lock.yaml` before running the stricter explicit-root provenance check.

Dependency policy is split by purpose:

- `cargo-deny` enforces bans, licenses, sources, and the approved duplicate-version tolerances documented in `.github/config/deny.toml`
- `cargo-audit` enforces RustSec advisories without depending on the current `cargo-deny` advisory-db parser path
- `RUSTSEC-2026-0097` is temporarily ignored because the remaining hit is inherited from the `ethabi` stack used by `cow-sdk-browser-wallet`
- the approved duplicate tolerances are limited to the browser-wallet `ethabi` subtree, the test-only subgraph `graphql_client` subtree, and the platform-specific verifier subtree under `rustls-platform-verifier`
- dependency freshness reporting is separate and read-only: `release-readiness.yml` runs it weekly and on manual dispatch, while `ci.yml` exposes the same report on manual dispatch only
- the freshness report is built directly in the workflow from `cargo update --dry-run` plus `cargo tree -d --workspace`, so it surfaces lockfile movement opportunities without rewriting `Cargo.lock` or introducing a repo-side maintenance script language

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
cargo clippy --workspace --all-targets --all-features --message-format short -- -W clippy::pedantic -W clippy::cargo -A clippy::multiple_crate_versions
cargo test --workspace
cargo test --workspace --doc
cargo test --all-features --workspace --doc
cargo +1.94.0 check --workspace --all-features
cargo +1.94.0 test --workspace
cargo nextest run --workspace --all-features --config-file .github/config/nextest.toml
cargo doc --workspace --all-features --no-deps
cargo hack check --workspace --feature-powerset --depth 1
typos --config .github/config/typos.toml
cargo deny check bans licenses sources --config .github/config/deny.toml
cargo audit --deny warnings --ignore RUSTSEC-2026-0097
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml
cargo check -p cow-sdk --examples
cargo build --target wasm32-unknown-unknown -p cow-sdk --features browser-wallet
cargo package -p cow-sdk --allow-dirty --config "patch.crates-io.cow-sdk-core.path='crates/core'" --config "patch.crates-io.cow-sdk-contracts.path='crates/contracts'" --config "patch.crates-io.cow-sdk-signing.path='crates/signing'" --config "patch.crates-io.cow-sdk-app-data.path='crates/app-data'" --config "patch.crates-io.cow-sdk-orderbook.path='crates/orderbook'" --config "patch.crates-io.cow-sdk-trading.path='crates/trading'" --config "patch.crates-io.cow-sdk-browser-wallet.path='crates/browser-wallet'"
```

```text
DOCS_RS=1 RUSTDOCFLAGS="--cfg docsrs -D warnings -Zunstable-options --generate-link-to-definition --show-type-layout --enable-index-page" cargo +nightly doc --workspace --all-features --no-deps
```

```text
RUSTFLAGS="-Wmissing-docs -Wmissing-debug-implementations -Wunreachable-pub -Wunnameable-types" cargo check --workspace --all-features
```

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- provision-upstreams --source-lock parity/source-lock.yaml --output-root <path>
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml --cow-sdk-root <path>/cow-sdk --contracts-root <path>/contracts --services-root <path>/services
```

```text
cargo update --dry-run --color never
cargo tree -d --workspace
```

## Examples

- `examples/native/` contains native SDK scenarios.
- `examples/native/scenarios/subgraph_query_roundtrip.rs`, `subgraph_custom_query_roundtrip.rs`, and `subgraph_live_query.rs` cover canonical helper, custom-query, and opt-in live subgraph usage through `cow-sdk-subgraph`.
- `examples/wasm/sdk-verification-console/` contains deterministic WASM checks and a browser review surface for SDK verification.
- `examples/wasm/browser-wallet-console/` contains deterministic mock-wallet proof mode and explicit injected-provider browser flows.
