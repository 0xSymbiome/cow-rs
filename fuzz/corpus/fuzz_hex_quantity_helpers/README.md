# `fuzz_hex_quantity_helpers` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_hex_quantity_helpers.rs`. The
named browser-wallet helpers (`hex_quantity`, `parse_chain_id_value`,
`parse_quantity_to_decimal`) are crate-private and reached only through
`async fn` wrappers, so the harness today exercises the adjacent public
`RpcErrorPayload` deserialization seam that participates in the same
wallet RPC normalization pipeline. The seed corpus therefore covers the
JSON-RPC error payload shapes the helpers would observe in production.

Seed sources:

- canonical: `seed-canonical-00-rpc-error-rejected.bin` carries the
  EIP-1193 user-rejected JSON-RPC error payload referenced by the
  browser-wallet RPC normalization parity (`parity/fixtures/core.json`
  id `core-runtime-trait-surfaces`).
- canonical: `seed-canonical-01-rpc-error-disconnected.bin` carries a
  documented disconnection error payload.
- boundary: `seed-boundary-02-empty.bin` is an empty body that must be
  rejected without panic.
- boundary: `seed-boundary-03-null-data.bin` is a JSON-RPC error with
  an explicit `null` data field, exercising the documented optional
  data branch.
- adversarial: `seed-adversarial-04-non-json.bin` is non-JSON ASCII
  noise that must be rejected without panic.
- adversarial: `seed-adversarial-05-credential-bearing.bin` is a
  JSON-RPC error message containing credential-shaped material that the
  redaction wrappers must not surface through `Debug` rendering.
- adversarial: `seed-adversarial-06-oversized-code.bin` is a JSON-RPC
  error with a numerically large code that must remain a stable typed
  value.
