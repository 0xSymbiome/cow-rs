# `fuzz_contract_call_serde` Corpus

This corpus seeds `fuzz/fuzz_targets/fuzz_contract_call_serde.rs`,
which fuzzes the `cow_sdk_core::ContractCall` serde boundary plus
free-form JSON parsing of the embedded `abi_json` and `args_json`
strings the wallet contract-read pipeline forwards into the
crate-private dyn-value coercion path.

Seed sources:

- canonical: `seed-canonical-00-erc20-balance.bin` carries an ERC20
  `balanceOf` shape that mirrors the documented contract-read
  `ContractCall` contract pinned by
  `crates/core/tests/traits_contract.rs`.
- canonical: `seed-canonical-01-fixed-bytes32.bin` carries a
  `bytes32`-returning call, exercising the documented fixed-bytes
  branch the downstream `json_to_dyn_value` coercion would select.
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
