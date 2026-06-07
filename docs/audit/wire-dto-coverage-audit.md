# Wire DTO Coverage Audit

Status: Current
Last reviewed: 2026-06-07
Owning surface: cow-sdk-orderbook DTO coverage
Refresh trigger: changes to `parity/openapi/services-orderbook.yml`, changes to `parity/openapi/coverage.yaml`, source-lock refreshes for the services OpenAPI, or public field changes on covered orderbook request or response DTOs
Related docs:
- [ADR 0031](../adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md)
- [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md)
- [ADR 0058](../adr/0058-typed-quote-request-response-surface.md)

## Scope

This audit covers:

- source-lock-pinned OpenAPI vendoring for the orderbook service schema
- inventory-backed Rust DTO coverage for `Order`, `OrderQuoteResponse`, `OrderParameters`, `Trade`, `StoredOrderQuote`, `OnchainOrderData`, `TotalSurplus`, and `SolverExecution`
- reviewed request payload coverage for `OrderCreation`, `OrderQuoteRequest`,
  `AppDataObject`, and `OrderCancellations`
- recorded fixture coverage and field-level round-trip tests for the eight covered DTOs
- manifest-level required-field lists that must match each inventory's
  expanded OpenAPI `required` set
- forward-compatible response deserialization without `serde(deny_unknown_fields)`
- inbound rejection of non-zero `OrderCreation.feeAmount` before a parsed
  request can be used as a valid order submission DTO

It does not cover app-data schema content, contract ABI DTOs, or live orderbook endpoint behavior.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| OpenAPI provenance | `parity/openapi/services-orderbook.yml` is vendored from the services commit pinned in `parity/source-lock.yaml` and carries a source-stamp header. | Conforms |
| Inventory coverage | Every DTO listed in `parity/openapi/coverage.yaml` has a committed per-schema inventory under `parity/openapi/`. | Conforms |
| Required-field drift | Every manifest entry records `required_fields`, and validation fails if the list diverges from the inventory's `expanded_required` set. | Conforms |
| Rust DTO shape | The covered Rust response DTOs contain every inventory field with OpenAPI optionality preserved at the Rust boundary. | Conforms |
| Fixture coverage | Each covered DTO has a recorded fixture under `parity/fixtures/orderbook/` that exercises every modeled top-level inventory field. | Conforms |
| Forward compatibility | Covered response DTOs do not use `serde(deny_unknown_fields)`, so additive upstream fields do not break deserialization. | Conforms |
| Request DTO coverage | Every constructed orderbook request payload has a reviewed fixture under `parity/fixtures/orderbook-requests/` with source references to the pinned services revision. | Conforms |
| OrderCreation fee boundary | `OrderCreation` serializes `feeAmount` as `"0"` and rejects inbound non-zero `feeAmount` during deserialization. | Conforms |
| OrderCreation app-data routing | `OrderCreation` serialises the `(app_data, app_data_hash)` pair onto the three services `OrderCreationAppData` untagged-enum variants (`Both`, `Hash`, `Full`); the hash-only case keys the hash hex string under the `appData` key per the services `Hash` variant. | Conforms |
| Identity wire-form preservation | Cow newtypes `Address`, `Hash32`, `AppDataHash`, `HexData`, and `OrderUid` emit the lowercase `0x`-prefixed hex wire form through their cow-owned or alloy-forwarded `Display`/`Serialize`/`Deserialize` impls; `Amount` and `SignedAmount` emit strict-decimal-only wire form through their cow-owned `Serialize`/`Deserialize` impls per [ADR 0052](../adr/0052-alloy-primitives-canonical-primitive-layer.md). | Conforms |

## Current Contract

### OpenAPI Provenance

The vendored orderbook OpenAPI document is committed at
`parity/openapi/services-orderbook.yml`. The file records the upstream services
commit and source path in its header. `parity/openapi/coverage.yaml` is the
public manifest for the eight covered DTOs, and each manifest entry points to the
inventory and fixture used to validate that DTO.
The manifest also carries the required-field set for each DTO. The
`openapi-coverage --validate` command compares that list against the committed
inventory's `expanded_required` values so required-field drift is visible even
when optional additive fields remain forward-compatible.

### Order Field Scope

`cow_sdk_orderbook::Order` is the single order-shaped response DTO. It models
`OrderCreation`, `OrderMetaData`, and optional inline interactions. It does not
carry auction-only fields (`protocolFees`, `preInteractions`,
`postInteractions`, `created`, `executed`, or an auction-side `quote`): those
belong to the auction schema, which has no public producer and is not mirrored
(see [ADR 0031](../adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md)).

### Solver-Competition v2 Coverage

`cow_sdk_orderbook::SolverCompetitionResponse` (the `/api/v2/solver_competition/*`
payload) is deliberately not enrolled in the inventory manifest above. The
vendored v2 schema omits a `required:` block, so `openapi-coverage --validate`
would force every field — including the always-present `auctionId`, block
deadlines, and `auction` — to `Option<T>`. The upstream producer (the `Response`
struct in `services` `solver_competition_v2.rs`) instead models the identity and
collection fields as required and only `txHash` / `referenceScore` as optional,
and the typed `SolverCompetitionResponse` mirrors that producer contract exactly.
Coverage is provided by a producer-pinned round-trip fixture
(`parity/fixtures/orderbook/solver_competition_response.json`) built from the
producer's own canonical serialization vector and exercised by
`crates/orderbook/tests/transform_contract.rs::solver_competition_response_fixture_roundtrips_upstream_producer_vector`,
rather than by the OpenAPI-optionality manifest. This divergence is recorded in
[ADR 0031](../adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md)
and [Parity Scope](../parity.md).

### Field Provenance

| Rust DTO | OpenAPI schema | Inventory | Fixture | Covered inventory fields |
| --- | --- | --- | --- | --- |
| `Order` | `components.schemas.Order` | `parity/openapi/order-inventory.yaml` | `parity/fixtures/orderbook/order_with_full_metadata.json` | `appData`, `appDataHash`, `availableBalance`, `buyAmount`, `buyToken`, `buyTokenBalance`, `class`, `creationDate`, `ethflowData`, `executedBuyAmount`, `executedFee`, `executedFeeAmount`, `executedFeeToken`, `executedSellAmount`, `executedSellAmountBeforeFees`, `feeAmount`, `from`, `fullAppData`, `fullBalanceCheck`, `interactions`, `invalidated`, `isLiquidityOrder`, `kind`, `onchainOrderData`, `onchainUser`, `owner`, `partiallyFillable`, `quote`, `quoteId`, `receiver`, `sellAmount`, `sellToken`, `sellTokenBalance`, `settlementContract`, `signature`, `signingScheme`, `status`, `uid`, `validTo` |
| `OrderQuoteResponse` | `components.schemas.OrderQuoteResponse` | `parity/openapi/order-quote-response-inventory.yaml` | `parity/fixtures/orderbook/order_quote_response.json` | `expiration`, `from`, `id`, `protocolFeeBps`, `quote`, `verified` |
| `QuoteData` | `components.schemas.OrderParameters` | `parity/openapi/order-parameters-inventory.yaml` | `parity/fixtures/orderbook/order_parameters.json` | `appData`, `appDataHash`, `buyAmount`, `buyToken`, `buyTokenBalance`, `feeAmount`, `gasAmount`, `gasPrice`, `kind`, `partiallyFillable`, `receiver`, `sellAmount`, `sellToken`, `sellTokenBalance`, `sellTokenPrice`, `signingScheme`, `validTo` |
| `Trade` | `components.schemas.Trade` | `parity/openapi/trade-inventory.yaml` | `parity/fixtures/orderbook/trade.json` | `blockNumber`, `buyAmount`, `buyToken`, `executedProtocolFees`, `logIndex`, `orderUid`, `owner`, `sellAmount`, `sellAmountBeforeFees`, `sellToken`, `txHash` |
| `StoredOrderQuote` | `components.schemas.StoredOrderQuote` | `parity/openapi/stored-order-quote-inventory.yaml` | `parity/fixtures/orderbook/stored_order_quote.json` | `buyAmount`, `feeAmount`, `gasAmount`, `gasPrice`, `metadata`, `sellAmount`, `sellTokenPrice`, `solver`, `verified` |
| `OnchainOrderData` | `components.schemas.OnchainOrderData` | `parity/openapi/onchain-order-data-inventory.yaml` | `parity/fixtures/orderbook/onchain_order_data.json` | `placementError`, `sender` |
| `TotalSurplus` | `components.schemas.TotalSurplus` | `parity/openapi/total-surplus-inventory.yaml` | `parity/fixtures/orderbook/total_surplus.json` | `totalSurplus` |
| `SolverExecution` | `components.schemas.CompetitionOrderStatus.value.items` | `parity/openapi/solver-execution-inventory.yaml` | `parity/fixtures/orderbook/solver_execution.json` | `executedBuyAmount`, `executedSellAmount`, `solver` |

### Request DTOs

| DTO type | Source file and Rust type | Audit verdict | Fixture path | Last reviewed |
| --- | --- | --- | --- | --- |
| `OrderCreation` | `crates/orderbook/src/types/order.rs::cow_sdk_orderbook::OrderCreation` | Conforms | `parity/fixtures/orderbook-requests/order_creation.json` | 2026-05-21 |
| `OrderQuoteRequest` | `crates/orderbook/src/types/quote.rs::cow_sdk_orderbook::OrderQuoteRequest` | Conforms | `parity/fixtures/orderbook-requests/order_quote_request.json` | 2026-05-12 |
| `AppDataObject` PUT payload | `crates/orderbook/src/api.rs::cow_sdk_orderbook::AppDataObject` | Conforms | `parity/fixtures/orderbook-requests/app_data_put.json` | 2026-05-04 |
| `OrderCancellations` | `crates/orderbook/src/types/order.rs::cow_sdk_orderbook::OrderCancellations` | Conforms | `parity/fixtures/orderbook-requests/order_cancellations.json` | 2026-05-12 |

Request payload semantics reviewed against the services revision pinned in
`parity/source-lock.yaml`:

| DTO | Mandatory fields | Optional fields and defaults | Mutual-exclusion or dependency guard |
| --- | --- | --- | --- |
| `OrderCreation` | `sellToken`, `buyToken`, `sellAmount`, `buyAmount`, `validTo`, `appData`, `feeAmount`, `kind`, `partiallyFillable`, `signingScheme`, and `signature` are required by the vendored OpenAPI. The SDK also requires `from` on the typed constructor so owner intent is explicit. | `receiver`, `appDataHash`, `quoteId`, and `fullBalanceCheck` are optional. `sellTokenBalance` and `buyTokenBalance` default to `erc20`. The SDK emits `feeAmount` as `"0"` and deserializes omitted `feeAmount` as zero for compatibility with existing typed payloads. The cow `Serialize` impl is hand-rolled and routes the `(app_data, app_data_hash)` pair onto the services `OrderCreationAppData` untagged-enum variants: `(Some(s), None)` -> services `Full` (`{"appData": s}`); `(None, Some(h))` -> services `Hash` (`{"appData": "0x<hash hex>"}` — the hash hex string lives under the `appData` key); `(Some(s), Some(h))` -> services `Both` (`{"appData": s, "appDataHash": "0x<hash hex>"}`); `(None, None)` omits both fields and surfaces as a services rejection so callers must attach app-data through `with_app_data` or `with_app_data_hash`. | Non-zero `feeAmount` now fails during `OrderCreation` deserialization with the stable serde error substring. Services also returns `OrderbookRejection::NonZeroFee` if a non-zero order-level fee reaches the backend. App-data hash mismatches remain a services-side `OrderbookRejection::AppDataHashMismatch`. |
| `OrderQuoteRequest` | `sellToken`, `buyToken`, and `from` are required by the vendored OpenAPI. Exactly one side amount is required through the flattened side. | `receiver`, `validFor`, `validTo`, `appData`, `appDataHash`, `sellTokenBalance`, `buyTokenBalance`, `signingScheme`, `onchainOrder`, `verificationGasLimit`, and `timeout` are optional. The SDK constructor sets the public `appData` hash to the zero hash, `sellTokenBalance` and `buyTokenBalance` to `erc20`, `signingScheme` to `eip712`, `onchainOrder` to `false`, and `priceQuality` to `optimal`. The `optimal` default matches the estimate the orderbook expects order creation to build from, so the quote-then-sign path produces a submittable order by default (see [ADR 0058](../adr/0058-typed-quote-request-response-surface.md)). | The quote `oneOf`s are typed so an invalid request is unrepresentable: `OrderQuoteSide` carries exactly one side (with `SellAmount` distinguishing the before-fee and after-fee sell amount), `QuoteValidity` carries `validTo` or `validFor`, and `QuoteSigningScheme` keeps `verificationGasLimit` on EIP-1271 only and makes an ECDSA on-chain order unrepresentable. Malformed or conflicting wire input is rejected during deserialization: the `SellAmount` deserializer requires exactly one sell-amount key and the `QuoteSigningScheme` `try_from` guard rejects an ECDSA on-chain scheme or a stray `verificationGasLimit`. `OrderQuoteRequest::validate` is retained as an infallible pre-dispatch hook because these invariants now hold by construction. |
| `AppDataObject` PUT payload | `fullAppData` is the single request-body field for `PUT /api/v1/app_data/{hash}`. | The path hash is required by the route. No request-body defaults are applied. | Services validates the full app-data document and returns typed `AppDataInvalid`, `AppDataHashMismatch`, or `AppDataMismatch` rejections. The SDK constructs the body with the single `fullAppData` field. |
| `OrderCancellations` | `signature` and `signingScheme` are required by the vendored OpenAPI. The services model signs flattened `orderUids` and the SDK always constructs that list explicitly. | `signingScheme` defaults to `eip712` in the SDK constructor. `orderUids` is supplied by callers and the upstream description caps the list at 128 UIDs. | Signature verification remains services-side and maps into orderbook cancellation rejection variants such as malformed or invalid signature responses. There are no mutually exclusive fields in the payload. |

### Forward Compatibility

The covered response DTOs are open to additive upstream fields. Unknown fields
are ignored during deserialization, while known fields remain modeled in the
public typed surface and covered by fixtures.

### Validator Self-Test Enforcement

The OpenAPI coverage validator has negative self-tests for structured field
mismatches and required-field drift:
`scripts/parity-maintainer/tests/openapi_coverage.rs::openapi_coverage_validate_reports_structured_field_mismatches`
and
`scripts/parity-maintainer/tests/openapi_coverage.rs::openapi_coverage_validate_reports_required_field_drift`.
The shared quality gate runs the full `parity-maintainer` test suite through
the `parity-maintainer` job, so validator regressions fail CI instead of
remaining only locally reproducible.

## Evidence

Primary implementation points:

- `crates/orderbook/src/types/order.rs`
- `crates/orderbook/src/types/quote.rs`
- `crates/orderbook/src/types/prices.rs`
- `crates/orderbook/src/types/lists.rs`
- `crates/orderbook/src/types/auction.rs`
- `crates/orderbook/src/api.rs`
- `scripts/parity-maintainer/src/openapi_coverage.rs`
- `.github/workflows/_quality-gate.yml`
- `parity/openapi/coverage.yaml`
- `parity/openapi/services-orderbook.yml`
- `parity/openapi/order-inventory.yaml`
- `parity/openapi/order-quote-response-inventory.yaml`
- `parity/openapi/trade-inventory.yaml`
- `parity/openapi/stored-order-quote-inventory.yaml`
- `parity/openapi/onchain-order-data-inventory.yaml`
- `parity/openapi/total-surplus-inventory.yaml`
- `parity/openapi/solver-execution-inventory.yaml`
- `parity/fixtures/orderbook-requests/order_creation.json`
- `parity/fixtures/orderbook-requests/order_quote_request.json`
- `parity/fixtures/orderbook-requests/app_data_put.json`
- `parity/fixtures/orderbook-requests/order_cancellations.json`

Primary regression coverage:

- `crates/orderbook/tests/types_contract.rs::order_creation_serialize_routes_app_data_combinations_to_services_variants`
- `crates/orderbook/tests/types_contract.rs::order_creation_from_quote_serialize_emits_services_hash_variant`
- `crates/orderbook/tests/order_creation_fee_deserialize.rs::order_creation_deserialize_accepts_zero_or_omitted_fee_amount`
- `crates/orderbook/tests/order_creation_fee_deserialize.rs::order_creation_deserialize_rejects_non_zero_fee_amount`
- `crates/orderbook/tests/order_creation_fee_deserialize.rs::order_creation_deserialize_keeps_malformed_fee_amount_parser_error`
- `crates/orderbook/tests/order_creation_fee_deserialize.rs::quote_data_deserialize_keeps_non_zero_network_cost_fee_amount`
- `crates/orderbook/tests/order_creation_fee_deserialize.rs::order_creation_deserialize_fee_amount_boundary_is_zero_only`
- `crates/orderbook/tests/transform_contract.rs::order_fixture_deserializes_nested_typed_accessors`
- `crates/orderbook/tests/transform_contract.rs::order_quote_response_fixture_deserializes_typed_accessors`
- `crates/orderbook/tests/transform_contract.rs::trade_fixture_deserializes_typed_accessors`
- `crates/orderbook/tests/transform_contract.rs::stored_order_quote_fixture_deserializes_typed_accessors`
- `crates/orderbook/tests/transform_contract.rs::onchain_order_data_fixture_deserializes_typed_accessors`
- `crates/orderbook/tests/wire_contract.rs::promoted_amount_dtos_roundtrip_byte_identical`
- `crates/orderbook/tests/wire_contract.rs::openapi_response_dtos_roundtrip_required_fixture_fields`
- `scripts/parity-maintainer/tests/openapi_coverage.rs::openapi_coverage_validate_reports_structured_field_mismatches`
- `scripts/parity-maintainer/tests/openapi_coverage.rs::openapi_coverage_validate_reports_required_field_drift`

Validation surface:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- openapi-coverage --validate
cargo parity-validate --source-lock parity/source-lock.yaml
cargo test --manifest-path scripts/parity-maintainer/Cargo.toml
cargo test -p cow-sdk-orderbook --test order_creation_fee_deserialize
cargo test -p cow-sdk-orderbook --test wire_contract
cargo test -p cow-sdk-orderbook --test transform_contract
cargo run --manifest-path scripts/policy-maintainer/Cargo.toml -- check-deny-unknown-fields
```
