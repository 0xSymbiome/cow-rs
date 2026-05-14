# `fuzz_json_to_dyn_value` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_json_to_dyn_value.rs`. The
named browser-wallet helpers
(`json_to_dyn_value`, `parse_u256`, `parse_i256`, `bytes_from_json`,
`decode_hex`) are module-private and reached only through the `async
fn` wrapper `AsyncProvider::read_contract`, so the harness today
exercises the adjacent public `ContractCall` deserialization seam plus
free-form JSON parsing of the embedded ABI and argument strings.

Seed sources:

- canonical: `seed-canonical-00-erc20-balance.bin` carries an ERC20
  `balanceOf` shape that mirrors the documented contract-read parity
  case (`parity/fixtures/core.json` id `core-runtime-trait-surfaces`).
- canonical: `seed-canonical-01-fixed-bytes32.bin` carries a
  `bytes32`-returning call, exercising the documented fixed-bytes
  branch that `json_to_dyn_value` selects when downstream.
- boundary: `seed-boundary-02-empty.bin` is an empty body that must be
  rejected without panic.
- boundary: `seed-boundary-03-empty-strings.bin` is a `ContractCall`
  whose ABI and args strings are empty — both must fail closed.
- adversarial: `seed-adversarial-04-malformed-json.bin` is a
  `ContractCall` whose `args_json` is malformed; the downstream
  deserializer must fail closed without panicking.
- adversarial: `seed-adversarial-05-non-json.bin` is non-JSON noise
  that must be rejected by the outer `ContractCall` deserializer.
- adversarial: `seed-adversarial-06-oversized-integer.bin` carries an
  ABI argument with an oversized integer that would exceed the
  documented uint256 boundary if the coercion path were reached.
