# Properties Registry

This registry records the canonical properties and state contracts for the `cow-rs` workspace.

It is the authoritative index for invariant ownership:

- executable coverage stays with the crate or browser surface that owns the behavior
- this registry records what must remain true, where it belongs, and where current evidence lives
- the registry does not introduce a shared cross-workspace harness

`Covered` uses these values:

- `Yes`: dedicated executable coverage exists in the owning crate tests or committed browser automation
- `Partial`: deterministic coverage exists, but the property is not yet exercised by a dedicated property or state-machine suite
- `No`: the property is registered, but no executable coverage is attached yet

| Id | Crate | Property | Type | Covered | Evidence |
| --- | --- | --- | --- | --- | --- |
| `PROP-CORE-001` | `cow-sdk-core` | Unsupported chain and environment resolution stays explicit and typed instead of falling back silently. | Contract | Yes | `crates/core/tests/config_contract.rs`, `crates/core/tests/types_contract.rs` |
| `PROP-CORE-002` | `cow-sdk-core` | Runtime traits preserve typed transaction, hash, and provider boundaries across sync and async implementations. | Contract | Yes | `crates/core/tests/traits_contract.rs`, `crates/core/tests/types_contract.rs` |
| `PROP-CON-001` | `cow-sdk-contracts` | Order hashing remains deterministic across semantically equivalent normalized inputs. | Property | Partial | `crates/contracts/tests/order_contract.rs`, `crates/contracts/tests/signature_contract.rs` |
| `PROP-CON-002` | `cow-sdk-contracts` | ABI helper builders preserve explicit boundary semantics for settlement, swap, vault, reader, and interaction payloads. | Property | Partial | `crates/contracts/tests/settlement_contract.rs`, `crates/contracts/tests/swap_contract.rs`, `crates/contracts/tests/vault_contract.rs`, `crates/contracts/tests/reader_contract.rs`, `crates/contracts/tests/interaction_contract.rs` |
| `PROP-SIG-001` | `cow-sdk-signing` | Domain separation changes only when the typed-data domain changes. | Property | Partial | `crates/signing/tests/domain_contract.rs`, `crates/signing/tests/order_signing_contract.rs` |
| `PROP-SIG-002` | `cow-sdk-signing` | Order and cancellation payload encoding stays deterministic for equivalent inputs and explicit signing-scheme boundaries. | Property | Partial | `crates/signing/tests/order_signing_contract.rs`, `crates/signing/tests/cancellation_contract.rs`, `crates/signing/tests/eip1271_contract.rs` |
| `PROP-APP-001` | `cow-sdk-app-data` | CID conversion round-trips between digest and CID forms without silent mutation. | Property | Partial | `crates/app-data/tests/cid_contract.rs` |
| `PROP-APP-002` | `cow-sdk-app-data` | Invalid app-data, schema, fetch, and pinning inputs fail closed. | Property | Partial | `crates/app-data/tests/schema_contract.rs`, `crates/app-data/tests/fetch_contract.rs`, `crates/app-data/tests/pinning_contract.rs` |
| `PROP-ORD-001` | `cow-sdk-orderbook` | Request builders preserve explicit field shape and do not silently coerce unsupported inputs. | Property | Partial | `crates/orderbook/tests/request_contract.rs`, `crates/orderbook/tests/api_contract.rs` |
| `PROP-ORD-002` | `cow-sdk-orderbook` | Response decoding and transform layers fail closed on malformed upstream payloads. | Property | Partial | `crates/orderbook/tests/transform_contract.rs`, `crates/orderbook/tests/types_contract.rs` |
| `PROP-TRD-001` | `cow-sdk-trading` | Quote and post context precedence remains explicit and deterministic across builder defaults, overrides, and derived trade parameters. | Contract | Yes | `crates/trading/tests/sdk_contract.rs`, `crates/trading/tests/quote_contract.rs`, `crates/trading/tests/post_contract.rs` |
| `PROP-TRD-002` | `cow-sdk-trading` | Slippage outputs remain monotonic and clamped across valid inputs. | Property | Partial | `crates/trading/tests/slippage_contract.rs` |
| `PROP-TRD-003` | `cow-sdk-trading` | On-chain helper builders preserve unsigned `uint256` calldata boundary semantics. | Property | Partial | `crates/trading/tests/onchain_contract.rs` |
| `PROP-SBG-001` | `cow-sdk-subgraph` | Query requests preserve explicit operation-name handling and request shape. | Property | Partial | `crates/subgraph/tests/query_contract.rs`, `crates/subgraph/tests/api_contract.rs` |
| `PROP-SBG-002` | `cow-sdk-subgraph` | Typed response decoding fails closed on malformed or missing data. | Property | Partial | `crates/subgraph/tests/types_contract.rs`, `crates/subgraph/tests/api_contract.rs` |
| `PROP-BWL-001` | `cow-sdk-browser-wallet` | Ambiguous discovery never silently auto-selects a provider. | State machine | Yes | `crates/browser-wallet/tests/wallet_contract.rs` |
| `PROP-BWL-002` | `cow-sdk-browser-wallet` | Session, chain, and typed-data transitions stay explicit under deterministic transports and committed browser automation. | State machine | Yes | `crates/browser-wallet/tests/provider_contract.rs`, `crates/browser-wallet/tests/wallet_contract.rs`, `e2e/browser-wallet/tests/browser-wallet-console.spec.ts` |
| `PROP-SDK-001` | `cow-sdk` | The facade remains curated and feature-gated, without widening the default surface beyond leaf-crate ownership. | Public API | Yes | `crates/sdk/tests/public_api.rs` |
