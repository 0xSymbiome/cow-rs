# Parity Matrix

Authority order:

1. `parity/source-lock.yaml`
2. `docs/parity-sources.md`
3. `docs/validation-scope.md`
4. `docs/release-checklist.md`
5. committed parity fixtures and executable tests

## Upstream Authority Model

The primary parity authorities are the producers that define the protocol
contract: `cowprotocol/services` for the off-chain orderbook API, OpenAPI
schemas, wire DTOs, and validation semantics, and `cowprotocol/contracts` (with
`cowprotocol/ethflowcontract`) for on-chain EIP-712 hashing, ABI, and
addresses. The upstream TypeScript `cowprotocol/cow-sdk` is a cross-language
reference for consumer-workflow and ergonomic coverage; it does not govern the
Rust public API shape or the wire format. In the "Primary upstream producers"
column below, a `cow-sdk` entry names the workflow a surface mirrors, while the
wire and on-chain shapes that surface must match are owned by services and the
contracts. See [Parity Sources](parity-sources.md#source-ownership) for the
full ownership split.

## Supported Networks

The Rust SDK supports the CoW Protocol chains enumerated by
`cow_sdk_core::config::SupportedChainId`. Per-chain numeric ids, deployment
provenance, services-generated metadata, TypeScript SDK support, and
wrapped-native-token evidence are maintained in the
[Deployment Registry Audit](audit/deployment-registry-audit.md#per-chain-provenance)
instead of being repeated here.

Endpoint discovery via the `OrderbookApi::builder()` and `SubgraphApi::builder()`
typestate chains — each given the chain id and environment — continues to honor
production versus staging environment selection through the typed API context.

## Surface Matrix

| Surface | Primary upstream producers | Rust crates | Committed authority | Primary evidence |
| --- | --- | --- | --- | --- |
| Order creation, signing, and submission | `cowprotocol/cow-sdk` trading, order-signing, order-book, and sdk packages | `cow-sdk-signing`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk` | `parity/fixtures/orderbook.json`, `parity/fixtures/trading.json` | `crates/signing/tests/order_signing_contract.rs`, `crates/orderbook/tests/api_contract.rs`, `crates/trading/tests/post_contract.rs`, `crates/trading/tests/sdk_contract.rs`, `crates/sdk/tests/public_api.rs`, `crates/sdk/tests/public_api_default_features_only.rs`, `crates/sdk/tests/public_api_with_all_features.rs` |
| Contracts parity | `cowprotocol/contracts` plus selected `cow-sdk` contract helpers | `cow-sdk-contracts`, `cow-sdk-signing` | `parity/fixtures/contracts.json` | `crates/contracts/tests/order_contract.rs`, `crates/contracts/tests/settlement_contract.rs`, `crates/contracts/tests/reader_contract.rs`, `crates/contracts/tests/parity_contract.rs`, `crates/signing/tests/eip1271_contract.rs` |
| Codec fuzz corpora | `cowprotocol/contracts` order UID helpers plus selected `cowprotocol/cow-sdk` typed-data helpers | `cow-sdk-contracts`, `cow-sdk-signing` | `fuzz/corpus/fuzz_order_uid_pack_unpack/` (six 56-byte triples), `fuzz/corpus/fuzz_typed_data_digest/` (five 200-byte inputs), `parity/fixtures/contracts.json` | `fuzz/fuzz_targets/fuzz_order_uid_pack_unpack.rs`, `fuzz/fuzz_targets/fuzz_typed_data_digest.rs`, `cargo fuzz run fuzz_order_uid_pack_unpack --runs 65536`, `cargo fuzz run fuzz_typed_data_digest --runs 65536` |
| `GPv2Settlement` bindings | `cowprotocol/contracts` settlement surface | `cow-sdk-contracts::settlement` via `alloy::sol!` | Byte-identical Solidity mirror under `crates/contracts/abi/settlement/` gated by `cargo parity-verify-sol-provenance` | `crates/contracts/tests/parity_contract.rs::settlement_calldata_matches_upstream_fixtures` |
| `GPv2VaultRelayer` bindings | `cowprotocol/contracts` vault-relayer surface | `cow-sdk-contracts::vault` via `alloy::sol!` | Byte-identical Solidity mirror under `crates/contracts/abi/vault-relayer/` gated by `cargo parity-verify-sol-provenance` | `crates/contracts/tests/parity_contract.rs::vault_relayer_calldata_matches_upstream_fixtures` |
| `CoWSwapEthFlow` bindings | `cowprotocol/ethflowcontract` surface | `cow-sdk-contracts::eth_flow` via `alloy::sol!` | Byte-identical Solidity mirror under `crates/contracts/abi/eth-flow/` gated by `cargo parity-verify-sol-provenance` | `crates/contracts/tests/parity_contract.rs::eth_flow_create_and_invalidate_calldata_match_upstream_fixtures` |
| `CoWSwapOnchainOrders` event decoder | `cowprotocol/ethflowcontract` `CoWSwapOnchainOrders` mixin and interface | `cow-sdk-contracts::onchain_orders` via `alloy::sol!` | Byte-identical Solidity mirror under `crates/contracts/abi/eth-flow/` gated by `cargo parity-verify-sol-provenance` | `crates/contracts/tests/onchain_orders.rs::order_placement_topic0_matches_canonical_hash`, `crates/contracts/tests/onchain_orders.rs::order_hash_matches_canonical_ethflow_foundry_vector` |
| `IWrappedNativeToken` (WETH9-family) bindings | `cowprotocol/ethflowcontract` `IWrappedNativeToken` interface | `cow-sdk-contracts::weth` via `alloy::sol!` | Byte-identical Solidity mirror under `crates/contracts/abi/weth/` gated by `cargo parity-verify-sol-provenance` | `crates/contracts/tests/weth.rs::deposit_selector_matches_canonical_keccak`, `crates/contracts/tests/weth.rs::withdraw_selector_matches_canonical_keccak` |
| EIP-1967 proxy-slot surface | `cowprotocol/contracts` `GPv2EIP1967` library carrying the ERC-1967 storage-slot derivation | `cow-sdk-contracts::proxy` via `alloy::sol!` | Byte-identical Solidity mirror under `crates/contracts/abi/eip1967/` gated by `cargo parity-verify-sol-provenance` | `crates/contracts/tests/parity_contract.rs::eip1967_slot_reads_match_upstream_fixtures` |
| ERC-20 and ERC-20 Permit bindings | `cowprotocol/contracts` `IERC20` interface (carrying its own OpenZeppelin v3.4.0 lineage in the upstream header) plus the EIP-2612 `permit` extension inline-declared in `cow-sdk-contracts::erc20` | `cow-sdk-contracts::erc20` via `alloy::sol!` | Byte-identical Solidity mirror of `IERC20` under `crates/contracts/abi/erc20/` gated by `cargo parity-verify-sol-provenance`; the `IERC20Permit` interface is declared inline in `crates/contracts/src/erc20.rs` since EIP-2612 has no canonical upstream pinned in `parity/source-lock.yaml` | `crates/contracts/tests/parity_contract.rs::erc20_and_permit_calldata_match_upstream_fixtures` |
| Deployment registry authority | `cowprotocol/contracts` deployments record | `cow-sdk-contracts::Registry` via embedded `registry.toml` | `crates/contracts/registry.toml` | `crates/contracts/tests/registry.rs`, `crates/contracts/tests/schema_v2_rejection.rs` |
| App-data parity | `cowprotocol/cow-sdk` app-data package and schema inputs | `cow-sdk-app-data`, `cow-sdk-trading` | `parity/fixtures/app_data/` | `crates/app-data/tests/cid_contract.rs`, `crates/app-data/tests/schema_contract.rs`, `crates/app-data/tests/fetch_contract.rs`, `crates/trading/tests/quote_contract.rs` |
| Subgraph support | `cowprotocol/cow-sdk` subgraph package | `cow-sdk-subgraph` | `crates/subgraph/tests/schema_evidence/schema.graphql` | `crates/subgraph/tests/api_contract.rs`, `crates/subgraph/tests/query_contract.rs`, `crates/subgraph/tests/types_contract.rs` |
| Orderbook transport | `cowprotocol/cow-sdk` order-book package plus selected `cowprotocol/services` references | `cow-sdk-orderbook` | `parity/fixtures/orderbook.json`, `parity/openapi/coverage.yaml` | `crates/orderbook/tests/api_contract.rs`, `crates/orderbook/tests/request_contract.rs`, `crates/orderbook/tests/transform_contract.rs`, `crates/orderbook/tests/types_contract.rs`, `crates/orderbook/tests/openapi_dto_coverage.rs` |
| WASM target | `cowprotocol/cow-sdk` sdk package | `cow-sdk`, `cow-sdk-app-data`, the WASM example | committed workflow definitions, example READMEs | `crates/transport-wasm/tests/wasm.rs`, `wasm-pack test --headless --firefox`, and the `wasm.yml` compatibility workflow |
| WASM event-log decoders | `cowprotocol/contracts` settlement surface and `cowprotocol/ethflowcontract` mixin | `cow-sdk-wasm` `decodeSettlementLog` / `decodeEthFlowLog` over the `cow-sdk-contracts` decoders | Facade and raw TypeScript declaration snapshots under `crates/wasm/snapshots/` | `crates/wasm/tests/wasm_facade_snapshot_contract.rs::facade_declarations_match_flavour_matrix` |
| Browser wallet integration | selected `cowprotocol/cow-sdk` common, provider, trading, and sdk paths | `cow-sdk-browser-wallet`, `cow-sdk` | `examples/wasm/cow-trader-dioxus/README.md`, `docs/validation-scope.md` | `crates/browser-wallet/tests/provider_contract.rs`, `crates/browser-wallet/tests/wallet_contract.rs`, the direct browser-bridge proof, and the canonical browser-wallet example |
| Native Alloy adapters | `alloy-rs/alloy` and `alloy-rs/core` source-lock pins plus local trait contracts | `cow-sdk-alloy-provider`, `cow-sdk-alloy-signer`, `cow-sdk-alloy`, `cow-sdk` opt-in features | `parity/source-lock.yaml`, `docs/providers/adapting-alloy.md`, `examples/native/README.md` | `crates/alloy-provider/tests/*`, `crates/alloy-signer/tests/*`, `crates/alloy/tests/*`, `tests/alloy_umbrella_composition.rs` |

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

The `metadata.utm` row below is a local Rust SDK attribution policy rather
than an upstream fixture vector. It is intentionally asserted by
`crates/trading/tests/quote_contract.rs::default_utm_block_uses_env_cargo_pkg_version`
and not carried in `parity/fixtures/trading.json`.

| Surface | Default | Opt-out / opt-in |
| --- | --- | --- |
| `OrderToSignParams::new(...)` `apply_costs_slippage_and_fees` | applied on by default (cost, slippage, partner-fee, and protocol-fee adjustments are folded into the unsigned order amounts) | call `.with_apply_costs_slippage_and_fees(false)` to preserve raw caller amounts |
| `build_app_data` `metadata.utm` | when the caller does not supply `metadata.utm`, the helper stamps an SDK-family attribution block with `utmSource = "cow-sdk"`, `utmMedium = "cow-rs@<crate-version>"`, `utmCampaign = "developer-cohort"`, `utmContent = ""`, and `utmTerm = "rs"` so downstream analytics can group CoW SDK traffic while distinguishing the Rust SDK and its published version | supply any `metadata.utm` key in the advanced app-data parameters — partial or full — and the caller-declared block is carried through byte-identical with no defaults merged on top |

## Orderbook DTO defaults

| Surface | Default | Legacy access |
| --- | --- | --- |
| `Order.total_fee` | computed narrowly as the canonical executed-fee component (`calculate_total_fee(executed_fee)`); the legacy wire field `executedFeeAmount` is never folded into the canonical sum | `Order.executed_fee_amount: Amount` surfaces the legacy wire value as a typed read-only sibling so consumers that need the legacy summation compute `executed_fee + executed_fee_amount` explicitly at the call site |

## Provenance Anchors

- Global source contract: `parity/source-lock.yaml`
- Surface ownership and upstream paths: [Parity Sources](parity-sources.md)
- First-release shipped and deferred surfaces:
  [Parity Scope](parity-scope.md#first-release-scope)
- Scope-to-proof mapping: [Validation Scope](validation-scope.md)
- Packaging and release verification: [Release Checklist](release-checklist.md)

## Publish Order

The published crate-family dry-run order follows the release checklist:

1. `cow-sdk-core`
2. `cow-sdk-contracts`
3. `cow-sdk-app-data`
4. `cow-sdk-orderbook`
5. `cow-sdk-signing`
6. `cow-sdk-subgraph`
7. `cow-sdk-transport-wasm`
8. `cow-sdk-trading`
9. `cow-sdk-browser-wallet`
10. `cow-sdk-alloy-provider`
11. `cow-sdk-alloy-signer`
12. `cow-sdk-alloy`
13. `cow-sdk`
