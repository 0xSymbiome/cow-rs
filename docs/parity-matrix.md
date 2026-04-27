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
| Order creation, signing, and submission | `cowprotocol/cow-sdk` trading, order-signing, order-book, and sdk packages | `cow-sdk-signing`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk` | `parity/fixtures/signing.json`, `parity/fixtures/orderbook.json`, `parity/fixtures/trading.json`, `parity/fixtures/sdk.json` | `crates/signing/tests/order_signing_contract.rs`, `crates/orderbook/tests/api_contract.rs`, `crates/trading/tests/post_contract.rs`, `crates/trading/tests/sdk_contract.rs`, `crates/sdk/tests/public_api.rs` |
| Contracts parity | `cowprotocol/contracts` plus selected `cow-sdk` contract helpers | `cow-sdk-contracts`, `cow-sdk-signing` | `parity/fixtures/contracts.json` | `crates/contracts/tests/order_contract.rs`, `crates/contracts/tests/settlement_contract.rs`, `crates/contracts/tests/reader_contract.rs`, `crates/contracts/tests/parity_contract.rs`, `crates/signing/tests/eip1271_contract.rs` |
| Codec fuzz corpora | `cowprotocol/contracts` order UID helpers plus selected `cowprotocol/cow-sdk` typed-data helpers | `cow-sdk-contracts`, `cow-sdk-signing` | `fuzz/corpus/fuzz_order_uid_pack_unpack/` (six 56-byte triples), `fuzz/corpus/fuzz_typed_data_digest/` (five 200-byte inputs), `parity/fixtures/contracts.json`, `parity/fixtures/signing.json` | `fuzz/fuzz_targets/fuzz_order_uid_pack_unpack.rs`, `fuzz/fuzz_targets/fuzz_typed_data_digest.rs`, `cargo fuzz run fuzz_order_uid_pack_unpack --runs 65536`, `cargo fuzz run fuzz_typed_data_digest --runs 65536` |
| `GPv2Settlement` bindings | `cowprotocol/contracts` settlement surface | `cow-sdk-contracts::settlement` via `alloy::sol!` | Solidity excerpt under `crates/contracts/abi/settlement/` | `crates/contracts/tests/parity_contract.rs::settlement_calldata_matches_upstream_fixtures` |
| `GPv2VaultRelayer` bindings | `cowprotocol/contracts` vault-relayer surface | `cow-sdk-contracts::vault` via `alloy::sol!` | Solidity excerpt under `crates/contracts/abi/vault-relayer/` | `crates/contracts/tests/parity_contract.rs::vault_relayer_calldata_matches_upstream_fixtures` |
| `CoWSwapEthFlow` bindings | `cowprotocol/ethflowcontract` surface | `cow-sdk-contracts::eth_flow` via `alloy::sol!` | Solidity excerpt under `crates/contracts/abi/eth-flow/` | `crates/contracts/tests/parity_contract.rs::eth_flow_create_and_invalidate_calldata_match_upstream_fixtures` |
| EIP-1967 proxy-slot surface | ERC-1967 standard plus selected `cowprotocol/contracts` proxy usage | `cow-sdk-contracts::proxy` via `alloy::sol!` | Solidity excerpt under `crates/contracts/abi/eip1967/` | `crates/contracts/tests/parity_contract.rs::eip1967_slot_reads_match_upstream_fixtures` |
| ERC-20 and ERC-20 Permit bindings | ERC-20 and EIP-2612 standards | `cow-sdk-contracts::erc20` via `alloy::sol!` | Solidity excerpt under `crates/contracts/abi/erc20/` | `crates/contracts/tests/parity_contract.rs::erc20_and_permit_calldata_match_upstream_fixtures` |
| Deployment registry authority | `cowprotocol/contracts` deployments record | `cow-sdk-contracts::Registry` via embedded `registry.toml` | `crates/contracts/registry.toml` | `crates/contracts/tests/registry.rs`, `crates/contracts/tests/build_rs_compile_fail.rs` |
| App-data parity | `cowprotocol/cow-sdk` app-data package and schema inputs | `cow-sdk-app-data`, `cow-sdk-trading` | `parity/fixtures/app-data.json` | `crates/app-data/tests/cid_contract.rs`, `crates/app-data/tests/schema_contract.rs`, `crates/app-data/tests/pinning_contract.rs`, `crates/trading/tests/quote_contract.rs` |
| Subgraph support | `cowprotocol/cow-sdk` subgraph package | `cow-sdk-subgraph` | `parity/fixtures/subgraph.json` | `crates/subgraph/tests/api_contract.rs`, `crates/subgraph/tests/query_contract.rs`, `crates/subgraph/tests/types_contract.rs` |
| Orderbook transport | `cowprotocol/cow-sdk` order-book package plus selected `cowprotocol/services` references | `cow-sdk-orderbook` | `parity/fixtures/orderbook.json` | `crates/orderbook/tests/api_contract.rs`, `crates/orderbook/tests/request_contract.rs`, `crates/orderbook/tests/transform_contract.rs`, `crates/orderbook/tests/types_contract.rs` |
| WASM target | `cowprotocol/cow-sdk` sdk package | `cow-sdk`, `cow-sdk-app-data`, WASM examples | `parity/fixtures/sdk.json`, committed workflow definitions, example READMEs | `examples/wasm/sdk-verification-console/tests/deterministic_exports.rs`, `wasm-pack test --headless --chrome`, `bun run --cwd e2e/sdk-verification test` |
| Browser wallet integration | selected `cowprotocol/cow-sdk` common, provider, trading, and sdk paths | `cow-sdk-browser-wallet`, `cow-sdk` | `examples/wasm/browser-wallet-console/README.md`, `docs/validation-scope.md` | `crates/browser-wallet/tests/provider_contract.rs`, `crates/browser-wallet/tests/wallet_contract.rs`, direct browser-bridge proof, and committed browser-wallet console automation |

## Orderbook Rejection Tags

`OrderbookRejection` models 49 variants including the
forward-compatible `Unknown` fallback. The GET-side trade-filter and
pagination tags below are represented directly and preserve services wire
spelling.

| Services wire tag | Rust variant | Primary upstream producer | Primary evidence |
| --- | --- | --- | --- |
| `InvalidTradeFilter` | `OrderbookRejection::InvalidTradeFilter` | `cowprotocol/services` orderbook trade lookup filters | `crates/orderbook/tests/rejection_contract.rs::every_known_services_tag_parses_to_its_typed_variant` |
| `InvalidLimit` | `OrderbookRejection::InvalidLimit` | `cowprotocol/services` orderbook trade pagination limits | `crates/orderbook/tests/rejection_contract.rs::every_known_services_tag_parses_to_its_typed_variant` |
| `LIMIT_OUT_OF_BOUNDS` | `OrderbookRejection::LimitOutOfBounds` | `cowprotocol/services` user-order lookup pagination limits | `crates/orderbook/tests/rejection_contract.rs::every_known_services_tag_parses_to_its_typed_variant` |

## Trading helper defaults

| Surface | Default | Opt-out / opt-in |
| --- | --- | --- |
| `OrderToSignParams::new(...)` `apply_costs_slippage_and_fees` | applied on by default (cost, slippage, partner-fee, and protocol-fee adjustments are folded into the unsigned order amounts) | call `.with_apply_costs_slippage_and_fees(false)` to preserve raw caller amounts |
| `build_app_data` `metadata.utm` | when the caller does not supply `metadata.utm`, the helper stamps a Rust-identified attribution block with `utmSource = "cowmunity"`, `utmMedium = "cow-rs@<crate-version>"`, `utmCampaign = "developer-cohort"`, `utmContent = ""`, and `utmTerm = "rs"` so downstream analytics can attribute traffic to the Rust SDK and its published version | supply any `metadata.utm` key in the advanced app-data parameters — partial or full — and the caller-declared block is carried through byte-identical with no defaults merged on top |

## Orderbook DTO defaults

| Surface | Default | Legacy access |
| --- | --- | --- |
| `Order.total_fee` | computed narrowly as the canonical executed-fee component (`calculate_total_fee(executed_fee)`); the legacy wire field `executedFeeAmount` is never folded into the canonical sum | `Order.executed_fee_amount_legacy: Option<String>` surfaces the legacy wire value as a typed read-only sibling so consumers that need the legacy summation compute `executed_fee + executed_fee_amount_legacy` explicitly at the call site |

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

`cow-sdk-transport-wasm` is the shipped browser-target `HttpTransport`
adapter and is consumed through the workspace rather than through the
first-party publish sequence above; the exact verification commands
are recorded in [Release Checklist](release-checklist.md).
