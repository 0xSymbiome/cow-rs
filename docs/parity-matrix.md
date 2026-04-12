# Parity Matrix

Authority order:

1. `parity/source-lock.yaml`
2. `docs/parity-sources.md`
3. `docs/validation-scope.md`
4. `docs/release-checklist.md`
5. committed parity fixtures and executable tests

## Surface Matrix

| Surface | Primary upstream producers | Rust crates | Committed authority | Primary executable evidence | Main release commands |
| --- | --- | --- | --- | --- | --- |
| Order creation, signing, and submission | `cow-protocol/cow-sdk/packages/trading/src/*`, `packages/order-signing/src/*`, `packages/order-book/src/*`, `packages/sdk/src/*`, `packages/sdk/package.json`, `packages/sdk/README.md` | `cow-sdk-signing`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk` | `parity/fixtures/signing.json`, `parity/fixtures/orderbook.json`, `parity/fixtures/trading.json`, `parity/fixtures/sdk.json` | `crates/signing/tests/order_signing_contract.rs`, `crates/orderbook/tests/api_contract.rs`, `crates/trading/tests/post_contract.rs`, `crates/trading/tests/sdk_contract.rs`, `crates/sdk/tests/public_api.rs` | `cargo test -p cow-sdk-signing`, `cargo test -p cow-sdk-orderbook`, `cargo test -p cow-sdk-trading`, `cargo test -p cow-sdk` |
| Contracts parity | `cow-protocol/contracts/src/ts/*`, selected `cow-sdk/packages/contracts-ts/src/*` and `tests/*` | `cow-sdk-contracts`, `cow-sdk-signing` | `parity/fixtures/contracts.json` | `crates/contracts/tests/order_contract.rs`, `crates/contracts/tests/settlement_contract.rs`, `crates/contracts/tests/reader_contract.rs`, `crates/signing/tests/eip1271_contract.rs` | `cargo test -p cow-sdk-contracts`, `cargo clippy -p cow-sdk-contracts --all-targets --all-features -- -D warnings` |
| App-data parity | `cow-protocol/cow-sdk/packages/app-data/src/api/*`, `src/types.ts`, `src/consts.ts`, `src/importSchema.ts`, `src/generatedTypes/*`, selected `packages/app-data/test/*` | `cow-sdk-app-data`, `cow-sdk-trading` | `parity/fixtures/app-data.json` | `crates/app-data/tests/cid_contract.rs`, `crates/app-data/tests/schema_contract.rs`, `crates/app-data/tests/pinning_contract.rs`, `crates/trading/tests/quote_contract.rs` | `cargo test -p cow-sdk-app-data`, `cargo clippy -p cow-sdk-app-data --all-targets --all-features -- -D warnings` |
| Subgraph support | `cow-protocol/cow-sdk/packages/subgraph/src/api.ts`, `queries.ts`, `graphql.ts`, `api.spec.ts` | `cow-sdk-subgraph` | `parity/fixtures/subgraph.json` | `crates/subgraph/tests/api_contract.rs`, `crates/subgraph/tests/query_contract.rs`, `crates/subgraph/tests/types_contract.rs` | `cargo test -p cow-sdk-subgraph` |
| Blockchain fetch and decode | `cow-protocol/cow-sdk/packages/order-book/src/*`, selected `cow-protocol/services` orderbook and validation sources | `cow-sdk-orderbook` | `parity/fixtures/orderbook.json` | `crates/orderbook/tests/api_contract.rs`, `crates/orderbook/tests/request_contract.rs`, `crates/orderbook/tests/transform_contract.rs`, `crates/orderbook/tests/types_contract.rs` | `cargo test -p cow-sdk-orderbook`, `cargo test --workspace` |
| WASM target | `cow-protocol/cow-sdk/packages/sdk/src/index.ts`, `packages/sdk/src/typedoc-entry.ts` | `cow-sdk`, `cow-sdk-app-data`, WASM examples | `parity/fixtures/sdk.json`, `.github/workflows/wasm.yml`, `.github/workflows/sdk-verification-e2e.yml` | `examples/wasm/sdk-verification-console/tests/deterministic_exports.rs`, `wasm-pack test --headless --chrome`, `bun run --cwd e2e/sdk-verification test` | `cargo build --target wasm32-unknown-unknown -p cow-sdk`, `cargo build --target wasm32-unknown-unknown -p cow-sdk-app-data`, `cargo check --target wasm32-unknown-unknown -p cow-sdk --examples` |
| Quality and publishability | `cow-protocol/cow-sdk/packages/sdk/src/*`, package test producers already pinned in the leaf fixtures | `cow-sdk` workspace | `docs/validation-scope.md`, `docs/release-checklist.md`, `docs/parity-sources.md` | Formatting, linting, tests, docs, source-lock validation, and package dry-run chain recorded in `docs/release-checklist.md` | `cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- validate --source-lock parity/source-lock.yaml`, published package dry-run chain in `docs/release-checklist.md` |
| Browser wallet integration | `cow-protocol/cow-sdk/packages/common/src/adapters/*`, `packages/providers/*`, `packages/trading/src/utils/resolveSigner.ts`, `packages/sdk/src/typedoc-entry.ts` | `cow-sdk-browser-wallet`, `cow-sdk` | `examples/wasm/browser-wallet-console/README.md`, `docs/validation-scope.md` | `crates/browser-wallet/tests/provider_contract.rs`, `crates/browser-wallet/tests/wallet_contract.rs`, browser-wallet console mock mode, and browser-wallet WASM build | `cargo test -p cow-sdk-browser-wallet`, `cargo build --target wasm32-unknown-unknown -p cow-sdk --features browser-wallet`, `cargo build --target wasm32-unknown-unknown --manifest-path examples/wasm/browser-wallet-console/Cargo.toml` |

## Provenance anchors

- Global source contract: `parity/source-lock.yaml`
- Surface ownership and upstream paths: `docs/parity-sources.md`
- Scope-to-proof mapping: `docs/validation-scope.md`
- Packaging and release verification: `docs/release-checklist.md`

## Publish-order dry-run targets

The workspace publish order used for dry-run verification is:

1. `cow-sdk-core`
2. `cow-sdk-contracts`
3. `cow-sdk-app-data`
4. `cow-sdk-orderbook`
5. `cow-sdk-signing`
6. `cow-sdk-subgraph`
7. `cow-sdk-trading`
8. `cow-sdk-browser-wallet`
9. `cow-sdk`

Those dry runs use explicit `patch.crates-io` overrides during verification for unpublished
internal dependencies. The exact commands are recorded in `docs/release-checklist.md`.
