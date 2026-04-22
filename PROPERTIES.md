# Properties Registry

This registry is the canonical public index of invariants and state contracts
for `cow-rs`.

Use it with:

- [Verification Guide](docs/verification-guide.md) for how the evidence is
  interpreted
- [Verification Matrix](docs/verification-matrix.md) for the crate and
  workflow lanes that exercise each surface

Executable coverage stays with the crate or browser surface that owns the
behavior. This registry records what must remain true, who owns it, and where
the current evidence lives.

`Covered` uses these values:

- `Yes`: dedicated executable coverage exists
- `Partial`: deterministic coverage exists, but not through a dedicated
  property or state-machine suite
- `No`: the property is registered, but no executable coverage is attached yet

`Last reviewed` records the most recent date the row was confirmed against the
shipped code. The registry follows a 90-day re-review rhythm that mirrors the
dependency-exception policy in `.github/config/deny.toml`: every row is
re-confirmed at least once per 90-day window, and the date here is refreshed
in the same change that touches the owning surface.

## Methodology

Rows labeled `Property` use deterministic invariant sweeps, not randomized
framework-based property testing. The standard `property_contract.rs` suites
drive a hand-rolled `CaseRng` through a reproducible 128-case boundary set,
and some named narrow-search profiles extend the same deterministic approach
with larger curated sweeps.

This proves exhaustive coverage of the curated boundary cases attached to each
property and keeps failures reproducible from committed seeds and fixtures. It
does not prove random-exploration discovery of unexpected inputs, shrinking to
minimal counterexamples, or statistical guarantees over uncurated input space.
`cow-rs` therefore uses `Property` in the broad invariant-testing sense rather
than as a claim of `proptest`- or `quickcheck`-style randomized shrinking
coverage.

| Id | Crate | Property | Type | Covered | Evidence | Last reviewed |
| --- | --- | --- | --- | --- | --- | --- |
| `PROP-CORE-001` | `cow-sdk-core` | Unsupported chain and environment resolution stays explicit and typed instead of falling back silently. | Contract | Yes | `crates/core/tests/config_contract.rs`, `crates/core/tests/types_contract.rs` | 2026-04-17 |
| `PROP-CORE-002` | `cow-sdk-core` | Runtime traits preserve typed transaction, hash, and provider boundaries across sync and async implementations. | Contract | Yes | `crates/core/tests/traits_contract.rs`, `crates/core/tests/types_contract.rs` | 2026-04-17 |
| `PROP-CORE-003` | `cow-sdk-core` | Validated hex wrappers (`Address`, `Hash32`, `AppDataHex`, `OrderUid`, `HexData`) preserve input case and roundtrip through their string forms while failing closed on length, prefix, and character errors. | Property | Yes | `crates/core/tests/property_contract.rs`, `crates/core/tests/types_contract.rs` | 2026-04-17 |
| `PROP-CORE-004` | `cow-sdk-core` | `Amount` canonicalizes decimal and hexadecimal inputs into equal values, rejects negative and out-of-range inputs, and normalizes zero forms deterministically. | Property | Yes | `crates/core/tests/property_contract.rs` | 2026-04-17 |
| `PROP-CORE-005` | `cow-sdk-core` | `Address` case normalization keeps `addresses_equal` case-insensitive while `token_id` remains deterministic and sensitive to both chain id and address changes. | Property | Yes | `crates/core/tests/property_contract.rs` | 2026-04-17 |
| `PROP-CORE-006` | `cow-sdk-core` | `SupportedChainId` roundtrips through its numeric `ChainId` form for every supported variant and fails closed on unsupported chain ids. | Property | Yes | `crates/core/tests/property_contract.rs`, `crates/core/tests/config_contract.rs` | 2026-04-17 |
| `PROP-CON-001` | `cow-sdk-contracts` | Order hashing remains deterministic across semantically equivalent normalized inputs. | Property | Yes | `crates/contracts/tests/property_contract.rs`, `crates/contracts/tests/order_contract.rs` | 2026-04-17 |
| `PROP-CON-002` | `cow-sdk-contracts` | ABI helper builders, compact flag codecs, and signature payload codecs preserve explicit boundary semantics for settlement, swap, vault, reader, interaction, signature, and deployment payloads. | Property | Yes | `crates/contracts/tests/property_contract.rs` (includes `abi_layout_narrow_search_profile_preserves_eip1271_payload_boundaries`), `crates/contracts/tests/settlement_contract.rs`, `crates/contracts/tests/swap_contract.rs`, `crates/contracts/tests/signature_contract.rs`, `crates/contracts/tests/vault_contract.rs`, `crates/contracts/tests/reader_contract.rs`, `crates/contracts/tests/interaction_contract.rs`, `crates/contracts/src/primitives.rs (tests)`, `crates/contracts/src/order.rs (tests)`, `crates/contracts/src/settlement.rs (tests)`, `crates/contracts/src/deploy.rs (tests)` | 2026-04-17 |
| `PROP-SIG-001` | `cow-sdk-signing` | Domain separation changes only when the typed-data domain changes. | Property | Yes | `crates/signing/tests/property_contract.rs`, `crates/signing/tests/domain_contract.rs` | 2026-04-17 |
| `PROP-SIG-002` | `cow-sdk-signing` | Order and cancellation typed-data payloads, generated ids, and EIP-1271 helper payloads stay deterministic for equivalent inputs and explicit signing-scheme boundaries. | Property | Yes | `crates/signing/tests/property_contract.rs`, `crates/signing/tests/order_signing_contract.rs`, `crates/signing/tests/cancellation_contract.rs`, `crates/signing/tests/eip1271_contract.rs` | 2026-04-17 |
| `PROP-APP-001` | `cow-sdk-app-data` | CID conversion round-trips between digest and CID forms without silent mutation. | Property | Yes | `crates/app-data/tests/property_contract.rs`, `crates/app-data/tests/cid_contract.rs`, `crates/app-data/src/cid.rs (tests)` | 2026-04-17 |
| `PROP-APP-002` | `cow-sdk-app-data` | Invalid app-data, schema, fetch, info, and pinning inputs fail closed. | Property | Yes | `crates/app-data/tests/property_contract.rs` (includes `schema_parsing_narrow_search_profile_roundtrips_valid_triplets_and_rejects_invalid_forms`), `crates/app-data/tests/schema_contract.rs`, `crates/app-data/tests/fetch_contract.rs`, `crates/app-data/tests/pinning_contract.rs`, `crates/app-data/src/info.rs (tests)`, `crates/app-data/src/types.rs (tests)` | 2026-04-17 |
| `PROP-APP-003` | `cow-sdk-app-data` | Deterministic document sources render canonical JSON and stable latest-path digests for equivalent document shapes. | Property | Yes | `crates/app-data/tests/property_contract.rs` (includes `canonicalization_narrow_search_profile_preserves_equivalent_nested_documents`), `crates/app-data/tests/app_data_info_contract.rs`, `crates/app-data/src/info.rs (tests)` | 2026-04-17 |
| `PROP-ORD-001` | `cow-sdk-orderbook` | Request builders preserve explicit field shape, pagination defaults, and `appData` transport without silently coercing unsupported inputs. | Property | Yes | `crates/orderbook/tests/invariant_contract.rs`, `crates/orderbook/tests/request_contract.rs`, `crates/orderbook/tests/api_contract.rs`, `crates/orderbook/tests/types_contract.rs` | 2026-04-17 |
| `PROP-ORD-002` | `cow-sdk-orderbook` | Response decoding, retry termination, and transform layers fail closed on malformed upstream payloads. | Property | Yes | `crates/orderbook/tests/invariant_contract.rs`, `crates/orderbook/tests/transform_contract.rs`, `crates/orderbook/tests/types_contract.rs`, `crates/orderbook/tests/request_contract.rs` | 2026-04-17 |
| `PROP-TRD-001` | `cow-sdk-trading` | Quote and post context precedence remains explicit and deterministic across builder defaults, quote-request overrides, derived quote-to-order parameters, and collision-driven order-id retries. | Property | Yes | `crates/trading/tests/invariant_contract.rs`, `crates/trading/tests/sdk_contract.rs`, `crates/trading/tests/quote_contract.rs`, `crates/trading/tests/post_contract.rs`, `crates/trading/tests/order_contract.rs` | 2026-04-17 |
| `PROP-TRD-002` | `cow-sdk-trading` | Slippage outputs, protocol-fee sanitization, and partner-fee extraction remain explicit, monotonic, and clamped across valid inputs. | Property | Yes | `crates/trading/tests/invariant_contract.rs`, `crates/trading/tests/slippage_contract.rs` | 2026-04-17 |
| `PROP-TRD-003` | `cow-sdk-trading` | On-chain helper builders preserve unsigned `uint256` calldata boundary semantics. | Property | Yes | `crates/trading/tests/invariant_contract.rs`, `crates/trading/tests/onchain_contract.rs` | 2026-04-17 |
| `PROP-SBG-001` | `cow-sdk-subgraph` | Query requests preserve explicit operation-name handling plus nested variable object and array shape. | Property | Yes | `crates/subgraph/tests/invariant_contract.rs` (includes `raw_request_narrow_search_profile_preserves_explicit_documents_and_nested_variables`), `crates/subgraph/tests/query_contract.rs`, `crates/subgraph/tests/api_contract.rs` | 2026-04-17 |
| `PROP-SBG-002` | `cow-sdk-subgraph` | Typed response decoding accepts equivalent string-or-number scalar forms and fails closed on malformed or missing data. | Property | Yes | `crates/subgraph/tests/invariant_contract.rs` (includes `scalar_decode_narrow_search_profile_covers_boundary_numeric_forms`), `crates/subgraph/tests/types_contract.rs`, `crates/subgraph/tests/api_contract.rs` | 2026-04-17 |
| `PROP-BWL-001` | `cow-sdk-browser-wallet` | Ambiguous discovery never silently auto-selects a provider. | State machine | Yes | `crates/browser-wallet/src/wallet.rs`, `crates/browser-wallet/tests/wallet_contract.rs` | 2026-04-17 |
| `PROP-BWL-002` | `cow-sdk-browser-wallet` | Session, chain, typed-data, and typed RPC classification boundaries stay explicit under deterministic transports and committed browser automation. | State machine | Yes | `crates/browser-wallet/tests/state_machine_contract.rs`, `crates/browser-wallet/tests/provider_contract.rs`, `crates/browser-wallet/tests/wallet_contract.rs`, `crates/browser-wallet/src/provider.rs (tests)`, `crates/browser-wallet/src/error.rs (tests)`, `e2e/browser-wallet/tests/browser-wallet-console.spec.ts` | 2026-04-17 |
| `PROP-SDK-001` | `cow-sdk` | The facade remains curated and feature-gated, without widening the default surface beyond leaf-crate ownership. | Public API | Yes | `crates/sdk/tests/public_api.rs` | 2026-04-17 |
| `PROP-CORE-007` | `cow-sdk-core` | `HttpTransport` is dyn-compatible, typed failures flow through `TransportError` with a `TransportErrorClass` partition, and every default adapter strips the URL before wrapping so credential-bearing query strings never surface through `Debug` or `Display`. Governed by [ADR 0013](docs/adr/0013-http-transport-injection-and-typestate-builders.md). | Contract | Yes | `crates/core/tests/transport_contract.rs`, `crates/transport-wasm/tests/parity_contract.rs` | 2026-04-21 |
| `PROP-CON-003` | `cow-sdk-contracts` | Every ABI binding is generated through `alloy::sol!` from committed upstream Solidity excerpts and preserves byte-identity on the encoded call-data and hashed payloads of `GPv2Settlement`, `GPv2VaultRelayer`, `CoWSwapEthFlow`, the EIP-1967 proxy, and `IERC20` / `IERC20Permit`. Governed by [ADR 0012](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md). | Contract | Yes | `crates/contracts/tests/parity_contract.rs` | 2026-04-21 |
| `PROP-CON-004` | `cow-sdk-contracts` | `Registry` resolves every deployed contract address from the typed `(ContractId, SupportedChainId, CowEnv)` triple; the embedded TOML manifest is validated at compile time and the runtime parser surfaces every failure class as a typed `RegistryError`. Governed by [ADR 0012](docs/adr/0012-alloy-sol-bindings-and-registry-authority.md). | Contract | Yes | `crates/contracts/tests/registry.rs`, `crates/contracts/tests/build_rs_compile_fail.rs` | 2026-04-21 |
| `PROP-SIG-003` | `cow-sdk-signing` | `verify_eip1271_signature_async` takes a mandatory `Eip1271VerificationCache` parameter and caches only `Ok(())` and `Eip1271MagicValueMismatch` outcomes; every other error class re-hits the chain. Governed by [ADR 0014](docs/adr/0014-eip1271-verification-cache.md). | Contract | Yes | `crates/signing/tests/eip1271_cache_contract.rs` | 2026-04-21 |
| `PROP-ORD-003` | `cow-sdk-orderbook` | `OrderBookApi::builder()` is the sole production construction path; the typestate encodes the required inputs (chain, environment, transport) at compile time and no free-function constructor remains. Governed by [ADR 0013](docs/adr/0013-http-transport-injection-and-typestate-builders.md). | Contract | Yes | `crates/orderbook/tests/builder_contract.rs` | 2026-04-21 |
| `PROP-SBG-003` | `cow-sdk-subgraph` | `SubgraphApi::builder()` is the sole production construction path; the typestate encodes the required inputs (chain, API key, transport) at compile time and a `trybuild` UI witness asserts `.build()` without `.transport(...)` does not compile on `wasm32` targets. Governed by [ADR 0013](docs/adr/0013-http-transport-injection-and-typestate-builders.md). | Contract | Yes | `crates/subgraph/tests/builder_contract.rs`, `crates/subgraph/tests/ui/builder_wasm32_missing_transport.rs` | 2026-04-21 |
| `PROP-WS-001` | whole workspace | The published `cow-sdk` crate family (`cow-sdk`, `cow-sdk-core`, `cow-sdk-contracts`, `cow-sdk-signing`, `cow-sdk-app-data`, `cow-sdk-orderbook`, `cow-sdk-trading`, `cow-sdk-subgraph`, `cow-sdk-browser-wallet`) excludes `alloy-provider` from its transitive dependency graph. The `cargo tree --invert alloy-provider` command returns empty on every named crate and a CI workflow step enforces the invariant on every pull request. | Contract | Yes | `.github/workflows/ci.yml` (alloy-provider invariant step), `cargo tree --invert alloy-provider -p cow-sdk-core -p cow-sdk-contracts -p cow-sdk-signing -p cow-sdk-orderbook -p cow-sdk-subgraph -p cow-sdk-app-data -p cow-sdk-trading -p cow-sdk` | 2026-04-21 |
