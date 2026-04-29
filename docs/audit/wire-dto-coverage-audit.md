# Wire DTO Coverage Audit

Status: Current
Last reviewed: 2026-04-29
Owning surface: cow-sdk-orderbook DTO coverage
Refresh trigger: changes to `parity/openapi/services-orderbook.yml`, changes to `parity/openapi/coverage.yaml`, source-lock refreshes for the services OpenAPI, or public field changes on covered orderbook response DTOs

## Scope

This audit covers:

- source-lock-pinned OpenAPI vendoring for the orderbook service schema
- inventory-backed Rust DTO coverage for `Order`, `AuctionOrder`, `OrderQuoteResponse`, `Trade`, `StoredOrderQuote`, and `OnchainOrderData`
- recorded fixture coverage and field-level round-trip tests for the six covered DTOs
- forward-compatible response deserialization without `serde(deny_unknown_fields)`

It does not cover request builders, app-data schemas, contract ABI DTOs, or live orderbook endpoint behavior.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| OpenAPI provenance | `parity/openapi/services-orderbook.yml` is vendored from the services commit pinned in `parity/source-lock.yaml` and carries a source-stamp header. | Conforms |
| Inventory coverage | Every DTO listed in `parity/openapi/coverage.yaml` has a committed per-schema inventory under `parity/openapi/`. | Conforms |
| Rust DTO shape | The covered Rust response DTOs contain every inventory field with OpenAPI optionality preserved at the Rust boundary. | Conforms |
| Fixture coverage | Each covered DTO has a recorded fixture under `parity/fixtures/orderbook/` that exercises every modeled top-level inventory field. | Conforms |
| Forward compatibility | Covered response DTOs do not use `serde(deny_unknown_fields)`, so additive upstream fields do not break deserialization. | Conforms |

## Current Contract

### OpenAPI Provenance

The vendored orderbook OpenAPI document is committed at
`parity/openapi/services-orderbook.yml`. The file records the upstream services
commit and source path in its header. `parity/openapi/coverage.yaml` is the
public manifest for the six covered DTOs, and each manifest entry points to the
inventory and fixture used to validate that DTO.

### DTO Separation

`cow_sdk_orderbook::Order` and `cow_sdk_orderbook::AuctionOrder` cover separate
OpenAPI schemas. `Order` models `OrderCreation`, `OrderMetaData`, and optional
inline interactions. `AuctionOrder` models auction-only fields such as
`protocolFees`, `preInteractions`, `postInteractions`, `created`, `executed`,
and the auction-side `quote`.

### Field Provenance

| Rust DTO | OpenAPI schema | Inventory | Fixture | Covered inventory fields |
| --- | --- | --- | --- | --- |
| `Order` | `components.schemas.Order` | `parity/openapi/order-inventory.yaml` | `parity/fixtures/orderbook/order_with_full_metadata.json` | `appData`, `appDataHash`, `availableBalance`, `buyAmount`, `buyToken`, `buyTokenBalance`, `class`, `creationDate`, `ethflowData`, `executedBuyAmount`, `executedFee`, `executedFeeAmount`, `executedFeeToken`, `executedSellAmount`, `executedSellAmountBeforeFees`, `feeAmount`, `from`, `fullAppData`, `fullBalanceCheck`, `interactions`, `invalidated`, `isLiquidityOrder`, `kind`, `onchainOrderData`, `onchainUser`, `owner`, `partiallyFillable`, `quote`, `quoteId`, `receiver`, `sellAmount`, `sellToken`, `sellTokenBalance`, `settlementContract`, `signature`, `signingScheme`, `status`, `uid`, `validTo` |
| `AuctionOrder` | `components.schemas.AuctionOrder` | `parity/openapi/auction-order-inventory.yaml` | `parity/fixtures/orderbook/auction_order_with_protocol_fees.json` | `appData`, `buyAmount`, `buyToken`, `buyTokenBalance`, `class`, `created`, `executed`, `kind`, `owner`, `partiallyFillable`, `postInteractions`, `preInteractions`, `protocolFees`, `quote`, `receiver`, `sellAmount`, `sellToken`, `sellTokenBalance`, `signature`, `uid`, `validTo` |
| `OrderQuoteResponse` | `components.schemas.OrderQuoteResponse` | `parity/openapi/order-quote-response-inventory.yaml` | `parity/fixtures/orderbook/order_quote_response.json` | `expiration`, `from`, `id`, `protocolFeeBps`, `quote`, `verified` |
| `Trade` | `components.schemas.Trade` | `parity/openapi/trade-inventory.yaml` | `parity/fixtures/orderbook/trade.json` | `blockNumber`, `buyAmount`, `buyToken`, `executedProtocolFees`, `logIndex`, `orderUid`, `owner`, `sellAmount`, `sellAmountBeforeFees`, `sellToken`, `txHash` |
| `StoredOrderQuote` | `components.schemas.StoredOrderQuote` | `parity/openapi/stored-order-quote-inventory.yaml` | `parity/fixtures/orderbook/stored_order_quote.json` | `buyAmount`, `feeAmount`, `gasAmount`, `gasPrice`, `metadata`, `sellAmount`, `sellTokenPrice`, `solver`, `verified` |
| `OnchainOrderData` | `components.schemas.OnchainOrderData` | `parity/openapi/onchain-order-data-inventory.yaml` | `parity/fixtures/orderbook/onchain_order_data.json` | `placementError`, `sender` |

### Forward Compatibility

The covered response DTOs are open to additive upstream fields. Unknown fields
are ignored during deserialization, while known fields remain modeled in the
public typed surface and covered by fixtures.

## Evidence

Primary implementation points:

- `crates/orderbook/src/types.rs`
- `scripts/parity-maintainer/src/openapi_coverage.rs`
- `parity/openapi/coverage.yaml`
- `parity/openapi/services-orderbook.yml`
- `parity/openapi/order-inventory.yaml`
- `parity/openapi/auction-order-inventory.yaml`
- `parity/openapi/order-quote-response-inventory.yaml`
- `parity/openapi/trade-inventory.yaml`
- `parity/openapi/stored-order-quote-inventory.yaml`
- `parity/openapi/onchain-order-data-inventory.yaml`

Primary regression coverage:

- `crates/orderbook/tests/transform_contract.rs::order_fixture_matches_openapi_inventory`
- `crates/orderbook/tests/transform_contract.rs::auction_order_fixture_matches_openapi_inventory`
- `crates/orderbook/tests/transform_contract.rs::order_quote_response_fixture_matches_openapi_inventory`
- `crates/orderbook/tests/transform_contract.rs::trade_fixture_matches_openapi_inventory`
- `crates/orderbook/tests/transform_contract.rs::stored_order_quote_fixture_matches_openapi_inventory`
- `crates/orderbook/tests/transform_contract.rs::onchain_order_data_fixture_matches_openapi_inventory`

Validation surface:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- openapi-coverage --validate
cargo test -p cow-sdk-orderbook --test transform_contract
cargo run --manifest-path scripts/policy-maintainer/Cargo.toml -- check-deny-unknown-fields
```
