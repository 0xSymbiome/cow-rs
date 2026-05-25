# `fuzz_rpc_error_payload_serde` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_rpc_error_payload_serde.rs`,
which fuzzes the `cow_sdk_browser_wallet::RpcErrorPayload` serde and
`Debug`-redaction boundary. The seed corpus covers the JSON-RPC error
payload shapes the browser-wallet RPC normalization pipeline emits and
reparses through this DTO.

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
  `Redacted<String>` wrapper must not surface through `Debug` rendering.
- adversarial: `seed-adversarial-06-oversized-code.bin` is a JSON-RPC
  error with a numerically large code that must remain a stable typed
  value.
