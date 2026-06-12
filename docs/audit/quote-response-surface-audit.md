# Quote Response Surface Audit

Status: Current
Last reviewed: 2026-06-12
Owning surface: cow-sdk-orderbook quote request/response DTOs and cow-sdk-trading quote projection
Refresh trigger: changes to the quote DTOs (`OrderQuoteRequest`, `OrderQuoteResponse`, `QuoteData`), the orderbook quote OpenAPI schemas, the quote-amounts projection, the quote-echo binding (`ensure_matches`), or the `priceQuality` default
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
- the quote-echo binding: `OrderQuoteResponse::ensure_matches`, auto-invoked by
  `OrderbookApi::quote`, reconciles every request-determined field of the
  response against the request before the projection runs
- the SDK trust posture: the request-determined fields are bound to the request,
  the variable price leg stays free, the signed order binds the caller's balance
  sources, and the projected order is still validated by the client-side bounds
  validator
- the quote request payload's current validation contract

It does not cover order submission (other trading audits), app-data document
content, or composable-order quoting.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Quote response fidelity | `QuoteData` covers every required `OrderParameters` field — including `gasAmount`, `gasPrice`, `sellTokenPrice`, and `signingScheme` — and is enrolled in `parity/openapi/coverage.yaml`, where its required fields are checked against the vendored spec. | Conforms |
| Coverage enforcement | `openapi-coverage` checks `QuoteData` against the `OrderParameters` schema expanded in memory from the vendored spec, so a dropped or mistyped quote field fails the gate instead of passing silently. | Conforms |
| Price-quality default | `OrderQuoteRequest` defaults `priceQuality` to `optimal`, the submittable estimate, and always serializes it. | Conforms |
| Read-only quote costs | The quote network-cost fields are populated only from the `/quote` response and exposed through accessors; no public builder exposes a setter. | Conforms |
| Projection parity | The quote-amounts projection matches the orderbook quote-amounts algorithm and is locked by a parity regression test. | Conforms |
| Quote-echo binding | `OrderbookApi::quote` reconciles every request-determined field of the response against the request through `OrderQuoteResponse::ensure_matches`, failing closed with `OrderbookError::QuoteEchoMismatch`; the variable price leg stays free and the fixed-leg fold follows the services arithmetic per side basis. | Conforms |
| Trust posture | The request-determined fields are bound to the request and the signed order binds the caller's balance sources rather than the response echo; the variable price leg stays trusted, and the projected order is still validated through the bounds validator before submission. | Conforms |
| Forward compatibility | `QuoteData` stays open to additive upstream fields (no `serde(deny_unknown_fields)` in response position). | Conforms |

## Current Contract

### Quote Response Fidelity

`cow_sdk_orderbook::QuoteData` is the Rust mirror of the orderbook
`OrderParameters` schema returned in a `/quote` response. It is enrolled in
`parity/openapi/coverage.yaml` as `components.schemas.OrderParameters ->
cow_sdk_orderbook::QuoteData`, by expanding the `OrderParameters` inventory in memory from the vendored
spec. Because the expansion enumerates every `OrderParameters` property, the
coverage validator checks that `QuoteData` carries each one — closing the gap
where the `OrderQuoteResponse` `quote` field was validated only as an opaque
object.

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
It also folds the quote response's optional `protocolFeeBps` into the signed
amounts: the protocol fee enters the same composition as the partner fee and
slippage, with the partner-fee base taken from the reconstructed
before-protocol-fee amount, so a protocol fee combined with a partner fee
strictly lowers a sell order's signed buy amount. The signing and submission
lanes (`post_swap_order_from_quote`, the `EthFlow` transaction lane) default and
thread the same value, so the posted order signs the amounts the projection
previewed. The projection is locked by
`crates/trading/tests/quote_projection_parity.rs`, including the composition
goldens transcribed in
`parity/fixtures/trading/protocol_fee_partner_fee_composition.json`.

### Quote-Echo Binding

`OrderbookApi::quote` invokes `OrderQuoteResponse::ensure_matches` on every
response and fails closed with `OrderbookError::QuoteEchoMismatch` when a
request-determined field did not come back unchanged: the token pair, order
kind, owner (`from`, when the response carries it), partial-fill flag, both
balance sources, a pinned app-data hash (only when the request pinned one), an
absolute `validTo` (only the `validTo` validity form), an explicit receiver
(only when both sides carry one), and the fixed amount leg. The fixed-leg fold
mirrors the services quote arithmetic per side basis: a `sellAmountBeforeFee`
request holds `sellAmount + feeAmount == requested`, a `sellAmountAfterFee`
request holds `sellAmount == requested`, and a buy request holds `buyAmount ==
requested`. The variable price leg — the amount the solver quotes for the
unfixed side — is the answer to the request and is never constrained.
`QuoteEchoField` carries the typed discriminant so a caller can match the
specific field that diverged.

### Trust Posture

The SDK signs the client-computed app-data digest and the projected amounts.
The request-determined fields of the quote response are bound to the request by
the echo check above, and the signed order binds the caller's requested balance
sources rather than the response echo — so a coherent response that altered the
fixed leg or the balance sources fails closed before any signing path. The
variable price leg stays trusted, because it is the answer to the request. The
projected order is still validated through the client-side bounds validator
([ADR 0015](../adr/0015-client-side-order-bounds-validator.md)) before
submission: the bounds validator checks well-formedness (owner present, not
expired, non-zero amounts, token rules), which is orthogonal to the echo
check's intent-agreement guarantee (ADR 0058, revised 2026-06-11).

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
- `crates/orderbook/src/error.rs`
- `crates/orderbook/src/api.rs`
- `crates/orderbook/src/lib.rs`
- `crates/trading/src/order.rs`
- `crates/trading/src/slippage.rs`
- `crates/trading/src/post.rs`
- `parity/openapi/coverage.yaml`
- `parity/fixtures/orderbook/order_quote_response.json`
- `parity/fixtures/trading/protocol_fee_partner_fee_composition.json`

Primary regression coverage:

- `crates/orderbook/tests/wire_contract.rs::openapi_response_dtos_roundtrip_required_fixture_fields`
- `crates/orderbook/tests/fee_amount_is_not_a_public_builder_setter.rs::quote_data_surfaces_gas_estimates_through_read_only_accessors`
- `crates/orderbook/tests/wire_contract.rs::promoted_amount_dtos_roundtrip_byte_identical`
- `crates/orderbook/tests/types_contract.rs`
- `crates/trading/tests/quote_projection_parity.rs::sell_signable_amounts_fold_network_cost_into_sell`
- `crates/trading/tests/quote_projection_parity.rs::buy_signable_amounts_inflate_sell_by_network_cost`
- `crates/trading/tests/quote_projection_parity.rs::protocol_fee_partner_fee_composition_matches_upstream_goldens`
- `crates/trading/tests/post_contract.rs::post_from_quote_signs_the_order_the_quote_previewed_under_a_protocol_fee`
- `crates/orderbook/tests/quote_echo_contract.rs::honest_sell_before_fee_response_passes`
- `crates/orderbook/tests/quote_echo_contract.rs::inflated_fixed_sell_leg_fails`
- `crates/orderbook/tests/quote_echo_contract.rs::quote_fails_closed_end_to_end_on_a_tampered_fixed_leg`
- `crates/trading/tests/post_contract.rs::signed_balance_sources_bind_to_the_request_not_the_quote_response`

Validation surface:

```text
cargo parity-openapi-coverage
cargo test -p cow-sdk-orderbook --test fee_amount_is_not_a_public_builder_setter
cargo test -p cow-sdk-orderbook --test wire_contract
cargo test -p cow-sdk-orderbook --test quote_echo_contract
cargo test -p cow-sdk-orderbook --doc
cargo test -p cow-sdk-trading --test quote_projection_parity
cargo test -p cow-sdk-trading --test post_contract
```
