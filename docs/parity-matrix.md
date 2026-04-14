# Parity Matrix

Authority order:

1. `parity/source-lock.yaml`
2. `docs/parity-sources.md`
3. `docs/validation-scope.md`
4. `docs/release-checklist.md`
5. committed parity fixtures and executable tests

## Surface Matrix

| Surface | Primary upstream producers | Rust crates | Committed authority | Primary evidence |
| --- | --- | --- | --- | --- |
| Order creation, signing, and submission | `cow-protocol/cow-sdk` trading, order-signing, order-book, and sdk packages | `cow-sdk-signing`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk` | `parity/fixtures/signing.json`, `parity/fixtures/orderbook.json`, `parity/fixtures/trading.json`, `parity/fixtures/sdk.json` | `crates/signing/tests/order_signing_contract.rs`, `crates/orderbook/tests/api_contract.rs`, `crates/trading/tests/post_contract.rs`, `crates/trading/tests/sdk_contract.rs`, `crates/sdk/tests/public_api.rs` |
| Contracts parity | `cow-protocol/contracts` plus selected `cow-sdk` contract helpers | `cow-sdk-contracts`, `cow-sdk-signing` | `parity/fixtures/contracts.json` | `crates/contracts/tests/order_contract.rs`, `crates/contracts/tests/settlement_contract.rs`, `crates/contracts/tests/reader_contract.rs`, `crates/signing/tests/eip1271_contract.rs` |
| App-data parity | `cow-protocol/cow-sdk` app-data package and schema inputs | `cow-sdk-app-data`, `cow-sdk-trading` | `parity/fixtures/app-data.json` | `crates/app-data/tests/cid_contract.rs`, `crates/app-data/tests/schema_contract.rs`, `crates/app-data/tests/pinning_contract.rs`, `crates/trading/tests/quote_contract.rs` |
| Subgraph support | `cow-protocol/cow-sdk` subgraph package | `cow-sdk-subgraph` | `parity/fixtures/subgraph.json` | `crates/subgraph/tests/api_contract.rs`, `crates/subgraph/tests/query_contract.rs`, `crates/subgraph/tests/types_contract.rs` |
| Orderbook transport | `cow-protocol/cow-sdk` order-book package plus selected `cow-protocol/services` references | `cow-sdk-orderbook` | `parity/fixtures/orderbook.json` | `crates/orderbook/tests/api_contract.rs`, `crates/orderbook/tests/request_contract.rs`, `crates/orderbook/tests/transform_contract.rs`, `crates/orderbook/tests/types_contract.rs` |
| WASM target | `cow-protocol/cow-sdk` sdk package | `cow-sdk`, `cow-sdk-app-data`, WASM examples | `parity/fixtures/sdk.json`, committed workflow definitions, example READMEs | `examples/wasm/sdk-verification-console/tests/deterministic_exports.rs`, `wasm-pack test --headless --chrome`, `bun run --cwd e2e/sdk-verification test` |
| Browser wallet integration | selected `cow-protocol/cow-sdk` common, provider, trading, and sdk paths | `cow-sdk-browser-wallet`, `cow-sdk` | `examples/wasm/browser-wallet-console/README.md`, `docs/validation-scope.md` | `crates/browser-wallet/tests/provider_contract.rs`, `crates/browser-wallet/tests/wallet_contract.rs`, direct browser-bridge proof, and committed browser-wallet console automation |

## Provenance Anchors

- Global source contract: `parity/source-lock.yaml`
- Surface ownership and upstream paths: [Parity Sources](parity-sources.md)
- Scope-to-proof mapping: [Validation Scope](validation-scope.md)
- Packaging and release verification: [Release Checklist](release-checklist.md)

## Publish Order

The published crate-family dry-run order is:

1. `cow-sdk-core`
2. `cow-sdk-contracts`
3. `cow-sdk-app-data`
4. `cow-sdk-orderbook`
5. `cow-sdk-signing`
6. `cow-sdk-subgraph`
7. `cow-sdk-trading`
8. `cow-sdk-browser-wallet`
9. `cow-sdk`

The exact verification commands are recorded in
[Release Checklist](release-checklist.md).
