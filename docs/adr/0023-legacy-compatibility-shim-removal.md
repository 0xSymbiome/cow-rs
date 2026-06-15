# ADR 0023: Remove Legacy Compatibility Shims That Produced Protocol-Incorrect Order Digests

- Status: Superseded by [ADR 0059](0059-hash-concrete-orderdata-directly.md)
- Date: 2026-04-24
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: contracts, core, hashing, compatibility

## Superseded

The removal of the legacy compatibility shims (`OrderModel`, `QuoteModel`,
`hash_order_for_contract`, `uid_for_contract`, `compatibility_order`) that
produced protocol-incorrect digests is now recorded in
[ADR 0059](0059-hash-concrete-orderdata-directly.md), which establishes the
concrete `cow_sdk_core::OrderData` as the sole order-identity path and removes
both the legacy shims and the contracts-layer intermediate order types.
