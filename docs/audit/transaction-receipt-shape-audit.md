# Transaction Receipt Shape Audit

Status: Current
Last reviewed: 2026-06-14
Owning surface: `cow-sdk-core` transaction lifecycle types and adapter receipt conversions
Refresh trigger: ADR 0038 - transaction lifecycle types
Related docs:
- [ADR 0038](../adr/0038-transaction-lifecycle-types.md)
- [ADR 0037](../adr/0037-alloy-umbrella-adapter.md)

## Scope

This audit covers:

- `TransactionBroadcast`, `TransactionStatus`, and rich `TransactionReceipt`
  semantics in `cow-sdk-core`
- signer submission return types across native Alloy and browser-wallet
  adapters
- receipt population in `cow-sdk-alloy-provider`, `cow-sdk-alloy`, and
  `cow-sdk-browser-wallet`
- strict browser-wallet malformed-field handling for JSON-RPC receipt payloads

It does not cover a higher-level wait helper or live-chain inclusion timing.

## Outcome Summary

| Area | Reviewed contract | Result |
| --- | --- | --- |
| Type split | Signers return `TransactionBroadcast`; providers return `TransactionReceipt` for mined observation | Conforms |
| Alloy umbrella | `send_transaction` reads the broadcast hash through `*pending.tx_hash()` and does not wait for confirmation | Conforms |
| Alloy provider | Receipt conversion populates status, block, gas, sender, and recipient fields | Conforms |
| Browser wallet | Optional receipt fields are absent-tolerant and present-malformed strict | Conforms |
| Cross-adapter timing | Alloy umbrella and browser-wallet submission do not call `eth_getTransactionReceipt` during broadcast | Conforms |

## Current Contract

### Type Definitions

`TransactionBroadcast` is the signer submission acknowledgement and carries
only `transaction_hash`. `TransactionStatus` carries `Success` or `Reverted`.
`TransactionReceipt` carries `transaction_hash` plus optional `status`,
`block_number`, `block_hash`, `gas_used`, `from`, and `to` fields.

### Adapter Conformance

| Adapter | Broadcast contract | Receipt contract |
| --- | --- | --- |
| `cow-sdk-alloy` umbrella | Reads broadcast hash via `*pending.tx_hash()`; returns `TransactionBroadcast`; no confirmation wait | Delegates receipt lookup to `cow-sdk-alloy-provider` |
| `cow-sdk-alloy-provider` | n/a | `alloy_to_cow_receipt` populates `transaction_hash`, `status` via `receipt.inner.status_or_post_state().as_eip658()`, `block_number`, `block_hash`, `gas_used`, `from`, and `to`; post-state receipts map status to `None` |
| `cow-sdk-browser-wallet` | Returns `TransactionBroadcast` from the `eth_sendTransaction` hash response | Parses JSON-RPC receipts into the rich shape; null `to` stays `None` for contract creation |

### Browser-Wallet Parsing

The parser treats missing or `null` optional fields as `None`. It rejects
present malformed values for `status`, `blockNumber`, `blockHash`, `gasUsed`,
`from`, and `to` with `MalformedResponse`.

## Evidence

Primary implementation points:

- `crates/core/src/traits/transaction.rs`
- `crates/alloy/src/handle.rs`
- `crates/alloy-provider/src/conversion.rs`
- `crates/browser-wallet/src/provider/provider_impl.rs`

Primary regression coverage:

- `crates/core/tests/traits_contract.rs`
- `crates/alloy/tests/send_transaction_does_not_wait_for_confirmation.rs::send_transaction_does_not_dispatch_get_transaction_receipt`
- `crates/alloy/tests/provider_contract.rs::get_transaction_receipt_populates_rich_fields_from_alloy_receipt`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_populates_status_success`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_populates_status_reverted`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_returns_none_status_for_post_state_receipt`
- `crates/alloy-provider/src/conversion.rs::tests::alloy_to_cow_receipt_handles_contract_creation_no_to`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_populates_every_field_from_post_byzantium_response`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_handles_pre_byzantium_absent_status`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_handles_contract_creation_no_to`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_returns_none_when_chain_has_not_observed_tx`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_rejects_response_without_transaction_hash`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_returns_reverted_for_status_0x0`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_rejects_invalid_status_value`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_rejects_malformed_block_number`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_rejects_malformed_block_hash`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_rejects_malformed_gas_used`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_rejects_malformed_from`
- `crates/browser-wallet/tests/transaction_receipt_parsing.rs::parse_receipt_rejects_malformed_to`
- `tests/transaction_lifecycle_cross_adapter_invariant.rs`

Validation surface:

```text
cargo test -p cow-sdk-alloy --test send_transaction_does_not_wait_for_confirmation
cargo test -p cow-sdk-alloy-provider --lib
cargo test -p cow-sdk-browser-wallet --test transaction_receipt_parsing
cargo test -p cow-rs-workspace-tests --test transaction_lifecycle_cross_adapter_invariant
cargo check-property-citations
```
