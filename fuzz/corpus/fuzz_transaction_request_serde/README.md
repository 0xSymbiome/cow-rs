# `fuzz_transaction_request_serde` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_transaction_request_serde.rs`,
which fuzzes the `cow_sdk_core::TransactionRequest` serde boundary that
feeds the browser-wallet `eth_sendTransaction` and `eth_call`
normalization pipeline.

Seed sources:

- canonical: `seed-canonical-00-transfer.bin` carries a minimal native
  transfer transaction shape that mirrors the documented provider
  parity for transaction submission (`parity/fixtures/core.json` id
  `core-runtime-trait-surfaces`).
- canonical: `seed-canonical-01-call-with-data.bin` carries a contract
  call shape with `to`, `data`, and `gasLimit` populated.
- boundary: `seed-boundary-02-empty.bin` is an empty body that must be
  rejected without panic.
- boundary: `seed-boundary-03-all-none.bin` is the empty-object
  payload `{}` that must deserialize into a request with every
  optional field absent.
- adversarial: `seed-adversarial-04-malformed-address.bin` is a
  `TransactionRequest` whose `to` is not a valid address; the typed
  address validator must fail closed without panicking.
- adversarial: `seed-adversarial-05-non-json.bin` is non-JSON noise
  that must be rejected by the outer deserializer.
- adversarial: `seed-adversarial-06-oversized-value.bin` carries a
  numerically large value field that exercises the documented
  unsigned-uint256 boundary in `Amount`.
