# Quote Response Surface Audit

Status: Current
Last reviewed: 2026-06-07
Owning surface: cow-sdk-orderbook quote request/response DTOs and cow-sdk-trading quote projection
Refresh trigger: changes to the quote DTOs (`OrderQuoteRequest`, `OrderQuoteResponse`, `QuoteData`), the orderbook quote OpenAPI schemas, the quote-amounts projection, or the `priceQuality` default
Related docs:
- [ADR 0058](../adr/0058-typed-quote-request-response-surface.md)
- [ADR 0031](../adr/0031-wire-dto-openapi-driven-with-order-auction-order-split.md)
- [ADR 0021](../adr/0021-orderbook-total-fee-policy.md)
- [ADR 0015](../adr/0015-client-side-order-bounds-validator.md)

## Scope

This audit covers:

- quote response fidelity: `cow_sdk_orderbook::QuoteData` as the mirror of the
  orderbook `OrderParameters` schema, and its enrolment in the OpenAPI coverage
  inventory
- the `priceQuality` default and its serialization
- the read-only status of the quote network-cost fields (`feeAmount`,
  `gasAmount`, `gasPrice`, `sellTokenPrice`)
- the quote-amounts projection that derives the signable order from a `/quote`
  response, and its parity test
- the SDK trust posture: the projected order is validated by the client-side
  bounds validator and the quote response is not field-bound to the request
- the quote request payload's current validation contract

It does not cover order submission (other trading audits), app-data document
content, or composable-order quoting.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Quote response fidelity | `QuoteData` covers every required `OrderParameters` field — including `gasAmount`, `gasPrice`, `sellTokenPrice`, and `signingScheme` — and is enrolled in `parity/openapi/coverage.yaml` with its own inventory and fixture. | Conforms |
| Coverage enforcement | `openapi-coverage --validate` checks `QuoteData` against the `OrderParameters` inventory, so a dropped or mistyped quote field fails the gate instead of passing silently. | Conforms |
| Price-quality default | `OrderQuoteRequest` defaults `priceQuality` to `optimal`, the submittable estimate, and always serializes it. | Conforms |
| Read-only quote costs | The quote network-cost fields are populated only from the `/quote` response and exposed through accessors; no public builder exposes a setter. | Conforms |
| Projection parity | The quote-amounts projection matches the orderbook quote-amounts algorithm and is locked by a parity regression test. | Conforms |
| Trust posture | The SDK validates the projected order through the bounds validator before submission and does not field-bind the quote response to the request. | Conforms |
| Forward compatibility | `QuoteData` stays open to additive upstream fields (no `serde(deny_unknown_fields)` in response position). | Conforms |

## Current Contract

### Quote Response Fidelity

`cow_sdk_orderbook::QuoteData` is the Rust mirror of the orderbook
`OrderParameters` schema returned in a `/quote` response. It is enrolled in
`parity/openapi/coverage.yaml` as `components.schemas.OrderParameters ->
cow_sdk_orderbook::QuoteData`, with the inventory
`parity/openapi/order-parameters-inventory.yaml` and the fixture
`parity/fixtures/orderbook/order_parameters.json`. Because the inventory
enumerates every `OrderParameters` property, the coverage validator checks that
`QuoteData` carries each one — closing the gap where the `OrderQuoteResponse`
`quote` field was validated only as an opaque object.

The quote response carries the network-fee inputs `gasAmount`, `gasPrice`, and
`sellTokenPrice`, which the orderbook combines as
`feeAmount = ceil((gasAmount * gasPrice) / sellTokenPrice)`. `QuoteData` models
all three, plus `signingScheme`, and the optional `appDataHash` echo.

### Price-Quality Default

`OrderQuoteRequest` defaults `priceQuality` to `optimal` and always serializes
the field. `optimal` is the estimate the orderbook intends order creation to
build from, so the unmanaged "quote then sign" path produces a submittable order
by default. `verified` (simulated) and `fast` (preview) remain selectable.

### Read-Only Quote Costs

The quote network-cost fields (`feeAmount`, `gasAmount`, `gasPrice`,
`sellTokenPrice`) are read-only on `QuoteData`, consistent with
[ADR 0021](../adr/0021-orderbook-total-fee-policy.md). They are populated by
deserializing the `/quote` response and read through accessors; no public
builder exposes a setter. Compile-fail witnesses on the crate prove the absence
of `.fee_amount(...)` and `.gas_amount(...)` builder setters.

### Projection Parity

The quote-amounts projection that derives the signable order amounts from a
`/quote` response matches the orderbook quote-amounts algorithm. It restores the
network fee on a sell order's signed sell amount (the settlement contract
deducts it on-chain) and carries it on top of a buy order's signed sell amount.
The projection is locked by `crates/trading/tests/quote_projection_parity.rs`.

### Trust Posture

The SDK signs the client-computed app-data digest and the projected amounts and
validates the resulting order through the client-side bounds validator
([ADR 0015](../adr/0015-client-side-order-bounds-validator.md)) before
submission. It does not field-bind the quote response to the quote request; the
defensive layer is the bounds validator on the projected order, not a per-field
equality check against the request.

### Quote Request

`OrderQuoteRequest` models the orderbook quote `oneOf`s as typed Rust so that an
invalid request is unrepresentable rather than rejected at validation time:

- `QuoteValidity` carries either `validTo` or `validFor`, never both.
- `OrderQuoteSide` carries exactly one side, with `SellAmount` distinguishing the
  before-fee and after-fee sell amount.
- `QuoteSigningScheme` encodes the scheme-specific constraints — only EIP-1271
  carries a `verificationGasLimit`, and an ECDSA scheme can never be on-chain —
  enforced on the wire by a `try_from` deserialization guard.

App-data on the request stays modeled as the `appData`/`appDataHash` field pair
(`QuoteAppData`), consistent with `OrderCreation`, and is serialized through the
same hand-rolled routing so every form is wire-correct: a hash-only request
serializes the hash under `appData` (the services `Hash` form) rather than an
`appDataHash`-only body the orderbook rejects. The retained
`OrderQuoteRequest::validate` hook is now infallible because the invariants it
once checked are enforced by the type system.

## Evidence

Primary implementation points:

- `crates/orderbook/src/types/quote.rs`
- `crates/orderbook/src/types/enums.rs`
- `crates/orderbook/src/lib.rs`
- `crates/trading/src/slippage/amounts.rs`
- `parity/openapi/coverage.yaml`
- `parity/openapi/order-parameters-inventory.yaml`
- `parity/fixtures/orderbook/order_parameters.json`
- `parity/fixtures/orderbook/order_quote_response.json`

Primary regression coverage:

- `crates/orderbook/tests/wire_contract.rs::openapi_response_dtos_roundtrip_required_fixture_fields`
- `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs::quote_data_surfaces_gas_estimates_through_read_only_accessors`
- `crates/orderbook/tests/wire_contract.rs::promoted_amount_dtos_roundtrip_byte_identical`
- `crates/orderbook/tests/types_contract.rs`
- `crates/trading/tests/quote_projection_parity.rs::sell_signable_amounts_fold_network_cost_into_sell`
- `crates/trading/tests/quote_projection_parity.rs::buy_signable_amounts_inflate_sell_by_network_cost`

Validation surface:

```text
cargo run --manifest-path scripts/parity-maintainer/Cargo.toml -- openapi-coverage --validate
cargo test -p cow-sdk-orderbook --test fee_amount_is_not_a_public_builder_setter
cargo test -p cow-sdk-orderbook --test wire_contract
cargo test -p cow-sdk-orderbook --doc
cargo test -p cow-sdk-trading --test quote_projection_parity
```
