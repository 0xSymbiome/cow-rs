# Quote Request App-Data Fix Review

Status: Current
Last reviewed: 2026-05-30
Owning surface: `cow_sdk_orderbook` quote-request app-data wire shape (`OrderQuoteRequest`, `QuoteAppData`)
Refresh trigger: changes to the orderbook `/quote` app-data wire contract, the `OrderQuoteRequest` app-data builders, or `QuoteAppData` serialization or deserialization
Related docs:
- [ADR 0058](../adr/0058-typed-quote-request-response-surface.md)
- [Quote Response Surface Audit](quote-response-surface-audit.md)

## Scope

This fix review covers:

- the app-data wire shape `OrderQuoteRequest` produces for the `/quote`
  endpoint across the hash-only, document-only, document-plus-hash, and default
  forms
- the round-trip stability of `QuoteAppData` serialization and deserialization

It does not cover the order-submission app-data surface (`OrderCreation`), which
routes through the same shared wire helper and is reviewed under the wire DTO
coverage record.

## Findings Summary

| Finding | Expected closure condition | Result |
| --- | --- | --- |
| `OrderQuoteRequest::with_app_data_hash` paired the requested hash with the constructor's placeholder document, producing a document-plus-hash body whose document the orderbook re-hashes and rejects | a hash-only request serializes the hash under `appData` with no `appDataHash` key | Closed |
| `QuoteAppData` deserialization bucketed a lone `appData` hash as a full document, so a hash-only request did not round-trip and its accessors were inaccurate | a lone `appData` hash resolves into the hash slot and the request round-trips | Closed |

## Findings

### Hash-only quote requests serialize the orderbook `Hash` form

`OrderQuoteRequest::new` attaches no app-data by default. The orderbook treats an
omitted app-data field as the zero app-data hash, so the default request carries
neither an `appData` nor an `appDataHash` key. `with_app_data_hash` attaches an
explicit hash that serializes under the `appData` key (the orderbook `Hash`
form) with no `appDataHash` key; `with_app_data` attaches a full document under
`appData`; and calling both pins the document's expected hash as the
document-plus-hash form (`appData` plus `appDataHash`). Each form is a wire shape
the orderbook accepts, and the order-submission path that composes a full
document with its hash is unchanged.

### Quote request app-data round-trips

`QuoteAppData` deserialization resolves a lone `appData` that is itself a 32-byte
hash into the hash slot, matching the orderbook's own app-data parsing. A
hash-only, document-only, document-plus-hash, or default request therefore
serializes and deserializes back to an equal value, and the `app_data_hash` and
`full_app_data` accessors report a decoded request accurately.

## Evidence

Primary implementation points:

- `crates/orderbook/src/types/quote.rs` (`OrderQuoteRequest::new`, `OrderQuoteRequest::with_app_data`, `OrderQuoteRequest::with_app_data_hash`)
- `crates/orderbook/src/types/app_data.rs` (`QuoteAppData` `Serialize` and `Deserialize`)

Primary regression coverage:

- `crates/orderbook/tests/types_contract.rs::quote_request_app_data_routes_to_server_valid_wire_shapes`
- `crates/orderbook/tests/types_contract.rs::quote_request_defaults_match_transport_contract`
- `crates/orderbook/tests/invariant_contract.rs::quote_request_app_data_and_pagination_shape_roundtrip_without_normalization`

Validation surface:

```text
cargo test -p cow-sdk-orderbook
```
