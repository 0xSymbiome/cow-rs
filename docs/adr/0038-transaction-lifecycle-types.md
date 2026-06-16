# ADR 0038: Split Transaction Broadcast And Receipt Observation

- Status: Accepted
- Date: 2026-05-07
- Last reviewed: 2026-05-22
- Authors: [0xSymbiotic](https://github.com/0xSymbiotic)
- Tags: types, adapters, trading
- Related: [ADR 0010](0010-runtime-neutral-async-and-transport-posture.md), [ADR 0024](0024-asyncprovider-asyncsigningprovider-capability-split.md), [ADR 0037](0037-alloy-umbrella-adapter.md), [ADR 0052](0052-alloy-primitives-canonical-primitive-layer.md)

## Decision

Signer-backed submission returns `TransactionBroadcast`, a hash-only broadcast
acknowledgement. Provider receipt lookup returns `TransactionReceipt`, a mined
observation shape with optional `status`, `block_number`, `block_hash`,
`gas_used`, `from`, and `to` fields. `TransactionStatus` represents the
post-EIP-658 success or reverted bit when the backend exposes it.

## Why

Broadcast acknowledgement and mined receipt observation happen at different
times and through different RPC methods. A single hash-shaped receipt type made
adapters look equivalent even when one returned immediately and another waited
for confirmation. Splitting the types preserves the immediate submission
contract while giving receipt-capable providers a place to expose lifecycle
fields.

The Alloy conversion reads status through
`receipt.inner.status_or_post_state().as_eip658()`, so pre-Byzantium post-state
receipts remain `None` instead of being coerced into success. The wasm callback
receipt parser is tolerant when optional fields are absent and strict when
present fields are malformed.

## Must Remain True

- Public surface: `Signer::send_transaction` returns `TransactionBroadcast`;
  provider receipt lookups return `Option<TransactionReceipt>`.
- Runtime and support: adapters must not poll for receipts during
  `send_transaction`; mined observation is explicit through provider receipt
  lookup or a higher-level wait helper.
- Validation and review: Alloy and wasm callback tests must prove both
  broadcast timing and rich receipt population, including absent-status and
  malformed-field cases.
- Cost: adapters now maintain explicit conversion code for receipt fields
  instead of returning a hash-only placeholder.
- Wait verdict: `WaitError::reverted(&self) -> Option<&TransactionReceipt>`
  returns the reverted receipt only when a receipt wait failed because the mined
  transaction reverted (and `None` for `Broadcast`/`Lookup`/`Timeout`/`Cancelled`);
  a receipt reaches `WaitError::Reverted` only under `WaitOptions::require_success`,
  while an inclusion-only wait returns `Ok(receipt)`. `WaitError` is generic over
  the caller's signer/provider error types, so it stays outside the `ErrorClass`
  family.

## Alternatives Rejected

- Keep `TransactionReceipt` as the signer return type: this continued to imply
  mined observation where only a broadcast hash was known.
- Use the Alloy `receipt.status()` accessor: it coerces post-state receipts to
  success and hides the absence of an EIP-658 status bit.
- Make wasm callback receipt parsing silently ignore malformed optional fields:
  absence is normal across wallet providers, but malformed present data should
  be diagnosable.

## Links

- [EIP-658](https://eips.ethereum.org/EIPS/eip-658)
- [CoW Protocol SDK](https://github.com/cowprotocol/cow-sdk)
- [Alloy `PendingTransactionBuilder`](https://docs.rs/alloy-provider/latest/alloy_provider/struct.PendingTransactionBuilder.html)
- [ADR 0037](0037-alloy-umbrella-adapter.md)
- See also: ADR 0030.

**Proven by:**

- [Transaction Receipt Shape Audit](../audit/transaction-receipt-shape-audit.md)
